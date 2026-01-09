//! VAYA Screens
//!
//! Full page screen components for each route in the application.

pub mod home;
pub mod not_found;
pub mod oracle_loading;
pub mod oracle_result;
pub mod booking;
pub mod payment;

pub use home::Home;
pub use not_found::NotFound;
pub use oracle_loading::OracleLoading;
pub use oracle_result::OracleResult;
pub use booking::*;
pub use payment::*;
