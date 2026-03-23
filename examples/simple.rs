use std::time::Duration;

use opentelemetry::global;
use opentelemetry::trace::{Span, SpanKind, Tracer};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use rdkafka::ClientConfig;

use opentelemetry_instrumentation_rdkafka::{TracingConsumer, TracingProducer};

#[tokio::main]
async fn main() {
    // Set up OTel: stdout exporter + W3C TraceContext propagator
    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    global::set_tracer_provider(provider.clone());
    global::set_text_map_propagator(TraceContextPropagator::new());

    let brokers = "localhost:9092";
    let topic = "otel-test-topic";
    let group_id = "otel-test-group";

    // Producer
    let base_producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Failed to create producer");

    let producer = TracingProducer::new(base_producer);

    for i in 0..3 {
        let payload = format!("message-{}", i);
        let key = format!("key-{}", i);
        match producer
            .send(
                FutureRecord::to(topic).payload(&payload).key(&key),
                Duration::from_secs(5),
            )
            .await
        {
            Ok(delivery) => {
                println!(
                    "Sent message {} to partition {} offset {}",
                    i, delivery.partition, delivery.offset
                );
            }
            Err((err, _)) => {
                eprintln!("Failed to send message {}: {}", i, err);
            }
        }
    }

    // Consumer
    let base_consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create consumer");

    base_consumer
        .subscribe(&[topic])
        .expect("Failed to subscribe");

    let consumer = TracingConsumer::new(base_consumer, group_id);

    let tracer = global::tracer("simple-example");
    for _ in 0..3 {
        match consumer.recv().await {
            Ok((msg, producer_cx)) => {
                let payload = rdkafka::message::Message::payload(&msg)
                    .map(|p| std::str::from_utf8(p).unwrap_or("<non-utf8>"))
                    .unwrap_or("<empty>");
                println!("Received: {}", payload);

                // Create a process span using the extracted context
                let mut process_span = tracer
                    .span_builder(format!(
                        "{} process",
                        rdkafka::message::Message::topic(&msg)
                    ))
                    .with_kind(SpanKind::Consumer)
                    .start_with_context(&tracer, &producer_cx);
                // ... business logic here ...
                process_span.end();
            }
            Err(err) => {
                eprintln!("Error receiving message: {}", err);
            }
        }
    }

    provider
        .shutdown()
        .expect("Failed to shutdown tracer provider");
    println!("Done!");
}
