//! Amadeus GDS Client
//!
//! Implementation of the GDS provider trait for Amadeus API.

mod auth;
mod client;
mod response;

pub use client::AmadeusClient;
pub(crate) use auth::TokenManager;
#[allow(unused_imports)]
pub(crate) use response::*;
