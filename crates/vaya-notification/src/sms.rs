//! SMS client (Twilio)

use std::time::Duration;
use tracing::{debug, info, warn};

use vaya_common::{Timestamp, Uuid};

use crate::error::{NotificationError, NotificationResult};
use crate::templates::TemplateEngine;
use crate::types::*;
use crate::NotificationConfig;

/// Twilio API base URL
const TWILIO_API_BASE: &str = "https://api.twilio.com/2010-04-01";

/// SMS client using Twilio
pub struct SmsClient {
    /// HTTP client
    http_client: reqwest::Client,
    /// Account SID
    account_sid: String,
    /// Auth token
    auth_token: String,
    /// From phone number
    from_phone: String,
    /// Template engine
    templates: TemplateEngine,
    /// Max retries
    max_retries: u32,
    /// Sandbox mode
    sandbox_mode: bool,
}

impl SmsClient {
    /// Create new SMS client
    pub fn new(config: &NotificationConfig) -> NotificationResult<Self> {
        config.validate_sms()?;

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .map_err(|e| NotificationError::Configuration(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            http_client,
            account_sid: config.twilio_account_sid.clone(),
            auth_token: config.twilio_auth_token.clone(),
            from_phone: config.twilio_phone_number.clone(),
            templates: TemplateEngine::new(),
            max_retries: config.max_retries,
            sandbox_mode: config.sandbox_mode,
        })
    }

    /// Send an SMS
    pub async fn send(&self, request: &SmsRequest) -> NotificationResult<SmsResult> {
        request.validate()?;

        // Render template if needed
        let message = if let Some(ref template) = request.template {
            self.templates.render(template, &request.context)?
        } else {
            request.message.clone()
        };

        if self.sandbox_mode {
            info!(
                "Sandbox mode: would send SMS to {} with message '{}'",
                request.to_phone,
                if message.len() > 50 { &message[..50] } else { &message }
            );
            return Ok(SmsResult {
                message_sid: format!("sandbox_{}", Uuid::new_v4()),
                status: NotificationStatus::Sent,
                sent_at: Timestamp::now(),
                segments: self.calculate_segments(&message),
            });
        }

        let result = self.send_with_retry(&request.to_phone, &message).await?;

        Ok(result)
    }

    /// Calculate number of SMS segments
    fn calculate_segments(&self, message: &str) -> u8 {
        let len = message.len();
        if len <= 160 {
            1
        } else {
            // Multipart messages use 153 characters per segment
            ((len + 152) / 153) as u8
        }
    }

    /// Send with retry
    async fn send_with_retry(&self, to_phone: &str, message: &str) -> NotificationResult<SmsResult> {
        let mut last_error = NotificationError::ServiceUnavailable("No attempts made".to_string());

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
                debug!("Retry attempt {} after {:?}", attempt, delay);
            }

            match self.send_request(to_phone, message).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if e.is_retryable() && attempt < self.max_retries {
                        warn!("Retryable error on attempt {}: {:?}", attempt + 1, e);
                        last_error = e;
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        Err(last_error)
    }

    /// Send single request
    async fn send_request(&self, to_phone: &str, message: &str) -> NotificationResult<SmsResult> {
        let url = format!(
            "{}/Accounts/{}/Messages.json",
            TWILIO_API_BASE, self.account_sid
        );

        let params = [
            ("To", to_phone),
            ("From", &self.from_phone),
            ("Body", message),
        ];

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(NotificationError::from)?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if status.is_success() {
            let json: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| NotificationError::InvalidResponse(format!("Failed to parse response: {e}")))?;

            let message_sid = json
                .get("sid")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let sms_status = json
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("queued");

            let notification_status = match sms_status {
                "queued" => NotificationStatus::Queued,
                "sending" => NotificationStatus::Sent,
                "sent" => NotificationStatus::Sent,
                "delivered" => NotificationStatus::Delivered,
                "failed" => NotificationStatus::Failed,
                "undelivered" => NotificationStatus::Failed,
                _ => NotificationStatus::Queued,
            };

            let segments = json
                .get("num_segments")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            info!("SMS sent successfully: {} to {}", message_sid, to_phone);

            return Ok(SmsResult {
                message_sid,
                status: notification_status,
                sent_at: Timestamp::now(),
                segments,
            });
        }

        // Handle errors
        let error_json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
        let error_message = error_json
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or(&body);

        let error_code = error_json
            .get("code")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        match (status.as_u16(), error_code) {
            (401, _) => Err(NotificationError::Configuration("Invalid Twilio credentials".to_string())),
            (429, _) => Err(NotificationError::RateLimited { retry_after_secs: 60 }),
            (_, 21211) => Err(NotificationError::InvalidPhoneNumber(error_message.to_string())),
            (_, 21614) => Err(NotificationError::InvalidPhoneNumber("Phone number is not valid".to_string())),
            (_, 21608) => Err(NotificationError::SmsDeliveryFailed("Unverified number in trial".to_string())),
            _ => Err(NotificationError::SmsDeliveryFailed(error_message.to_string())),
        }
    }

    /// Get SMS status
    pub async fn get_status(&self, message_sid: &str) -> NotificationResult<NotificationStatus> {
        let url = format!(
            "{}/Accounts/{}/Messages/{}.json",
            TWILIO_API_BASE, self.account_sid, message_sid
        );

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .send()
            .await
            .map_err(NotificationError::from)?;

        if !response.status().is_success() {
            return Err(NotificationError::InvalidResponse(
                "Failed to fetch SMS status".to_string(),
            ));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            NotificationError::InvalidResponse(format!("Failed to parse response: {e}"))
        })?;

        let status = json
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        Ok(match status {
            "queued" => NotificationStatus::Queued,
            "sending" => NotificationStatus::Sent,
            "sent" => NotificationStatus::Sent,
            "delivered" => NotificationStatus::Delivered,
            "failed" => NotificationStatus::Failed,
            "undelivered" => NotificationStatus::Failed,
            _ => NotificationStatus::Queued,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> NotificationConfig {
        NotificationConfig::default()
            .with_twilio("AC123", "auth123", "+60123456789")
    }

    #[test]
    fn test_sms_client_creation() {
        let config = create_test_config();
        let client = SmsClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_sms_client_validation() {
        let config = NotificationConfig::default();
        let client = SmsClient::new(&config);
        assert!(client.is_err());
    }

    #[test]
    fn test_calculate_segments() {
        let config = create_test_config();
        let client = SmsClient::new(&config).expect("Should create");

        assert_eq!(client.calculate_segments("Hello"), 1);
        assert_eq!(client.calculate_segments(&"a".repeat(160)), 1);
        assert_eq!(client.calculate_segments(&"a".repeat(161)), 2);
        assert_eq!(client.calculate_segments(&"a".repeat(306)), 2);
        assert_eq!(client.calculate_segments(&"a".repeat(307)), 3);
    }
}
