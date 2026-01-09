//! VAYA Spacing System - 8px base unit
//!
//! All spacing values are multiples of 8px (with 4px exception for micro spacing).
//! Use these constants for consistent visual rhythm throughout the UI.

// ============================================================================
// SPACING SCALE
// ============================================================================

/// 4px - Micro spacing (exception to 8px rule)
pub const SPACE_1: &str = "4px";
/// 8px - Base unit
pub const SPACE_2: &str = "8px";
/// 12px - Compact spacing
pub const SPACE_3: &str = "12px";
/// 16px - Standard small
pub const SPACE_4: &str = "16px";
/// 20px - Medium-small
pub const SPACE_5: &str = "20px";
/// 24px - Medium
pub const SPACE_6: &str = "24px";
/// 32px - Large
pub const SPACE_8: &str = "32px";
/// 40px - Extra large
pub const SPACE_10: &str = "40px";
/// 48px - Section spacing
pub const SPACE_12: &str = "48px";
/// 64px - Major section spacing
pub const SPACE_16: &str = "64px";
/// 80px - Page section spacing
pub const SPACE_20: &str = "80px";
/// 96px - Maximum spacing
pub const SPACE_24: &str = "96px";

// ============================================================================
// BORDER RADIUS
// ============================================================================

/// 6px - Small radius (tags, badges)
pub const RADIUS_SM: &str = "6px";
/// 10px - Medium radius (inputs)
pub const RADIUS_MD: &str = "10px";
/// 14px - Large radius (cards)
pub const RADIUS_LG: &str = "14px";
/// 20px - Extra large radius (modals)
pub const RADIUS_XL: &str = "20px";
/// 28px - 2XL radius (hero cards)
pub const RADIUS_2XL: &str = "28px";
/// Full pill radius
pub const RADIUS_FULL: &str = "9999px";

// ============================================================================
// SHADOWS
// ============================================================================

/// Small shadow - Subtle elevation
pub const SHADOW_SM: &str = "0 2px 8px rgba(0, 0, 0, 0.3)";
/// Medium shadow - Cards
pub const SHADOW_MD: &str = "0 4px 16px rgba(0, 0, 0, 0.4)";
/// Large shadow - Modals
pub const SHADOW_LG: &str = "0 8px 32px rgba(0, 0, 0, 0.5)";
/// Extra large shadow - Popovers
pub const SHADOW_XL: &str = "0 16px 48px rgba(0, 0, 0, 0.6)";

// ============================================================================
// TOUCH TARGETS
// ============================================================================

/// Mobile minimum touch target (44px per Apple HIG)
pub const TOUCH_TARGET_MOBILE: &str = "44px";
/// Desktop minimum click target (36px)
pub const TOUCH_TARGET_DESKTOP: &str = "36px";
/// Kiosk minimum touch target (60px)
pub const TOUCH_TARGET_KIOSK: &str = "60px";

// ============================================================================
// COMPONENT HEIGHTS
// ============================================================================

/// Small button/input height
pub const HEIGHT_SM: &str = "36px";
/// Medium button/input height (mobile)
pub const HEIGHT_MD: &str = "44px";
/// Large button/input height (desktop)
pub const HEIGHT_LG: &str = "52px";
/// Extra large button/input height
pub const HEIGHT_XL: &str = "56px";

// ============================================================================
// BREAKPOINTS (for reference - use CSS media queries)
// ============================================================================

/// Mobile breakpoint start
pub const BREAKPOINT_MOBILE: u32 = 320;
/// Tablet breakpoint start
pub const BREAKPOINT_TABLET: u32 = 744;
/// Desktop breakpoint start
pub const BREAKPOINT_DESKTOP: u32 = 1024;
/// Large desktop breakpoint start
pub const BREAKPOINT_LARGE: u32 = 1440;
/// Extra large breakpoint start
pub const BREAKPOINT_XL: u32 = 1920;

// ============================================================================
// CONTAINER WIDTHS
// ============================================================================

/// Maximum content width
pub const CONTAINER_MAX: &str = "1440px";
/// Narrow content width (for forms)
pub const CONTAINER_NARROW: &str = "640px";
/// Wide content width (for dashboards)
pub const CONTAINER_WIDE: &str = "1920px";
