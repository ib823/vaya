//! Email client (SendGrid)

use std::time::Duration;
use tracing::{debug, info, warn};

use vaya_common::{Timestamp, Uuid};

use crate::error::{NotificationError, NotificationResult};
use crate::templates::TemplateEngine;
use crate::types::*;
use crate::NotificationConfig;

/// SendGrid API base URL
const SENDGRID_API_BASE: &str = "https://api.sendgrid.com/v3";

/// Email client using SendGrid
pub struct EmailClient {
    /// HTTP client
    http_client: reqwest::Client,
    /// API key
    api_key: String,
    /// Sender email
    from_email: String,
    /// Sender name
    from_name: String,
    /// Template engine
    templates: TemplateEngine,
    /// Max retries
    max_retries: u32,
    /// Sandbox mode
    sandbox_mode: bool,
}

impl EmailClient {
    /// Create new email client
    pub fn new(config: &NotificationConfig) -> NotificationResult<Self> {
        config.validate_email()?;

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .map_err(|e| NotificationError::Configuration(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            http_client,
            api_key: config.sendgrid_api_key.clone(),
            from_email: config.from_email.clone(),
            from_name: config.from_name.clone(),
            templates: TemplateEngine::new(),
            max_retries: config.max_retries,
            sandbox_mode: config.sandbox_mode,
        })
    }

    /// Send an email
    pub async fn send(&self, request: &EmailRequest) -> NotificationResult<EmailResult> {
        request.validate()?;

        // Render template if needed
        let (text_body, html_body) = if let Some(ref template) = request.template {
            let text = self.templates.render(&format!("{template}_text"), &request.context).ok();
            let html = self.templates.render(&format!("{template}_html"), &request.context)?;
            (text, Some(html))
        } else {
            (request.text_body.clone(), request.html_body.clone())
        };

        if self.sandbox_mode {
            info!(
                "Sandbox mode: would send email to {} with subject '{}'",
                request.to_email, request.subject
            );
            return Ok(EmailResult {
                message_id: format!("sandbox_{}", Uuid::new_v4()),
                status: NotificationStatus::Sent,
                sent_at: Timestamp::now(),
            });
        }

        let payload = self.build_sendgrid_payload(request, text_body, html_body)?;

        let result = self.send_with_retry(&payload).await?;

        Ok(result)
    }

    /// Build SendGrid API payload
    fn build_sendgrid_payload(
        &self,
        request: &EmailRequest,
        text_body: Option<String>,
        html_body: Option<String>,
    ) -> NotificationResult<serde_json::Value> {
        let mut to = serde_json::json!({
            "email": request.to_email
        });
        if let Some(ref name) = request.to_name {
            to["name"] = serde_json::json!(name);
        }

        let mut personalizations = serde_json::json!({
            "to": [to]
        });

        // Add custom headers
        if !request.headers.is_empty() {
            personalizations["headers"] = serde_json::to_value(&request.headers)
                .unwrap_or_default();
        }

        let mut payload = serde_json::json!({
            "personalizations": [personalizations],
            "from": {
                "email": self.from_email,
                "name": self.from_name
            },
            "subject": request.subject
        });

        // Add content
        let mut content = Vec::new();
        if let Some(text) = text_body {
            content.push(serde_json::json!({
                "type": "text/plain",
                "value": text
            }));
        }
        if let Some(html) = html_body {
            content.push(serde_json::json!({
                "type": "text/html",
                "value": html
            }));
        }
        payload["content"] = serde_json::json!(content);

        // Add reply-to
        if let Some(ref reply_to) = request.reply_to {
            payload["reply_to"] = serde_json::json!({
                "email": reply_to
            });
        }

        // Add categories/tags
        if !request.tags.is_empty() {
            payload["categories"] = serde_json::to_value(&request.tags)
                .unwrap_or_default();
        }

        // Add attachments
        if !request.attachments.is_empty() {
            let attachments: Vec<serde_json::Value> = request.attachments.iter().map(|a| {
                let mut attachment = serde_json::json!({
                    "content": a.content,
                    "filename": a.filename,
                    "type": a.content_type,
                    "disposition": match a.disposition {
                        AttachmentDisposition::Attachment => "attachment",
                        AttachmentDisposition::Inline => "inline",
                    }
                });
                if let Some(ref content_id) = a.content_id {
                    attachment["content_id"] = serde_json::json!(content_id);
                }
                attachment
            }).collect();
            payload["attachments"] = serde_json::json!(attachments);
        }

        // Add tracking settings
        payload["tracking_settings"] = serde_json::json!({
            "click_tracking": {"enable": true},
            "open_tracking": {"enable": true}
        });

        Ok(payload)
    }

    /// Send with retry
    async fn send_with_retry(&self, payload: &serde_json::Value) -> NotificationResult<EmailResult> {
        let mut last_error = NotificationError::ServiceUnavailable("No attempts made".to_string());

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
                debug!("Retry attempt {} after {:?}", attempt, delay);
            }

            match self.send_request(payload).await {
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
    async fn send_request(&self, payload: &serde_json::Value) -> NotificationResult<EmailResult> {
        let response = self
            .http_client
            .post(&format!("{}/mail/send", SENDGRID_API_BASE))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .map_err(NotificationError::from)?;

        let status = response.status();

        // SendGrid returns 202 Accepted on success
        if status.as_u16() == 202 {
            let message_id = response
                .headers()
                .get("X-Message-Id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_string();

            info!("Email sent successfully: {}", message_id);

            return Ok(EmailResult {
                message_id,
                status: NotificationStatus::Sent,
                sent_at: Timestamp::now(),
            });
        }

        // Handle errors
        let error_body = response.text().await.unwrap_or_default();

        match status.as_u16() {
            401 => Err(NotificationError::Configuration("Invalid API key".to_string())),
            429 => Err(NotificationError::RateLimited { retry_after_secs: 60 }),
            400 => {
                // Parse SendGrid error
                if let Ok(error) = serde_json::from_str::<serde_json::Value>(&error_body) {
                    let message = error
                        .get("errors")
                        .and_then(|e| e.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("Invalid request");
                    Err(NotificationError::DeliveryFailed(message.to_string()))
                } else {
                    Err(NotificationError::DeliveryFailed(error_body))
                }
            }
            _ => Err(NotificationError::ServiceUnavailable(format!(
                "HTTP {}: {}",
                status, error_body
            ))),
        }
    }

    /// Send bulk emails
    pub async fn send_bulk(&self, requests: &[EmailRequest]) -> Vec<NotificationResult<EmailResult>> {
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            results.push(self.send(request).await);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_client_creation() {
        let config = NotificationConfig::with_sendgrid("SG.test", "noreply@vaya.my");
        let client = EmailClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_email_client_validation() {
        let config = NotificationConfig::default();
        let client = EmailClient::new(&config);
        assert!(client.is_err());
    }

    #[test]
    fn test_build_payload() {
        let config = NotificationConfig::with_sendgrid("SG.test", "noreply@vaya.my");
        let client = EmailClient::new(&config).expect("Should create");

        let request = EmailRequest::new("user@example.com", "Test Subject")
            .with_name("John Doe")
            .with_text("Hello, world!");

        let payload = client.build_sendgrid_payload(
            &request,
            request.text_body.clone(),
            None,
        );

        assert!(payload.is_ok());
        let payload = payload.expect("Should build");
        assert!(payload.get("personalizations").is_some());
        assert!(payload.get("from").is_some());
        assert!(payload.get("subject").is_some());
    }
}
