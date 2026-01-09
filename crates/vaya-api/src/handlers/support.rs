//! Support handlers (4 handlers)
//!
//! Endpoints for customer support ticket management:
//! - POST /support/tickets - Create support ticket
//! - GET /support/tickets - List user's tickets
//! - GET /support/tickets/{id} - Get ticket details
//! - POST /support/tickets/{id}/reply - Reply to ticket

use crate::{ApiError, ApiResult, JsonSerialize, Request, Response};

/// Ticket priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Urgent,
}

impl TicketPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }
}

/// Ticket status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketStatus {
    Open,
    InProgress,
    WaitingCustomer,
    Resolved,
    Closed,
}

impl TicketStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::WaitingCustomer => "waiting_customer",
            Self::Resolved => "resolved",
            Self::Closed => "closed",
        }
    }
}

/// Ticket category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketCategory {
    Booking,
    Payment,
    Refund,
    TechnicalIssue,
    AccountIssue,
    PoolInquiry,
    General,
}

impl TicketCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Booking => "booking",
            Self::Payment => "payment",
            Self::Refund => "refund",
            Self::TechnicalIssue => "technical_issue",
            Self::AccountIssue => "account_issue",
            Self::PoolInquiry => "pool_inquiry",
            Self::General => "general",
        }
    }
}

/// Support ticket
pub struct SupportTicket {
    pub id: String,
    pub user_id: String,
    pub subject: String,
    pub description: String,
    pub category: TicketCategory,
    pub priority: TicketPriority,
    pub status: TicketStatus,
    pub booking_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl JsonSerialize for SupportTicket {
    fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","user_id":"{}","subject":"{}","description":"{}","category":"{}","priority":"{}","status":"{}","booking_id":{},"created_at":"{}","updated_at":"{}"}}"#,
            self.id,
            self.user_id,
            escape_json(&self.subject),
            escape_json(&self.description),
            self.category.as_str(),
            self.priority.as_str(),
            self.status.as_str(),
            self.booking_id
                .as_ref()
                .map(|b| format!("\"{}\"", b))
                .unwrap_or_else(|| "null".to_string()),
            self.created_at,
            self.updated_at
        )
    }
}

/// Ticket reply
pub struct TicketReply {
    pub id: String,
    pub ticket_id: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub message: String,
    pub is_internal: bool,
    pub created_at: String,
}

impl JsonSerialize for TicketReply {
    fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","ticket_id":"{}","user_id":{},"agent_id":{},"message":"{}","is_internal":{},"created_at":"{}"}}"#,
            self.id,
            self.ticket_id,
            self.user_id
                .as_ref()
                .map(|u| format!("\"{}\"", u))
                .unwrap_or_else(|| "null".to_string()),
            self.agent_id
                .as_ref()
                .map(|a| format!("\"{}\"", a))
                .unwrap_or_else(|| "null".to_string()),
            escape_json(&self.message),
            self.is_internal,
            self.created_at
        )
    }
}

/// Escape special JSON characters
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// POST /support/tickets - Create support ticket
pub fn create_ticket_handler(req: &Request) -> ApiResult<Response> {
    let user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;

    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }

    // Parse request body (simplified - in production would use proper JSON parsing)
    let body = String::from_utf8_lossy(&req.body);

    // Validate required fields
    if !body.contains("\"subject\"") || !body.contains("\"description\"") {
        return Err(ApiError::bad_request(
            "Missing required fields: subject, description",
        ));
    }

    // Generate ticket ID
    let ticket_id = format!("TKT-{}", generate_id());
    let now = current_timestamp();

    let ticket = SupportTicket {
        id: ticket_id.clone(),
        user_id: user_id.clone(),
        subject: extract_field(&body, "subject").unwrap_or_default(),
        description: extract_field(&body, "description").unwrap_or_default(),
        category: TicketCategory::General,
        priority: TicketPriority::Medium,
        status: TicketStatus::Open,
        booking_id: extract_field(&body, "booking_id"),
        created_at: now.clone(),
        updated_at: now,
    };

    let mut response = Response::created();
    response.set_json_body(&ticket);
    Ok(response)
}

/// GET /support/tickets - List user's tickets
pub fn list_tickets_handler(req: &Request) -> ApiResult<Response> {
    let user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;

    // Get query params for filtering
    let status = req.query("status");
    let page: u32 = req.query("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page: u32 = req
        .query("per_page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(20)
        .min(100);

    // In production, would query database here
    let _ = (user_id, status, page, per_page);

    let response_body = format!(
        r#"{{"tickets":[],"total":0,"page":{},"per_page":{},"total_pages":0}}"#,
        page, per_page
    );

    Ok(Response::ok().with_body(response_body.into_bytes()))
}

/// GET /support/tickets/{id} - Get ticket details
pub fn get_ticket_handler(req: &Request) -> ApiResult<Response> {
    let ticket_id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing ticket ID"))?;
    let user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;

    // In production, would query database and verify ownership
    let _ = user_id;

    let now = current_timestamp();
    let ticket = SupportTicket {
        id: ticket_id.clone(),
        user_id: user_id.clone(),
        subject: "Sample ticket".to_string(),
        description: "Sample description".to_string(),
        category: TicketCategory::General,
        priority: TicketPriority::Medium,
        status: TicketStatus::Open,
        booking_id: None,
        created_at: now.clone(),
        updated_at: now,
    };

    // Include replies
    let response_body = format!(r#"{{"ticket":{},"replies":[]}}"#, ticket.to_json());

    Ok(Response::ok().with_body(response_body.into_bytes()))
}

/// POST /support/tickets/{id}/reply - Reply to ticket
pub fn reply_to_ticket_handler(req: &Request) -> ApiResult<Response> {
    let ticket_id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing ticket ID"))?;
    let user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;

    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }

    let body = String::from_utf8_lossy(&req.body);

    // Validate message
    let message = extract_field(&body, "message")
        .ok_or(ApiError::bad_request("Missing required field: message"))?;

    if message.is_empty() {
        return Err(ApiError::bad_request("Message cannot be empty"));
    }

    let reply_id = format!("RPL-{}", generate_id());
    let now = current_timestamp();

    let reply = TicketReply {
        id: reply_id,
        ticket_id: ticket_id.clone(),
        user_id: Some(user_id.clone()),
        agent_id: None,
        message,
        is_internal: false,
        created_at: now,
    };

    let mut response = Response::created();
    response.set_json_body(&reply);
    Ok(response)
}

/// Generate a simple ID (in production would use proper UUID)
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}{:04x}", now.as_secs(), now.subsec_nanos() % 0xFFFF)
}

/// Get current timestamp as ISO 8601 string
fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Simple conversion - in production would use proper date library
    let days = now / 86400;
    let time_of_day = now % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let mut y = 1970i32;
    let mut remaining_days = days;
    loop {
        let days_in_year = if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let is_leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
    let days_in_months: [i64; 12] = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 1u32;
    for &days_in_month in &days_in_months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        m += 1;
    }
    let d = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

/// Extract a field value from JSON string (simplified parser)
fn extract_field(json: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\":", field);
    let start = json.find(&pattern)?;
    let value_start = start + pattern.len();
    let rest = &json[value_start..];

    // Skip whitespace
    let rest = rest.trim_start();

    if let Some(rest) = rest.strip_prefix('"') {
        // String value
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    } else if rest.starts_with("null") {
        None
    } else {
        // Number or boolean
        let end = rest.find([',', '}', ']']).unwrap_or(rest.len());
        Some(rest[..end].trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ticket_handler() {
        let mut req = Request::new("POST", "/support/tickets");
        req.user_id = Some("user_123".into());
        req.body =
            br#"{"subject":"Help needed","description":"I need help with my booking"}"#.to_vec();

        let resp = create_ticket_handler(&req).unwrap();
        assert_eq!(resp.status, 201);
    }

    #[test]
    fn test_create_ticket_missing_body() {
        let mut req = Request::new("POST", "/support/tickets");
        req.user_id = Some("user_123".into());

        let result = create_ticket_handler(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_tickets_handler() {
        let mut req = Request::new("GET", "/support/tickets");
        req.user_id = Some("user_123".into());

        let resp = list_tickets_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_get_ticket_handler() {
        let mut req = Request::new("GET", "/support/tickets/TKT-123");
        req.user_id = Some("user_123".into());
        req.path_params.insert("id".into(), "TKT-123".into());

        let resp = get_ticket_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_reply_to_ticket_handler() {
        let mut req = Request::new("POST", "/support/tickets/TKT-123/reply");
        req.user_id = Some("user_123".into());
        req.path_params.insert("id".into(), "TKT-123".into());
        req.body = br#"{"message":"Thank you for your help!"}"#.to_vec();

        let resp = reply_to_ticket_handler(&req).unwrap();
        assert_eq!(resp.status, 201);
    }

    #[test]
    fn test_extract_field() {
        let json = r#"{"subject":"Test","description":"Testing"}"#;
        assert_eq!(extract_field(json, "subject"), Some("Test".to_string()));
        assert_eq!(
            extract_field(json, "description"),
            Some("Testing".to_string())
        );
        assert_eq!(extract_field(json, "missing"), None);
    }

    #[test]
    fn test_ticket_priority_as_str() {
        assert_eq!(TicketPriority::Low.as_str(), "low");
        assert_eq!(TicketPriority::Urgent.as_str(), "urgent");
    }

    #[test]
    fn test_ticket_status_as_str() {
        assert_eq!(TicketStatus::Open.as_str(), "open");
        assert_eq!(TicketStatus::Resolved.as_str(), "resolved");
    }
}
