//! VAYA Color System - Exact hex values from Design System v2
//!
//! All colors are specified as CSS-compatible strings (hex or rgba).
//! These MUST match the values in `VAYA_Design_System_v2.html` exactly.

// ============================================================================
// PRIMARY BRAND COLORS
// ============================================================================

/// Primary mint green - The signature VAYA color
pub const MINT_500: &str = "#00F5A0";
/// Primary mint hover state
pub const MINT_600: &str = "#00CC85";
/// Primary mint dim (for backgrounds)
pub const MINT_DIM: &str = "rgba(0, 245, 160, 0.12)";
/// Primary mint muted (for subtle accents)
pub const MINT_MUTED: &str = "rgba(0, 245, 160, 0.24)";

/// Secondary cyan - Accent and highlights
pub const CYAN_500: &str = "#00D9FF";
/// Secondary cyan hover state
pub const CYAN_600: &str = "#00B8D9";
/// Secondary cyan dim (for backgrounds)
pub const CYAN_DIM: &str = "rgba(0, 217, 255, 0.12)";

// ============================================================================
// NEUTRAL SCALE (N0-N1000)
// ============================================================================

/// Void black - Pure black
pub const N0: &str = "#000000";
/// Background - Primary app background
pub const N50: &str = "#0A0A0B";
/// Card background - Elevated surfaces
pub const N100: &str = "#111113";
/// Elevated - Higher elevation surfaces
pub const N150: &str = "#161618";
/// Border default - Default border color
pub const N200: &str = "#1A1A1D";
/// Border hover - Border on hover
pub const N300: &str = "#232326";
/// Disabled bg - Disabled element backgrounds
pub const N400: &str = "#2D2D31";
/// Placeholder - Placeholder text
pub const N500: &str = "#3D3D42";
/// Secondary text - Less prominent text
pub const N600: &str = "#5C5C63";
/// Tertiary text - Least prominent text
pub const N700: &str = "#7A7A82";
/// Body text - Standard body text
pub const N800: &str = "#9999A1";
/// Primary text light - Light mode primary
pub const N900: &str = "#CCCCCC";
/// Primary text - Maximum contrast text
pub const N1000: &str = "#FFFFFF";

// ============================================================================
// SEMANTIC COLORS
// ============================================================================

/// Warning - Attention required
pub const WARNING: &str = "#FFB800";
/// Warning dim (for backgrounds)
pub const WARNING_DIM: &str = "rgba(255, 184, 0, 0.12)";

/// Error - Destructive/error states
pub const ERROR: &str = "#FF4757";
/// Error dim (for backgrounds)
pub const ERROR_DIM: &str = "rgba(255, 71, 87, 0.12)";

/// Success - Positive outcomes (same as MINT_500)
pub const SUCCESS: &str = "#00F5A0";
/// Success dim (for backgrounds)
pub const SUCCESS_DIM: &str = "rgba(0, 245, 160, 0.12)";

/// Purple - Special/premium features
pub const PURPLE: &str = "#A855F7";
/// Purple dim (for backgrounds)
pub const PURPLE_DIM: &str = "rgba(168, 85, 247, 0.12)";

/// Blue - Info/links
pub const BLUE: &str = "#3B82F6";
/// Blue dim (for backgrounds)
pub const BLUE_DIM: &str = "rgba(59, 130, 246, 0.12)";

// ============================================================================
// GLOW EFFECTS
// ============================================================================

/// Small glow - Subtle emphasis
pub const GLOW_SM: &str = "0 0 16px rgba(0, 245, 160, 0.3)";
/// Medium glow - Standard emphasis
pub const GLOW_MD: &str = "0 0 24px rgba(0, 245, 160, 0.4)";
/// Large glow - Strong emphasis
pub const GLOW_LG: &str = "0 0 40px rgba(0, 245, 160, 0.5)";

/// Cyan glow - Secondary emphasis
pub const GLOW_CYAN_SM: &str = "0 0 16px rgba(0, 217, 255, 0.3)";
pub const GLOW_CYAN_MD: &str = "0 0 24px rgba(0, 217, 255, 0.4)";

// ============================================================================
// GRADIENTS
// ============================================================================

/// Primary gradient - Mint to Cyan
pub const GRADIENT_PRIMARY: &str = "linear-gradient(135deg, #00F5A0 0%, #00D9FF 100%)";
/// Dark gradient - For card backgrounds
pub const GRADIENT_DARK: &str = "linear-gradient(180deg, #111113 0%, #0A0A0B 100%)";
/// Radial gradient - For spotlight effects
pub const GRADIENT_RADIAL: &str =
    "radial-gradient(circle at 50% 0%, rgba(0, 245, 160, 0.15) 0%, transparent 50%)";

// ============================================================================
// ORACLE VERDICT COLORS
// ============================================================================

/// Book Now verdict - Strong green
pub const VERDICT_BOOK: &str = "#00F5A0";
/// Wait verdict - Warning amber
pub const VERDICT_WAIT: &str = "#FFB800";
/// Join Pool verdict - Cyan collaborative
pub const VERDICT_POOL: &str = "#00D9FF";
/// Uncertain verdict - Neutral gray
pub const VERDICT_UNCERTAIN: &str = "#7A7A82";
