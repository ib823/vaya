//! Template engine for notifications

use handlebars::Handlebars;
use std::collections::HashMap;

use crate::error::{NotificationError, NotificationResult};

/// Template engine using Handlebars
pub struct TemplateEngine {
    /// Handlebars instance
    hbs: Handlebars<'static>,
}

impl TemplateEngine {
    /// Create new template engine with default templates
    pub fn new() -> Self {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);

        // Register default templates
        Self::register_default_templates(&mut hbs);

        Self { hbs }
    }

    /// Register default email templates
    fn register_default_templates(hbs: &mut Handlebars<'static>) {
        // Booking confirmation email (HTML)
        let _ = hbs.register_template_string(
            "booking_confirmation_html",
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Booking Confirmation</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: #1a56db; color: white; padding: 20px; text-align: center; }
        .content { padding: 20px; background: #f9fafb; }
        .flight-info { background: white; padding: 15px; margin: 10px 0; border-radius: 8px; }
        .price { font-size: 24px; color: #1a56db; font-weight: bold; }
        .footer { text-align: center; padding: 20px; color: #666; font-size: 12px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Booking Confirmed!</h1>
        </div>
        <div class="content">
            <p>Dear {{passenger_name}},</p>
            <p>Your flight has been booked successfully.</p>

            <div class="flight-info">
                <h3>Flight Details</h3>
                <p><strong>Booking Reference:</strong> {{booking_ref}}</p>
                <p><strong>Route:</strong> {{origin}} → {{destination}}</p>
                <p><strong>Date:</strong> {{departure_date}}</p>
                <p><strong>Flight:</strong> {{flight_number}}</p>
            </div>

            <p class="price">Total: {{currency}} {{total_amount}}</p>

            <p>Your e-ticket will be sent separately.</p>
        </div>
        <div class="footer">
            <p>VAYA Flights - Your journey starts here</p>
            <p>Need help? Contact us at support@vaya.my</p>
        </div>
    </div>
</body>
</html>"#,
        );

        // Booking confirmation email (text)
        let _ = hbs.register_template_string(
            "booking_confirmation_text",
            r#"BOOKING CONFIRMED

Dear {{passenger_name}},

Your flight has been booked successfully.

FLIGHT DETAILS
--------------
Booking Reference: {{booking_ref}}
Route: {{origin}} → {{destination}}
Date: {{departure_date}}
Flight: {{flight_number}}

Total: {{currency}} {{total_amount}}

Your e-ticket will be sent separately.

---
VAYA Flights - Your journey starts here
Need help? Contact us at support@vaya.my"#,
        );

        // Payment confirmation
        let _ = hbs.register_template_string(
            "payment_confirmation_html",
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Payment Confirmed</title>
</head>
<body>
    <h1>Payment Received</h1>
    <p>Dear {{passenger_name}},</p>
    <p>We have received your payment of {{currency}} {{amount}} for booking {{booking_ref}}.</p>
    <p>Your ticket will be issued shortly.</p>
</body>
</html>"#,
        );

        // Flight reminder
        let _ = hbs.register_template_string(
            "flight_reminder_text",
            "VAYA Flight Reminder: Your flight {{flight_number}} from {{origin}} to {{destination}} departs in {{hours_until}} hours. Check-in at {{airport_terminal}}.",
        );

        // Price alert
        let _ = hbs.register_template_string(
            "price_alert_html",
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Price Alert</title>
</head>
<body>
    <h1>Price Drop Alert!</h1>
    <p>Great news! The price for {{origin}} → {{destination}} has dropped to {{currency}} {{new_price}}!</p>
    <p>Previous price: {{currency}} {{old_price}}</p>
    <p>You save: {{currency}} {{savings}}</p>
    <a href="{{booking_url}}">Book Now</a>
</body>
</html>"#,
        );

        // Welcome email
        let _ = hbs.register_template_string(
            "welcome_html",
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Welcome to VAYA</title>
</head>
<body>
    <h1>Welcome to VAYA!</h1>
    <p>Hi {{name}},</p>
    <p>Welcome to VAYA Flights! We're excited to have you on board.</p>
    <p>Start exploring amazing flight deals today.</p>
</body>
</html>"#,
        );

        // Password reset
        let _ = hbs.register_template_string(
            "password_reset_html",
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Reset Your Password</title>
</head>
<body>
    <h1>Reset Your Password</h1>
    <p>Hi {{name}},</p>
    <p>Click the link below to reset your password:</p>
    <p><a href="{{reset_link}}">Reset Password</a></p>
    <p>This link expires in 1 hour.</p>
    <p>If you didn't request this, please ignore this email.</p>
</body>
</html>"#,
        );
    }

    /// Register a custom template
    pub fn register(&mut self, name: &str, template: &str) -> NotificationResult<()> {
        self.hbs
            .register_template_string(name, template)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register template: {e}")))
    }

    /// Render a template with context
    pub fn render(
        &self,
        template_name: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> NotificationResult<String> {
        if !self.hbs.has_template(template_name) {
            return Err(NotificationError::TemplateNotFound(template_name.to_string()));
        }

        self.hbs
            .render(template_name, context)
            .map_err(NotificationError::from)
    }

    /// Check if template exists
    #[must_use]
    pub fn has_template(&self, name: &str) -> bool {
        self.hbs.has_template(name)
    }

    /// Get list of registered templates
    #[must_use]
    pub fn list_templates(&self) -> Vec<String> {
        self.hbs
            .get_templates()
            .keys()
            .cloned()
            .collect()
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.has_template("booking_confirmation_html"));
        assert!(engine.has_template("welcome_html"));
    }

    #[test]
    fn test_render_template() {
        let engine = TemplateEngine::new();

        let mut context = HashMap::new();
        context.insert("passenger_name".to_string(), serde_json::json!("John Doe"));
        context.insert("booking_ref".to_string(), serde_json::json!("VAY123"));
        context.insert("origin".to_string(), serde_json::json!("KUL"));
        context.insert("destination".to_string(), serde_json::json!("NRT"));
        context.insert("departure_date".to_string(), serde_json::json!("2025-02-15"));
        context.insert("flight_number".to_string(), serde_json::json!("MH88"));
        context.insert("currency".to_string(), serde_json::json!("MYR"));
        context.insert("total_amount".to_string(), serde_json::json!("1,500.00"));

        let result = engine.render("booking_confirmation_html", &context);
        assert!(result.is_ok());

        let html = result.expect("Should render");
        assert!(html.contains("John Doe"));
        assert!(html.contains("VAY123"));
        assert!(html.contains("KUL"));
    }

    #[test]
    fn test_template_not_found() {
        let engine = TemplateEngine::new();
        let context = HashMap::new();

        let result = engine.render("non_existent_template", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_custom_template() {
        let mut engine = TemplateEngine::new();

        let result = engine.register("custom", "Hello, {{name}}!");
        assert!(result.is_ok());
        assert!(engine.has_template("custom"));

        let mut context = HashMap::new();
        context.insert("name".to_string(), serde_json::json!("World"));

        let rendered = engine.render("custom", &context);
        assert_eq!(rendered.expect("Should render"), "Hello, World!");
    }

    #[test]
    fn test_list_templates() {
        let engine = TemplateEngine::new();
        let templates = engine.list_templates();

        assert!(templates.contains(&"booking_confirmation_html".to_string()));
        assert!(templates.contains(&"welcome_html".to_string()));
    }
}
