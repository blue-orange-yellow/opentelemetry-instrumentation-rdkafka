use opentelemetry::{
    global,
    trace::{Link, Span, SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
use rdkafka::consumer::StreamConsumer;
use rdkafka::error::KafkaError;
use rdkafka::message::{BorrowedMessage, Message};

use crate::propagation::HeaderExtractor;
use crate::semantic::*;

const INSTRUMENTATION_NAME: &str = "opentelemetry-instrumentation-rdkafka";

pub struct TracingConsumer {
    inner: StreamConsumer,
    group_id: String,
}

impl TracingConsumer {
    pub fn new(consumer: StreamConsumer, group_id: impl Into<String>) -> Self {
        Self {
            inner: consumer,
            group_id: group_id.into(),
        }
    }

    pub fn inner(&self) -> &StreamConsumer {
        &self.inner
    }

    pub async fn recv(&self) -> Result<(BorrowedMessage<'_>, Context), KafkaError> {
        let msg = self.inner.recv().await?;

        let producer_cx = extract_context_from_message(&msg);
        let topic = msg.topic();
        let tracer = global::tracer(INSTRUMENTATION_NAME);

        let mut attributes = base_attributes(topic, "receive");
        attributes.push(KeyValue::new(
            MESSAGING_KAFKA_CONSUMER_GROUP,
            self.group_id.clone(),
        ));
        attributes.push(KeyValue::new(
            MESSAGING_KAFKA_DESTINATION_PARTITION,
            msg.partition() as i64,
        ));
        attributes.push(KeyValue::new(
            MESSAGING_KAFKA_MESSAGE_OFFSET,
            msg.offset(),
        ));
        if let Some(key) = msg.key() {
            if let Ok(key_str) = std::str::from_utf8(key) {
                attributes.push(KeyValue::new(MESSAGING_KAFKA_MESSAGE_KEY, key_str.to_owned()));
            }
        }

        let producer_span_context = producer_cx.span().span_context().clone();
        let mut span_builder = tracer
            .span_builder(format!("{} receive", topic))
            .with_kind(SpanKind::Consumer)
            .with_attributes(attributes);

        if producer_span_context.is_valid() {
            span_builder = span_builder.with_links(vec![Link::with_context(producer_span_context)]);
        }

        let mut span = span_builder.start(&tracer);
        span.end();

        Ok((msg, producer_cx))
    }
}

pub fn extract_context_from_message(msg: &BorrowedMessage<'_>) -> Context {
    match msg.headers() {
        Some(headers) => {
            let extractor = HeaderExtractor::new(headers);
            global::get_text_map_propagator(|propagator| propagator.extract(&extractor))
        }
        None => Context::new(),
    }
}
