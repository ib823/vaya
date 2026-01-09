//! Notification types

use vaya_common::Timestamp;
use std::collections::HashMap;

/// Notification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationStatus {
    /// Queued for sending
    Queued,
    /// Sent to provider
    Sent,
    /// Delivered to recipient
    Delivered,
    /// Opened (email only)
    Opened,
    /// Clicked (email only)
    Clicked,
    /// Bounced
    Bounced,
    /// Failed
    Failed,
    /// Spam complaint
    SpamComplaint,
}

impl NotificationStatus {
    /// Is this a terminal status?
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Delivered | Self::Bounced | Self::Failed | Self::SpamComplaint
        )
    }

    /// Is this a successful delivery?
    #[must_use]
    pub const fn is_successful(&self) -> bool {
        matches!(self, Self::Delivered | Self::Opened | Self::Clicked)
    }

    /// Display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Sent => "Sent",
            Self::Delivered => "Delivered",
            Self::Opened => "Opened",
            Self::Clicked => "Clicked",
            Self::Bounced => "Bounced",
            Self::Failed => "Failed",
            Self::SpamComplaint => "Spam Complaint",
        }
    }
}

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationType {
    /// Booking confirmation
    BookingConfirmation,
    /// Payment confirmation
    PaymentConfirmation,
    /// E-ticket
    ETicket,
    /// Flight reminder
    FlightReminder,
    /// Flight change
    FlightChange,
    /// Flight cancellation
    FlightCancellation,
    /// Price alert
    PriceAlert,
    /// Marketing
    Marketing,
    /// Password reset
    PasswordReset,
    /// Welcome email
    Welcome,
}

impl NotificationType {
    /// Get template name for this type
    #[must_use]
    pub const fn template_name(&self) -> &'static str {
        match self {
            Self::BookingConfirmation => "booking_confirmation",
            Self::PaymentConfirmation => "payment_confirmation",
            Self::ETicket => "e_ticket",
            Self::FlightReminder => "flight_reminder",
            Self::FlightChange => "flight_change",
            Self::FlightCancellation => "flight_cancellation",
            Self::PriceAlert => "price_alert",
            Self::Marketing => "marketing",
            Self::PasswordReset => "password_reset",
            Self::Welcome => "welcome",
        }
    }

    /// Is this a transactional email?
    #[must_use]
    pub const fn is_transactional(&self) -> bool {
        !matches!(self, Self::Marketing | Self::PriceAlert)
    }

    /// Default subject line
    #[must_use]
    pub const fn default_subject(&self) -> &'static str {
        match self {
            Self::BookingConfirmation => "Your Booking Confirmation",
            Self::PaymentConfirmation => "Payment Confirmed",
            Self::ETicket => "Your E-Ticket",
            Self::FlightReminder => "Flight Reminder",
            Self::FlightChange => "Flight Schedule Change",
            Self::FlightCancellation => "Flight Cancellation Notice",
            Self::PriceAlert => "Price Drop Alert",
            Self::Marketing => "Special Offers from VAYA",
            Self::PasswordReset => "Reset Your Password",
            Self::Welcome => "Welcome to VAYA",
        }
    }
}

/// Email request
#[derive(Debug, Clone)]
pub struct EmailRequest {
    /// Recipient email
    pub to_email: String,
    /// Recipient name
    pub to_name: Option<String>,
    /// Subject line
    pub subject: String,
    /// Plain text body (optional)
    pub text_body: Option<String>,
    /// HTML body (optional)
    pub html_body: Option<String>,
    /// Template name
    pub template: Option<String>,
    /// Template context data
    pub context: HashMap<String, serde_json::Value>,
    /// Notification type
    pub notification_type: NotificationType,
    /// Attachments
    pub attachments: Vec<EmailAttachment>,
    /// Reply-to address
    pub reply_to: Option<String>,
    /// Custom headers
    pub headers: HashMap<String, String>,
    /// Tags for analytics
    pub tags: Vec<String>,
}

impl EmailRequest {
    /// Create new email request
    #[must_use]
    pub fn new(to_email: impl Into<String>, subject: impl Into<String>) -> Self {
        Self {
            to_email: to_email.into(),
            to_name: None,
            subject: subject.into(),
            text_body: None,
            html_body: None,
            template: None,
            context: HashMap::new(),
            notification_type: NotificationType::BookingConfirmation,
            attachments: Vec::new(),
            reply_to: None,
            headers: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Create from notification type
    #[must_use]
    pub fn from_type(to_email: impl Into<String>, notification_type: NotificationType) -> Self {
        Self {
            to_email: to_email.into(),
            to_name: None,
            subject: notification_type.default_subject().to_string(),
            text_body: None,
            html_body: None,
            template: Some(notification_type.template_name().to_string()),
            context: HashMap::new(),
            notification_type,
            attachments: Vec::new(),
            reply_to: None,
            headers: HashMap::new(),
            tags: vec![notification_type.template_name().to_string()],
        }
    }

    /// Set recipient name
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.to_name = Some(name.into());
        self
    }

    /// Set text body
    #[must_use]
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text_body = Some(text.into());
        self
    }

    /// Set HTML body
    #[must_use]
    pub fn with_html(mut self, html: impl Into<String>) -> Self {
        self.html_body = Some(html.into());
        self
    }

    /// Set template
    #[must_use]
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Add context variable
    #[must_use]
    pub fn with_context(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.context.insert(key.into(), v);
        }
        self
    }

    /// Add attachment
    #[must_use]
    pub fn with_attachment(mut self, attachment: EmailAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Add tag
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Validate email request
    pub fn validate(&self) -> crate::NotificationResult<()> {
        if self.to_email.is_empty() || !self.to_email.contains('@') {
            return Err(crate::NotificationError::InvalidRecipient(
                "Invalid email address".to_string(),
            ));
        }
        if self.subject.is_empty() {
            return Err(crate::NotificationError::Configuration(
                "Subject is required".to_string(),
            ));
        }
        if self.text_body.is_none() && self.html_body.is_none() && self.template.is_none() {
            return Err(crate::NotificationError::Configuration(
                "Email body or template is required".to_string(),
            ));
        }
        Ok(())
    }
}

/// Email attachment
#[derive(Debug, Clone)]
pub struct EmailAttachment {
    /// Filename
    pub filename: String,
    /// Content type (MIME)
    pub content_type: String,
    /// Base64 encoded content
    pub content: String,
    /// Disposition (attachment or inline)
    pub disposition: AttachmentDisposition,
    /// Content ID (for inline)
    pub content_id: Option<String>,
}

/// Attachment disposition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentDisposition {
    /// Regular attachment
    Attachment,
    /// Inline (for embedded images)
    Inline,
}

/// Email delivery result
#[derive(Debug, Clone)]
pub struct EmailResult {
    /// Message ID from provider
    pub message_id: String,
    /// Status
    pub status: NotificationStatus,
    /// Sent timestamp
    pub sent_at: Timestamp,
}

/// SMS request
#[derive(Debug, Clone)]
pub struct SmsRequest {
    /// Recipient phone number (E.164 format)
    pub to_phone: String,
    /// Message body
    pub message: String,
    /// Template name (optional)
    pub template: Option<String>,
    /// Template context
    pub context: HashMap<String, serde_json::Value>,
    /// Notification type
    pub notification_type: NotificationType,
}

impl SmsRequest {
    /// Create new SMS request
    #[must_use]
    pub fn new(to_phone: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            to_phone: to_phone.into(),
            message: message.into(),
            template: None,
            context: HashMap::new(),
            notification_type: NotificationType::FlightReminder,
        }
    }

    /// Set template
    #[must_use]
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Add context
    #[must_use]
    pub fn with_context(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.context.insert(key.into(), v);
        }
        self
    }

    /// Validate SMS request
    pub fn validate(&self) -> crate::NotificationResult<()> {
        if self.to_phone.is_empty() {
            return Err(crate::NotificationError::InvalidPhoneNumber(
                "Phone number is required".to_string(),
            ));
        }
        // Basic E.164 format check
        if !self.to_phone.starts_with('+') {
            return Err(crate::NotificationError::InvalidPhoneNumber(
                "Phone number must be in E.164 format (e.g., +60123456789)".to_string(),
            ));
        }
        if self.message.is_empty() && self.template.is_none() {
            return Err(crate::NotificationError::Configuration(
                "Message or template is required".to_string(),
            ));
        }
        if self.message.len() > 1600 {
            return Err(crate::NotificationError::Configuration(
                "Message exceeds maximum length (1600 characters)".to_string(),
            ));
        }
        Ok(())
    }
}

/// SMS delivery result
#[derive(Debug, Clone)]
pub struct SmsResult {
    /// Message SID from provider
    pub message_sid: String,
    /// Status
    pub status: NotificationStatus,
    /// Sent timestamp
    pub sent_at: Timestamp,
    /// Number of segments
    pub segments: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_status() {
        assert!(NotificationStatus::Delivered.is_terminal());
        assert!(NotificationStatus::Bounced.is_terminal());
        assert!(!NotificationStatus::Sent.is_terminal());

        assert!(NotificationStatus::Delivered.is_successful());
        assert!(!NotificationStatus::Failed.is_successful());
    }

    #[test]
    fn test_notification_type() {
        assert_eq!(
            NotificationType::BookingConfirmation.template_name(),
            "booking_confirmation"
        );
        assert!(NotificationType::BookingConfirmation.is_transactional());
        assert!(!NotificationType::Marketing.is_transactional());
    }

    #[test]
    fn test_email_request() {
        let email = EmailRequest::new("user@example.com", "Test Subject")
            .with_name("John Doe")
            .with_text("Hello, world!")
            .with_tag("test");

        assert_eq!(email.to_email, "user@example.com");
        assert_eq!(email.to_name, Some("John Doe".to_string()));
        assert!(email.validate().is_ok());
    }

    #[test]
    fn test_email_request_validation() {
        let email = EmailRequest::new("invalid-email", "Test");
        assert!(email.validate().is_err());

        let email = EmailRequest::new("valid@email.com", "Test");
        assert!(email.validate().is_err()); // No body

        let email = EmailRequest::new("valid@email.com", "Test")
            .with_template("test_template");
        assert!(email.validate().is_ok());
    }

    #[test]
    fn test_sms_request() {
        let sms = SmsRequest::new("+60123456789", "Your flight is in 2 hours");
        assert!(sms.validate().is_ok());

        let sms = SmsRequest::new("123456789", "Test");
        assert!(sms.validate().is_err()); // Not E.164 format
    }
}
