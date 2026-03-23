//! OpenTelemetry [Messaging Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/messaging/)
//! attribute constants for Kafka.

use opentelemetry::KeyValue;

// Core attributes
pub const MESSAGING_SYSTEM: &str = "messaging.system";
pub const MESSAGING_DESTINATION_NAME: &str = "messaging.destination.name";
pub const MESSAGING_OPERATION_TYPE: &str = "messaging.operation.type";
pub const MESSAGING_CLIENT_ID: &str = "messaging.client_id";
pub const MESSAGING_MESSAGE_ID: &str = "messaging.message.id";
pub const MESSAGING_MESSAGE_BODY_SIZE: &str = "messaging.message.body.size";
pub const MESSAGING_BATCH_MESSAGE_COUNT: &str = "messaging.batch.message_count";

// Kafka-specific attributes
pub const MESSAGING_KAFKA_DESTINATION_PARTITION: &str = "messaging.kafka.destination.partition";
pub const MESSAGING_KAFKA_MESSAGE_OFFSET: &str = "messaging.kafka.message.offset";
pub const MESSAGING_KAFKA_CONSUMER_GROUP: &str = "messaging.kafka.consumer.group";
pub const MESSAGING_KAFKA_MESSAGE_KEY: &str = "messaging.kafka.message.key";
pub const MESSAGING_KAFKA_MESSAGE_TOMBSTONE: &str = "messaging.kafka.message.tombstone";

/// Returns the base attributes common to all messaging spans:
/// `messaging.system`, `messaging.destination.name`, and `messaging.operation.type`.
pub fn base_attributes(topic: &str, operation: &str) -> Vec<KeyValue> {
    vec![
        KeyValue::new(MESSAGING_SYSTEM, "kafka"),
        KeyValue::new(MESSAGING_DESTINATION_NAME, topic.to_owned()),
        KeyValue::new(MESSAGING_OPERATION_TYPE, operation.to_owned()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_attributes_returns_system_destination_and_operation() {
        let attrs = base_attributes("my-topic", "publish");
        assert_eq!(attrs.len(), 3);
        assert_eq!(attrs[0].key.as_str(), MESSAGING_SYSTEM);
        assert_eq!(attrs[0].value.as_str(), "kafka");
        assert_eq!(attrs[1].key.as_str(), MESSAGING_DESTINATION_NAME);
        assert_eq!(attrs[1].value.as_str(), "my-topic");
        assert_eq!(attrs[2].key.as_str(), MESSAGING_OPERATION_TYPE);
        assert_eq!(attrs[2].value.as_str(), "publish");
    }
}
