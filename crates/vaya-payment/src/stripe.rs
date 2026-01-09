//! Stripe payment provider implementation

use async_trait::async_trait;
use std::time::Duration;
use tracing::{debug, info, warn};

use vaya_common::{CurrencyCode, MinorUnits, Price, Timestamp};

use crate::error::{PaymentError, PaymentResult};
use crate::types::{
    CardBrand, PaymentIntent, PaymentMethodDetails, PaymentRequest, PaymentStatus, Refund,
    RefundReason, RefundRequest, RefundStatus,
};
use crate::PaymentConfig;

/// Stripe base URL
const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";

/// Stripe client for payment processing
pub struct StripeClient {
    /// HTTP client
    http_client: reqwest::Client,
    /// Secret key
    secret_key: String,
    /// Max retries
    max_retries: u32,
}

impl StripeClient {
    /// Create new Stripe client
    pub fn new(config: &PaymentConfig) -> PaymentResult<Self> {
        config.validate()?;

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .map_err(|e| {
                PaymentError::Configuration(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            http_client,
            secret_key: config.stripe_secret_key.clone(),
            max_retries: config.max_retries,
        })
    }

    /// Create a payment intent
    pub async fn create_payment(&self, request: &PaymentRequest) -> PaymentResult<PaymentIntent> {
        request.validate()?;

        let mut params = vec![
            ("amount", request.amount.amount.as_i64().to_string()),
            ("currency", request.currency.as_str().to_lowercase()),
            ("receipt_email", request.customer_email.clone()),
            ("metadata[booking_ref]", request.booking_ref.clone()),
        ];

        if let Some(ref desc) = request.description {
            params.push(("description", desc.clone()));
        }

        if let Some(ref return_url) = request.return_url {
            params.push(("return_url", return_url.clone()));
        }

        // Add allowed payment methods
        for (i, method) in request.allowed_methods.iter().enumerate() {
            params.push((
                format!("payment_method_types[{i}]").leak(),
                method.stripe_type().to_string(),
            ));
        }

        // Add metadata
        for (key, value) in &request.metadata {
            params.push((format!("metadata[{key}]").leak(), value.clone()));
        }

        let idempotency_key = request
            .idempotency_key
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let response: serde_json::Value = self
            .post_with_retry(
                &format!("{STRIPE_API_BASE}/payment_intents"),
                &params,
                Some(&idempotency_key),
            )
            .await?;

        self.parse_payment_intent(&response)
    }

    /// Retrieve a payment intent
    pub async fn get_payment(&self, payment_id: &str) -> PaymentResult<PaymentIntent> {
        let url = format!("{STRIPE_API_BASE}/payment_intents/{payment_id}");
        let response: serde_json::Value = self.get(&url).await?;
        self.parse_payment_intent(&response)
    }

    /// Cancel a payment intent
    pub async fn cancel_payment(&self, payment_id: &str) -> PaymentResult<PaymentIntent> {
        let url = format!("{STRIPE_API_BASE}/payment_intents/{payment_id}/cancel");
        let response: serde_json::Value = self.post_with_retry(&url, &[], None).await?;
        self.parse_payment_intent(&response)
    }

    /// Create a refund
    pub async fn create_refund(&self, request: &RefundRequest) -> PaymentResult<Refund> {
        let mut params = vec![
            ("payment_intent", request.payment_id.clone()),
            ("reason", request.reason.stripe_reason().to_string()),
        ];

        if let Some(ref amount) = request.amount {
            params.push(("amount", amount.amount.as_i64().to_string()));
        }

        let idempotency_key = request
            .idempotency_key
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let response: serde_json::Value = self
            .post_with_retry(
                &format!("{STRIPE_API_BASE}/refunds"),
                &params,
                Some(&idempotency_key),
            )
            .await?;

        self.parse_refund(&response)
    }

    /// Get refund status
    pub async fn get_refund(&self, refund_id: &str) -> PaymentResult<Refund> {
        let url = format!("{STRIPE_API_BASE}/refunds/{refund_id}");
        let response: serde_json::Value = self.get(&url).await?;
        self.parse_refund(&response)
    }

    /// Make GET request
    async fn get(&self, url: &str) -> PaymentResult<serde_json::Value> {
        let response = self
            .http_client
            .get(url)
            .basic_auth(&self.secret_key, None::<&str>)
            .send()
            .await
            .map_err(PaymentError::from)?;

        self.handle_response(response).await
    }

    /// Make POST request with retry
    async fn post_with_retry(
        &self,
        url: &str,
        params: &[(&str, String)],
        idempotency_key: Option<&str>,
    ) -> PaymentResult<serde_json::Value> {
        let mut last_error = PaymentError::ServiceUnavailable("No attempts made".to_string());

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
                debug!("Retry attempt {} after {:?}", attempt, delay);
            }

            match self.post(url, params, idempotency_key).await {
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

    /// Make POST request
    async fn post(
        &self,
        url: &str,
        params: &[(&str, String)],
        idempotency_key: Option<&str>,
    ) -> PaymentResult<serde_json::Value> {
        let mut request = self
            .http_client
            .post(url)
            .basic_auth(&self.secret_key, None::<&str>)
            .form(params);

        if let Some(key) = idempotency_key {
            request = request.header("Idempotency-Key", key);
        }

        let response = request.send().await.map_err(PaymentError::from)?;
        self.handle_response(response).await
    }

    /// Handle HTTP response
    async fn handle_response(
        &self,
        response: reqwest::Response,
    ) -> PaymentResult<serde_json::Value> {
        let status = response.status();

        if status.is_success() {
            let json: serde_json::Value = response.json().await.map_err(|e| {
                PaymentError::InvalidResponse(format!("Failed to parse response: {e}"))
            })?;
            return Ok(json);
        }

        // Parse error response
        let error_body = response.text().await.unwrap_or_default();
        let error_json: serde_json::Value = serde_json::from_str(&error_body)
            .unwrap_or_else(|_| serde_json::json!({"error": {"message": error_body}}));

        let error = error_json.get("error").cloned().unwrap_or(error_json);
        let error_type = error.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let error_code = error.get("code").and_then(|v| v.as_str()).unwrap_or("");
        let error_message = error
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");

        // Map Stripe errors to our error types
        match (status.as_u16(), error_type, error_code) {
            (401, _, _) => Err(PaymentError::AuthenticationFailed(
                error_message.to_string(),
            )),
            (429, _, _) => Err(PaymentError::RateLimited {
                retry_after_secs: 60,
            }),
            (_, "card_error", "card_declined") => Err(PaymentError::CardDeclined {
                code: error
                    .get("decline_code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("generic_decline")
                    .to_string(),
                message: error_message.to_string(),
            }),
            (_, "card_error", "insufficient_funds") => Err(PaymentError::InsufficientFunds),
            (_, "card_error", "expired_card") => Err(PaymentError::ExpiredCard),
            (_, "card_error", _) => Err(PaymentError::InvalidCard(error_message.to_string())),
            (_, "invalid_request_error", "payment_intent_unexpected_state") => {
                // Check if already succeeded
                if error_message.contains("already succeeded") {
                    Err(PaymentError::AlreadyProcessed {
                        payment_id: "unknown".to_string(),
                    })
                } else {
                    Err(PaymentError::InvalidResponse(error_message.to_string()))
                }
            }
            _ => Err(PaymentError::ServiceUnavailable(format!(
                "{error_type}: {error_message}"
            ))),
        }
    }

    /// Parse payment intent from JSON response
    fn parse_payment_intent(&self, json: &serde_json::Value) -> PaymentResult<PaymentIntent> {
        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PaymentError::InvalidResponse("Missing payment intent ID".to_string()))?
            .to_string();

        let client_secret = json
            .get("client_secret")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let amount_cents = json
            .get("amount")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);

        let currency_str = json
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("myr");

        let currency = CurrencyCode::new(currency_str);
        let amount = Price::new(MinorUnits::new(amount_cents), currency);

        let status_str = json
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending");

        let status = match status_str {
            "requires_payment_method" | "requires_confirmation" => PaymentStatus::Pending,
            "requires_action" => PaymentStatus::RequiresAction,
            "processing" => PaymentStatus::Processing,
            "succeeded" => PaymentStatus::Succeeded,
            "canceled" => PaymentStatus::Cancelled,
            _ => PaymentStatus::Failed,
        };

        let created_at = json
            .get("created")
            .and_then(serde_json::Value::as_i64)
            .map_or_else(Timestamp::now, Timestamp::from_unix);

        let booking_ref = json
            .get("metadata")
            .and_then(|m| m.get("booking_ref"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let error_message = json
            .get("last_payment_error")
            .and_then(|e| e.get("message"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let next_action_url = json
            .get("next_action")
            .and_then(|a| a.get("redirect_to_url"))
            .and_then(|r| r.get("url"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let payment_method = self.parse_payment_method(json.get("payment_method_details"));

        info!("Parsed payment intent: {} status={:?}", id, status);

        Ok(PaymentIntent {
            id,
            client_secret,
            amount,
            status,
            payment_method,
            created_at,
            updated_at: Timestamp::now(),
            booking_ref,
            error_message,
            next_action_url,
        })
    }

    /// Parse payment method details
    fn parse_payment_method(
        &self,
        json: Option<&serde_json::Value>,
    ) -> Option<PaymentMethodDetails> {
        let details = json?;
        let method_type = details.get("type")?.as_str()?;

        match method_type {
            "card" => {
                let card = details.get("card")?;
                Some(PaymentMethodDetails::Card {
                    brand: CardBrand::from_stripe(
                        card.get("brand")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown"),
                    ),
                    last4: card
                        .get("last4")
                        .and_then(|v| v.as_str())
                        .unwrap_or("****")
                        .to_string(),
                    exp_month: card
                        .get("exp_month")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(1) as u8,
                    exp_year: card
                        .get("exp_year")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(2025) as u16,
                })
            }
            "fpx" => {
                let fpx = details.get("fpx")?;
                Some(PaymentMethodDetails::Fpx {
                    bank: fpx
                        .get("bank")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                })
            }
            "grabpay" => Some(PaymentMethodDetails::GrabPay),
            other => Some(PaymentMethodDetails::Other {
                method_type: other.to_string(),
            }),
        }
    }

    /// Parse refund from JSON response
    fn parse_refund(&self, json: &serde_json::Value) -> PaymentResult<Refund> {
        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PaymentError::InvalidResponse("Missing refund ID".to_string()))?
            .to_string();

        let payment_id = json
            .get("payment_intent")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let amount_cents = json
            .get("amount")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);

        let currency_str = json
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("myr");

        let currency = CurrencyCode::new(currency_str);
        let amount = Price::new(MinorUnits::new(amount_cents), currency);

        let status_str = json
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending");

        let status = match status_str {
            "succeeded" => RefundStatus::Succeeded,
            "failed" => RefundStatus::Failed,
            "canceled" => RefundStatus::Cancelled,
            _ => RefundStatus::Pending,
        };

        let created_at = json
            .get("created")
            .and_then(serde_json::Value::as_i64)
            .map_or_else(Timestamp::now, Timestamp::from_unix);

        let reason_str = json
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("requested_by_customer");

        let reason = match reason_str {
            "duplicate" => RefundReason::Duplicate,
            "fraudulent" => RefundReason::Fraudulent,
            _ => RefundReason::CustomerRequest,
        };

        Ok(Refund {
            id,
            payment_id,
            amount,
            status,
            created_at,
            reason,
        })
    }
}

/// Payment provider trait for abstraction
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    /// Create a payment
    async fn create_payment(&self, request: &PaymentRequest) -> PaymentResult<PaymentIntent>;

    /// Get payment status
    async fn get_payment(&self, payment_id: &str) -> PaymentResult<PaymentIntent>;

    /// Cancel a payment
    async fn cancel_payment(&self, payment_id: &str) -> PaymentResult<PaymentIntent>;

    /// Create a refund
    async fn create_refund(&self, request: &RefundRequest) -> PaymentResult<Refund>;

    /// Get refund status
    async fn get_refund(&self, refund_id: &str) -> PaymentResult<Refund>;
}

#[async_trait]
impl PaymentProvider for StripeClient {
    async fn create_payment(&self, request: &PaymentRequest) -> PaymentResult<PaymentIntent> {
        self.create_payment(request).await
    }

    async fn get_payment(&self, payment_id: &str) -> PaymentResult<PaymentIntent> {
        self.get_payment(payment_id).await
    }

    async fn cancel_payment(&self, payment_id: &str) -> PaymentResult<PaymentIntent> {
        self.cancel_payment(payment_id).await
    }

    async fn create_refund(&self, request: &RefundRequest) -> PaymentResult<Refund> {
        self.create_refund(request).await
    }

    async fn get_refund(&self, refund_id: &str) -> PaymentResult<Refund> {
        self.get_refund(refund_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stripe_client_creation() {
        let config = PaymentConfig::new("sk_test_123", "pk_test_456");
        let client = StripeClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_stripe_client_validation() {
        let config = PaymentConfig::default();
        let client = StripeClient::new(&config);
        assert!(client.is_err());

        let config = PaymentConfig::new("invalid_key", "pk_test_456");
        let client = StripeClient::new(&config);
        assert!(client.is_err());
    }
}
