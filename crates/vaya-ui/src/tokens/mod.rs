//! VAYA Design System Tokens
//!
//! These tokens define the visual language of VAYA. All values are derived from
//! the Design System v2 specification (`docs/docs/VAYA_Design_System_v2.html`).
//!
//! # Usage
//!
//! Use these constants when setting inline styles or generating CSS:
//!
//! ```rust
//! use vaya_ui::tokens::colors;
//!
//! let style = format!("background: {}; color: {}", colors::MINT_500, colors::N0);
//! ```

pub mod animations;
pub mod colors;
pub mod spacing;
pub mod typography;

// Re-export commonly used tokens
pub use animations::*;
pub use colors::*;
pub use spacing::*;
pub use typography::*;
