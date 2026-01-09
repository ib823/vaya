//! Booking handlers

use vaya_api::{ApiError, ApiResult, JsonSerialize, Request, Response};

/// Create a new booking
pub fn create_booking(req: &Request) -> ApiResult<Response> {
    // Require authentication
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let _body = req
        .body_string()
        .ok_or(ApiError::BadRequest("Missing request body".into()))?;

    // TODO: Parse and create booking
    let booking = BookingResponse {
        id: generate_booking_id(),
        pnr: generate_pnr(),
        status: "pending".into(),
        created_at: current_timestamp(),
    };

    let mut response = Response::created();
    response.set_json_body(&booking);
    Ok(response)
}

/// List user's bookings
pub fn list_bookings(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let _page = req
        .query("page")
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(1);

    let _limit = req
        .query("limit")
        .and_then(|l| l.parse::<u32>().ok())
        .unwrap_or(20)
        .min(100);

    // TODO: Fetch bookings from database
    let response = BookingsListResponse {
        bookings: vec![],
        total: 0,
        page: 1,
        page_size: 20,
    };

    let mut resp = Response::ok();
    resp.set_json_body(&response);
    Ok(resp)
}

/// Get a specific booking
pub fn get_booking(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let booking_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing booking ID".into()))?;

    // TODO: Fetch booking from database
    // For now, return not found as placeholder
    Err(ApiError::NotFound(format!(
        "Booking {} not found",
        booking_id
    )))
}

/// Confirm a booking
pub fn confirm_booking(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let booking_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing booking ID".into()))?;

    // TODO: Confirm booking
    Err(ApiError::NotFound(format!(
        "Booking {} not found",
        booking_id
    )))
}

/// Cancel a booking
pub fn cancel_booking(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let booking_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing booking ID".into()))?;

    // TODO: Cancel booking
    Err(ApiError::NotFound(format!(
        "Booking {} not found",
        booking_id
    )))
}

/// Booking response
#[derive(Debug, Clone)]
pub struct BookingResponse {
    pub id: String,
    pub pnr: String,
    pub status: String,
    pub created_at: String,
}

impl JsonSerialize for BookingResponse {
    fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","pnr":"{}","status":"{}","created_at":"{}"}}"#,
            self.id, self.pnr, self.status, self.created_at
        )
    }
}

/// Bookings list response
#[derive(Debug, Clone)]
pub struct BookingsListResponse {
    pub bookings: Vec<BookingResponse>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

impl JsonSerialize for BookingsListResponse {
    fn to_json(&self) -> String {
        let bookings: Vec<String> = self.bookings.iter().map(|b| b.to_json()).collect();
        format!(
            r#"{{"bookings":[{}],"total":{},"page":{},"page_size":{}}}"#,
            bookings.join(","),
            self.total,
            self.page,
            self.page_size
        )
    }
}

/// Generate booking ID
fn generate_booking_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("bk-{:x}", timestamp)
}

/// Generate PNR (6 character alphanumeric)
fn generate_pnr() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    // Simple PNR generation
    let chars = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let chars: Vec<char> = chars.chars().collect();
    let mut pnr = String::with_capacity(6);
    let mut val = timestamp;

    for _ in 0..6 {
        let idx = (val % chars.len() as u64) as usize;
        pnr.push(chars[idx]);
        val /= chars.len() as u64;
    }

    pnr
}

/// Get current timestamp as ISO string
fn current_timestamp() -> String {
    use time::OffsetDateTime;
    let now = OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_booking_requires_auth() {
        let req = Request::new("POST", "/bookings");
        let result = create_booking(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    #[test]
    fn test_list_bookings_requires_auth() {
        let req = Request::new("GET", "/bookings");
        let result = list_bookings(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    #[test]
    fn test_booking_response_json() {
        let booking = BookingResponse {
            id: "bk-123".into(),
            pnr: "ABC123".into(),
            status: "confirmed".into(),
            created_at: "2026-01-15T10:00:00Z".into(),
        };
        let json = booking.to_json();
        assert!(json.contains(r#""pnr":"ABC123""#));
        assert!(json.contains(r#""status":"confirmed""#));
    }

    #[test]
    fn test_generate_pnr() {
        let pnr = generate_pnr();
        assert_eq!(pnr.len(), 6);
        // PNR should only contain allowed characters
        let allowed = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        for c in pnr.chars() {
            assert!(allowed.contains(c));
        }
    }
}
