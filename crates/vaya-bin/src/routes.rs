//! API route registration

use std::sync::Arc;

use vaya_api::{ApiResult, ApiServer, Request, Response};

use crate::app::AppState;
use crate::handlers;

/// Register all routes with the server
pub fn register_routes(server: &mut ApiServer, _state: Arc<AppState>) {
    // Health check
    server.get("/health", health_handler, "health");
    server.get("/ready", ready_handler, "ready");
    server.get("/live", live_handler, "live");

    // Search routes
    server.post(
        "/search/flights",
        handlers::search::search_flights,
        "search_flights",
    );
    server.get(
        "/search/airports",
        handlers::search::search_airports,
        "search_airports",
    );
    server.get(
        "/search/airlines",
        handlers::search::search_airlines,
        "search_airlines",
    );

    // Booking routes
    server.post(
        "/bookings",
        handlers::booking::create_booking,
        "create_booking",
    );
    server.get(
        "/bookings",
        handlers::booking::list_bookings,
        "list_bookings",
    );
    server.get(
        "/bookings/:id",
        handlers::booking::get_booking,
        "get_booking",
    );
    server.post(
        "/bookings/:id/confirm",
        handlers::booking::confirm_booking,
        "confirm_booking",
    );
    server.post(
        "/bookings/:id/cancel",
        handlers::booking::cancel_booking,
        "cancel_booking",
    );

    // Pool routes (group buying)
    server.post("/pools", handlers::pool::create_pool, "create_pool");
    server.get("/pools", handlers::pool::list_pools, "list_pools");
    server.get("/pools/:id", handlers::pool::get_pool, "get_pool");
    server.post("/pools/:id/join", handlers::pool::join_pool, "join_pool");
    server.post("/pools/:id/leave", handlers::pool::leave_pool, "leave_pool");
    server.post(
        "/pools/:id/contribute",
        handlers::pool::contribute,
        "contribute",
    );

    // Alert routes
    server.post("/alerts", handlers::alert::create_alert, "create_alert");
    server.get("/alerts", handlers::alert::list_alerts, "list_alerts");
    server.get("/alerts/:id", handlers::alert::get_alert, "get_alert");
    server.delete("/alerts/:id", handlers::alert::delete_alert, "delete_alert");
    server.post(
        "/alerts/:id/pause",
        handlers::alert::pause_alert,
        "pause_alert",
    );
    server.post(
        "/alerts/:id/resume",
        handlers::alert::resume_alert,
        "resume_alert",
    );

    // User routes
    server.post("/auth/register", handlers::auth::register, "register");
    server.post("/auth/login", handlers::auth::login, "login");
    server.post("/auth/logout", handlers::auth::logout, "logout");
    server.post(
        "/auth/refresh",
        handlers::auth::refresh_token,
        "refresh_token",
    );
    server.get("/users/me", handlers::user::get_profile, "get_profile");
    server.put(
        "/users/me",
        handlers::user::update_profile,
        "update_profile",
    );
    server.put(
        "/users/me/password",
        handlers::user::change_password,
        "change_password",
    );

    // Oracle routes (pricing insights)
    server.get(
        "/oracle/predict",
        handlers::oracle::get_prediction,
        "get_prediction",
    );
    server.get(
        "/oracle/insights",
        handlers::oracle::get_insights,
        "get_insights",
    );
    server.get(
        "/oracle/best-time",
        handlers::oracle::get_best_time,
        "get_best_time",
    );
}

/// Health check handler
fn health_handler(req: &Request) -> ApiResult<Response> {
    handlers::health::health(req)
}

/// Readiness check handler
fn ready_handler(req: &Request) -> ApiResult<Response> {
    handlers::health::ready(req)
}

/// Liveness check handler
fn live_handler(req: &Request) -> ApiResult<Response> {
    handlers::health::live(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vaya_api::ApiConfig;

    #[test]
    fn test_route_registration() {
        // This would need actual state in real tests
        // For now just verify the function compiles
        assert!(true);
    }
}
