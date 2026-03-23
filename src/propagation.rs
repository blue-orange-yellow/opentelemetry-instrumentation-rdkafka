use opentelemetry::propagation::Injector;
use rdkafka::message::{Header, Headers, OwnedHeaders};

pub struct HeaderInjector {
    entries: Vec<(String, String)>,
}

impl HeaderInjector {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn into_owned_headers(self) -> OwnedHeaders {
        self.into_owned_headers_with_existing(None)
    }

    pub fn into_owned_headers_with_existing(self, existing: Option<OwnedHeaders>) -> OwnedHeaders {
        let mut headers = match existing {
            Some(existing) => {
                let count = existing.count();
                let mut h = OwnedHeaders::new_with_capacity(count + self.entries.len());
                for i in 0..count {
                    if let Ok(header) = existing.get_as::<[u8]>(i) {
                        if let Some(value) = header.value {
                            h = h.insert(Header {
                                key: header.key,
                                value: Some(value),
                            });
                        }
                    }
                }
                h
            }
            None => OwnedHeaders::new_with_capacity(self.entries.len()),
        };

        for (key, value) in &self.entries {
            headers = headers.insert(Header {
                key: key.as_str(),
                value: Some(value.as_bytes()),
            });
        }

        headers
    }
}

impl Injector for HeaderInjector {
    fn set(&mut self, key: &str, value: String) {
        self.entries.push((key.to_owned(), value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::propagation::Injector;
    use rdkafka::message::Headers;

    #[test]
    fn injector_collects_entries() {
        let mut injector = HeaderInjector::new();
        injector.set("traceparent", "00-abc-def-01".to_string());
        injector.set("tracestate", "foo=bar".to_string());

        let headers = injector.into_owned_headers();
        assert_eq!(headers.count(), 2);

        let h0 = headers.get_as::<str>(0).unwrap();
        assert_eq!(h0.key, "traceparent");
        assert_eq!(h0.value, Some("00-abc-def-01"));

        let h1 = headers.get_as::<str>(1).unwrap();
        assert_eq!(h1.key, "tracestate");
        assert_eq!(h1.value, Some("foo=bar"));
    }

    #[test]
    fn injector_merges_with_existing_headers() {
        let existing = OwnedHeaders::new()
            .insert(rdkafka::message::Header {
                key: "custom-header",
                value: Some("custom-value"),
            });

        let mut injector = HeaderInjector::new();
        injector.set("traceparent", "00-abc-def-01".to_string());

        let headers = injector.into_owned_headers_with_existing(Some(existing));
        assert_eq!(headers.count(), 2);

        let h0 = headers.get_as::<str>(0).unwrap();
        assert_eq!(h0.key, "custom-header");
        assert_eq!(h0.value, Some("custom-value"));

        let h1 = headers.get_as::<str>(1).unwrap();
        assert_eq!(h1.key, "traceparent");
        assert_eq!(h1.value, Some("00-abc-def-01"));
    }

    #[test]
    fn injector_handles_none_existing_headers() {
        let mut injector = HeaderInjector::new();
        injector.set("traceparent", "00-abc-def-01".to_string());

        let headers = injector.into_owned_headers_with_existing(None);
        assert_eq!(headers.count(), 1);
    }
}
