//! # opentelemetry-instrumentation-rdkafka
//!
//! OpenTelemetry instrumentation for [rust-rdkafka](https://docs.rs/rdkafka).
//!
//! This crate provides [`TracingProducer`] and [`TracingConsumer`] wrappers that
//! automatically generate OpenTelemetry spans and propagate trace context through
//! Kafka message headers using the W3C TraceContext format.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use opentelemetry_instrumentation_rdkafka::TracingProducer;
//! use rdkafka::producer::{FutureProducer, FutureRecord};
//! use rdkafka::ClientConfig;
//! use std::time::Duration;
//!
//! # async fn example() {
//! let base_producer: FutureProducer = ClientConfig::new()
//!     .set("bootstrap.servers", "localhost:9092")
//!     .create()
//!     .expect("Failed to create producer");
//!
//! let producer = TracingProducer::new(base_producer);
//! producer.send(
//!     FutureRecord::<str, str>::to("my-topic").payload("hello"),
//!     Duration::from_secs(5),
//! ).await.expect("Failed to send");
//! # }
//! ```
//!
//! ## Trace Topology
//!
//! Producer and consumer spans are connected via **Span Links** (not parent-child),
//! following the [OTel Messaging Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/messaging/).
//! The receive span is ended immediately; callers create their own process spans
//! using the returned [`Context`](opentelemetry::Context).

pub mod consumer;
pub mod producer;
pub mod propagation;
pub mod semantic;

pub use consumer::extract_context_from_message;
pub use consumer::TracingConsumer;
pub use producer::TracingProducer;
