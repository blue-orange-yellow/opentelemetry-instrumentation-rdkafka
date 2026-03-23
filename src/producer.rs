use std::time::Duration;

use opentelemetry::{
    global,
    trace::{SpanKind, Status, TraceContextExt, Tracer},
    Context, KeyValue,
};
use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::producer::future_producer::OwnedDeliveryResult;

use crate::propagation::HeaderInjector;
use crate::semantic::*;

const INSTRUMENTATION_NAME: &str = "opentelemetry-instrumentation-rdkafka";

pub struct TracingProducer {
    inner: FutureProducer,
}

impl TracingProducer {
    pub fn new(producer: FutureProducer) -> Self {
        Self { inner: producer }
    }

    pub fn inner(&self) -> &FutureProducer {
        &self.inner
    }

    pub async fn send<'a, K, P>(
        &self,
        mut record: FutureRecord<'a, K, P>,
        queue_timeout: Duration,
    ) -> OwnedDeliveryResult
    where
        K: ToBytes + ?Sized,
        P: ToBytes + ?Sized,
    {
        let topic = record.topic.to_owned();
        let tracer = global::tracer(INSTRUMENTATION_NAME);

        let mut attributes = base_attributes(&topic, "publish");

        if let Some(key) = record.key {
            if let Ok(key_str) = std::str::from_utf8(key.to_bytes()) {
                attributes.push(KeyValue::new(MESSAGING_KAFKA_MESSAGE_KEY, key_str.to_owned()));
            }
        }
        if let Some(partition) = record.partition {
            attributes.push(KeyValue::new(MESSAGING_KAFKA_DESTINATION_PARTITION, partition as i64));
        }
        if record.payload.is_none() {
            attributes.push(KeyValue::new(MESSAGING_KAFKA_MESSAGE_TOMBSTONE, true));
        }

        let span = tracer
            .span_builder(format!("{} publish", topic))
            .with_kind(SpanKind::Producer)
            .with_attributes(attributes)
            .start(&tracer);

        let cx = Context::current_with_span(span);
        let _guard = cx.clone().attach();

        let mut injector = HeaderInjector::new();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut injector);
        });

        let existing_headers = record.headers.take();
        record = record.headers(injector.into_owned_headers_with_existing(existing_headers));

        let result = self.inner.send(record, queue_timeout).await;

        let span = cx.span();
        match &result {
            Ok(delivery) => {
                span.set_attribute(KeyValue::new(
                    MESSAGING_KAFKA_DESTINATION_PARTITION,
                    delivery.partition as i64,
                ));
                span.set_attribute(KeyValue::new(
                    MESSAGING_KAFKA_MESSAGE_OFFSET,
                    delivery.offset,
                ));
            }
            Err((err, _msg)) => {
                span.set_status(Status::error(err.to_string()));
                span.add_event(
                    "exception",
                    vec![
                        KeyValue::new("exception.type", "KafkaError"),
                        KeyValue::new("exception.message", err.to_string()),
                    ],
                );
            }
        }
        span.end();

        result
    }
}
