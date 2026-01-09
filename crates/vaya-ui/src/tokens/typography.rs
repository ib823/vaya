//! VAYA Typography System
//!
//! Font families, sizes, weights, and line heights from the Design System.

// ============================================================================
// FONT FAMILIES
// ============================================================================

/// Display font - Headings, prices, brand elements
pub const FONT_DISPLAY: &str = "'Space Grotesk', sans-serif";
/// Body font - Body text, UI elements
pub const FONT_BODY: &str = "'DM Sans', sans-serif";
/// Monospace font - Code, data, timestamps
pub const FONT_MONO: &str = "'JetBrains Mono', monospace";

// ============================================================================
// FONT WEIGHTS
// ============================================================================

pub const WEIGHT_REGULAR: u16 = 400;
pub const WEIGHT_MEDIUM: u16 = 500;
pub const WEIGHT_SEMIBOLD: u16 = 600;
pub const WEIGHT_BOLD: u16 = 700;
pub const WEIGHT_EXTRABOLD: u16 = 800;

// ============================================================================
// MOBILE TYPOGRAPHY
// ============================================================================

/// Hero prices (48px/700)
pub const TYPE_MOBILE_PRICE_XL: &str = "48px";
pub const TYPE_MOBILE_PRICE_XL_WEIGHT: u16 = 700;

/// Card prices (36px/700)
pub const TYPE_MOBILE_PRICE_LG: &str = "36px";
pub const TYPE_MOBILE_PRICE_LG_WEIGHT: u16 = 700;

/// Inline prices (28px/700)
pub const TYPE_MOBILE_PRICE_MD: &str = "28px";
pub const TYPE_MOBILE_PRICE_MD_WEIGHT: u16 = 700;

/// H1 (28px/700)
pub const TYPE_MOBILE_H1: &str = "28px";
pub const TYPE_MOBILE_H1_WEIGHT: u16 = 700;

/// H2 (22px/600)
pub const TYPE_MOBILE_H2: &str = "22px";
pub const TYPE_MOBILE_H2_WEIGHT: u16 = 600;

/// H3 (18px/600)
pub const TYPE_MOBILE_H3: &str = "18px";
pub const TYPE_MOBILE_H3_WEIGHT: u16 = 600;

/// Body (15px/400)
pub const TYPE_MOBILE_BODY: &str = "15px";
pub const TYPE_MOBILE_BODY_WEIGHT: u16 = 400;

/// Caption (12px/400)
pub const TYPE_MOBILE_CAPTION: &str = "12px";
pub const TYPE_MOBILE_CAPTION_WEIGHT: u16 = 400;

// ============================================================================
// TABLET TYPOGRAPHY
// ============================================================================

/// Hero prices (56px/700)
pub const TYPE_TABLET_PRICE_XL: &str = "56px";

/// H1 (36px/700)
pub const TYPE_TABLET_H1: &str = "36px";

/// H2 (28px/600)
pub const TYPE_TABLET_H2: &str = "28px";

// ============================================================================
// DESKTOP TYPOGRAPHY
// ============================================================================

/// Hero prices (72px/700)
pub const TYPE_DESKTOP_PRICE_XL: &str = "72px";

/// Hero text (96px/700)
pub const TYPE_DESKTOP_HERO: &str = "96px";

/// H1 (48px/700)
pub const TYPE_DESKTOP_H1: &str = "48px";

/// H2 (36px/600)
pub const TYPE_DESKTOP_H2: &str = "36px";

/// H3 (28px/600)
pub const TYPE_DESKTOP_H3: &str = "28px";

/// Body large (18px/400)
pub const TYPE_DESKTOP_BODY_LG: &str = "18px";

/// Body (16px/400)
pub const TYPE_DESKTOP_BODY: &str = "16px";

// ============================================================================
// LINE HEIGHTS
// ============================================================================

/// Tight line height (1.1) - Headings
pub const LINE_HEIGHT_TIGHT: &str = "1.1";
/// Snug line height (1.25) - Subheadings
pub const LINE_HEIGHT_SNUG: &str = "1.25";
/// Normal line height (1.5) - Body text
pub const LINE_HEIGHT_NORMAL: &str = "1.5";
/// Relaxed line height (1.75) - Long form
pub const LINE_HEIGHT_RELAXED: &str = "1.75";

// ============================================================================
// LETTER SPACING
// ============================================================================

/// Tight letter spacing (-0.02em) - Large headings
pub const LETTER_SPACING_TIGHT: &str = "-0.02em";
/// Normal letter spacing (0) - Body text
pub const LETTER_SPACING_NORMAL: &str = "0";
/// Wide letter spacing (0.05em) - Buttons, labels
pub const LETTER_SPACING_WIDE: &str = "0.05em";
/// Wider letter spacing (0.1em) - All caps text
pub const LETTER_SPACING_WIDER: &str = "0.1em";
