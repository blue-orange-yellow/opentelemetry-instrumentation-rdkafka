# opentelemetry-instrumentation-rdkafka

OpenTelemetry instrumentation for [rust-rdkafka](https://docs.rs/rdkafka).

Provides `TracingProducer` and `TracingConsumer` wrappers that automatically generate OpenTelemetry spans and propagate trace context through Kafka message headers using the W3C TraceContext format.

## Features

- Automatic span creation for produce and consume operations
- Trace context propagation via Kafka message headers
- Existing user headers are preserved
- Follows [OTel Messaging Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/messaging/)
- Consumer spans connected to producer spans via **Span Links**

## Requirements

| Dependency | Version |
|---|---|
| `opentelemetry` | 0.31 |
| `rdkafka` | 0.39 |
| Rust | 1.75+ |

Building `rdkafka` with the `cmake-build` feature requires `cmake` to be installed on your system.

## Usage

### Producer

```rust
use opentelemetry_instrumentation_rdkafka::TracingProducer;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;

let base_producer: FutureProducer = ClientConfig::new()
    .set("bootstrap.servers", "localhost:9092")
    .create()
    .expect("Failed to create producer");

let producer = TracingProducer::new(base_producer);

// Spans and trace context injection happen automatically
producer.send(
    FutureRecord::<str, str>::to("my-topic").payload("hello").key("key-1"),
    Duration::from_secs(5),
).await?;
```

### Consumer

```rust
use opentelemetry_instrumentation_rdkafka::TracingConsumer;
use opentelemetry::trace::{Span, SpanKind, Tracer};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::ClientConfig;

let base_consumer: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", "localhost:9092")
    .set("group.id", "my-group")
    .set("auto.offset.reset", "earliest")
    .create()
    .expect("Failed to create consumer");

base_consumer.subscribe(&["my-topic"]).expect("Failed to subscribe");

let consumer = TracingConsumer::new(base_consumer, "my-group");

// Receive span is created and ended automatically
let (msg, producer_cx) = consumer.recv().await?;

// Create your own process span using the extracted context
let tracer = opentelemetry::global::tracer("my-service");
let mut span = tracer
    .span_builder("my-topic process")
    .with_kind(SpanKind::Consumer)
    .start_with_context(&tracer, &producer_cx);
// ... business logic ...
span.end();
```

### Standalone Context Extraction

If you use a custom consumer implementation, you can extract trace context directly:

```rust
use opentelemetry_instrumentation_rdkafka::extract_context_from_message;

let context = extract_context_from_message(&borrowed_message);
```

## Trace Topology

```
Producer service                        Consumer service
┌──────────────────┐                    ┌──────────────────────┐
│ {topic} publish   │ ── link ───────> │ {topic} receive       │
│  (SpanKind:       │   (via Kafka     │  (SpanKind: CONSUMER) │
│   PRODUCER)       │    headers)      └──────────────────────┘
└──────────────────┘
```

Producer and consumer spans are connected via Span Links (not parent-child), following the OTel Messaging Semantic Conventions. This avoids broken trace trees in 1:N fan-out scenarios common in Kafka.

## Semantic Conventions

This crate emits the following attributes per the [Messaging Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/messaging/):

| Attribute | Producer | Consumer |
|---|---|---|
| `messaging.system` | `"kafka"` | `"kafka"` |
| `messaging.destination.name` | topic | topic |
| `messaging.operation.type` | `"publish"` | `"receive"` |
| `messaging.kafka.destination.partition` | partition | partition |
| `messaging.kafka.message.offset` | offset | offset |
| `messaging.kafka.message.key` | key | key |
| `messaging.kafka.consumer.group` | - | group ID |
| `messaging.kafka.message.tombstone` | `true` (if no payload) | - |

## Running the Example

Start Kafka locally, then:

```bash
cargo run --example simple
```

See [`examples/simple.rs`](examples/simple.rs) for the full source.

## License

Apache-2.0
