//! Stripe webhook handling

use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{debug, info, warn};

use crate::error::{PaymentError, PaymentResult};
use crate::types::{WebhookEvent, WebhookEventType};
use crate::PaymentConfig;

/// Webhook handler for Stripe events
pub struct WebhookHandler {
    /// Webhook signing secret
    signing_secret: String,
    /// Tolerance for timestamp validation (seconds)
    timestamp_tolerance: u64,
}

impl WebhookHandler {
    /// Create new webhook handler
    #[must_use]
    pub fn new(config: &PaymentConfig) -> Self {
        Self {
            signing_secret: config.stripe_webhook_secret.clone(),
            timestamp_tolerance: 300, // 5 minutes
        }
    }

    /// Set timestamp tolerance
    #[must_use]
    pub fn with_tolerance(mut self, secs: u64) -> Self {
        self.timestamp_tolerance = secs;
        self
    }

    /// Verify and parse webhook event
    pub fn verify_and_parse(
        &self,
        payload: &str,
        signature_header: &str,
    ) -> PaymentResult<WebhookEvent> {
        // Parse signature header
        let sig_parts = self.parse_signature_header(signature_header)?;

        // Verify timestamp is recent
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if current_time > sig_parts.timestamp + self.timestamp_tolerance {
            warn!(
                "Webhook timestamp too old: {} vs current {}",
                sig_parts.timestamp, current_time
            );
            return Err(PaymentError::InvalidSignature);
        }

        // Verify signature
        let signed_payload = format!("{}.{}", sig_parts.timestamp, payload);
        if !self.verify_signature(&signed_payload, &sig_parts.signatures) {
            warn!("Webhook signature verification failed");
            return Err(PaymentError::InvalidSignature);
        }

        debug!("Webhook signature verified successfully");

        // Parse event
        self.parse_event(payload)
    }

    /// Parse Stripe signature header
    fn parse_signature_header(&self, header: &str) -> PaymentResult<SignatureParts> {
        let mut timestamp = 0u64;
        let mut signatures = Vec::new();

        for part in header.split(',') {
            let mut kv = part.split('=');
            let key = kv.next().unwrap_or("").trim();
            let value = kv.next().unwrap_or("").trim();

            match key {
                "t" => {
                    timestamp = value.parse().unwrap_or(0);
                }
                "v1" => {
                    signatures.push(value.to_string());
                }
                _ => {}
            }
        }

        if timestamp == 0 || signatures.is_empty() {
            return Err(PaymentError::InvalidSignature);
        }

        Ok(SignatureParts {
            timestamp,
            signatures,
        })
    }

    /// Verify HMAC signature
    fn verify_signature(&self, signed_payload: &str, signatures: &[String]) -> bool {
        type HmacSha256 = Hmac<Sha256>;

        let secret_bytes = self.signing_secret.as_bytes();
        let Ok(mut mac) = HmacSha256::new_from_slice(secret_bytes) else {
            return false;
        };

        mac.update(signed_payload.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());

        // Check if any signature matches
        signatures.iter().any(|sig| sig == &expected)
    }

    /// Parse event from JSON payload
    fn parse_event(&self, payload: &str) -> PaymentResult<WebhookEvent> {
        let json: serde_json::Value = serde_json::from_str(payload).map_err(|e| {
            PaymentError::InvalidResponse(format!("Failed to parse webhook payload: {e}"))
        })?;

        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let event_type_str = json
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let event_type = WebhookEventType::from_stripe(event_type_str);

        let timestamp = json
            .get("created")
            .and_then(serde_json::Value::as_i64)
            .map_or_else(
                vaya_common::Timestamp::now,
                vaya_common::Timestamp::from_unix,
            );

        let data = json.get("data").cloned().unwrap_or_default();

        let payment_id = data
            .get("object")
            .and_then(|o| o.get("id"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let refund_id = if matches!(
            event_type,
            WebhookEventType::ChargeRefunded | WebhookEventType::RefundUpdated
        ) {
            payment_id.clone()
        } else {
            None
        };

        info!("Parsed webhook event: {} type={:?}", id, event_type);

        Ok(WebhookEvent {
            id,
            event_type,
            timestamp,
            payment_id,
            refund_id,
            data,
        })
    }
}

/// Parsed signature parts
struct SignatureParts {
    timestamp: u64,
    signatures: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_handler() -> WebhookHandler {
        let config = PaymentConfig::default().with_webhook_secret("whsec_test_secret");
        WebhookHandler::new(&config)
    }

    #[test]
    fn test_parse_signature_header() {
        let handler = create_test_handler();

        let header = "t=1234567890,v1=abc123,v1=def456";
        let parts = handler.parse_signature_header(header);

        assert!(parts.is_ok());
        let parts = parts.expect("Should parse");
        assert_eq!(parts.timestamp, 1234567890);
        assert_eq!(parts.signatures.len(), 2);
    }

    #[test]
    fn test_parse_signature_header_invalid() {
        let handler = create_test_handler();

        let header = "invalid=header";
        let parts = handler.parse_signature_header(header);
        assert!(parts.is_err());
    }

    #[test]
    fn test_parse_event() {
        let handler = create_test_handler();

        let payload = r#"{
            "id": "evt_123",
            "type": "payment_intent.succeeded",
            "created": 1234567890,
            "data": {
                "object": {
                    "id": "pi_123"
                }
            }
        }"#;

        let event = handler.parse_event(payload);
        assert!(event.is_ok());

        let event = event.expect("Should parse");
        assert_eq!(event.id, "evt_123");
        assert_eq!(event.event_type, WebhookEventType::PaymentIntentSucceeded);
        assert_eq!(event.payment_id, Some("pi_123".to_string()));
    }
}
