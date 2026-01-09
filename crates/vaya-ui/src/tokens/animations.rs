//! VAYA Animation System
//!
//! Timing functions and durations from the Animation Guide.
//! All animations should use these tokens for consistency.

// ============================================================================
// DURATION TOKENS
// ============================================================================

/// 50ms - Instant feedback (ripples, micro-interactions)
pub const DURATION_INSTANT: &str = "50ms";
/// 150ms - Fast transitions (hover, focus)
pub const DURATION_FAST: &str = "150ms";
/// 250ms - Normal transitions (state changes)
pub const DURATION_NORMAL: &str = "250ms";
/// 400ms - Slow transitions (page transitions)
pub const DURATION_SLOW: &str = "400ms";
/// 600ms - Slower transitions (complex animations)
pub const DURATION_SLOWER: &str = "600ms";
/// 800ms - Loading spinners
pub const DURATION_LOADING: &str = "800ms";
/// 2000-3000ms - Celebration animations
pub const DURATION_CELEBRATION: &str = "2500ms";

// ============================================================================
// EASING FUNCTIONS
// ============================================================================

/// Ease out - For entering elements
pub const EASE_OUT: &str = "cubic-bezier(0.0, 0.0, 0.2, 1)";
/// Ease in - For exiting elements
pub const EASE_IN: &str = "cubic-bezier(0.4, 0.0, 1, 1)";
/// Ease in-out - For transforms
pub const EASE_IN_OUT: &str = "cubic-bezier(0.4, 0.0, 0.2, 1)";
/// Spring - For bouncy enter animations
pub const EASE_SPRING: &str = "cubic-bezier(0.175, 0.885, 0.32, 1.275)";
/// Bounce - For overshoot effects
pub const EASE_BOUNCE: &str = "cubic-bezier(0.68, -0.55, 0.265, 1.55)";

// ============================================================================
// COMMON TRANSITIONS
// ============================================================================

/// Standard button transition
pub const TRANSITION_BUTTON: &str = "all 150ms cubic-bezier(0.0, 0.0, 0.2, 1)";
/// Standard input transition
pub const TRANSITION_INPUT: &str = "all 150ms cubic-bezier(0.0, 0.0, 0.2, 1)";
/// Card hover transition
pub const TRANSITION_CARD: &str = "all 250ms cubic-bezier(0.0, 0.0, 0.2, 1)";
/// Page transition
pub const TRANSITION_PAGE: &str = "all 400ms cubic-bezier(0.4, 0.0, 0.2, 1)";

// ============================================================================
// ORACLE REVEAL SEQUENCE TIMINGS
// ============================================================================

/// Stage 1: Backdrop blur in (0-200ms)
pub const ORACLE_BACKDROP_DURATION: u32 = 200;
pub const ORACLE_BACKDROP_DELAY: u32 = 0;

/// Stage 2: Card scale in (200-500ms)
pub const ORACLE_CARD_DURATION: u32 = 300;
pub const ORACLE_CARD_DELAY: u32 = 200;

/// Stage 3: Confidence bar fill (500-1300ms)
pub const ORACLE_CONFIDENCE_DURATION: u32 = 800;
pub const ORACLE_CONFIDENCE_DELAY: u32 = 500;

/// Stage 4: Verdict typewriter (1300-1700ms)
pub const ORACLE_VERDICT_CHAR_DELAY: u32 = 30;
pub const ORACLE_VERDICT_START_DELAY: u32 = 1300;

/// Stage 5: CTA slide up (1700-1900ms)
pub const ORACLE_CTA_DURATION: u32 = 200;
pub const ORACLE_CTA_DELAY: u32 = 1700;

// ============================================================================
// KEYFRAME NAMES
// ============================================================================

/// Spin animation (for loading spinners)
pub const KEYFRAME_SPIN: &str = "vaya-spin";
/// Pulse animation (for loading states)
pub const KEYFRAME_PULSE: &str = "vaya-pulse";
/// Fade in animation
pub const KEYFRAME_FADE_IN: &str = "vaya-fade-in";
/// Slide up animation
pub const KEYFRAME_SLIDE_UP: &str = "vaya-slide-up";
/// Scale in animation
pub const KEYFRAME_SCALE_IN: &str = "vaya-scale-in";
/// Confidence fill animation
pub const KEYFRAME_CONFIDENCE_FILL: &str = "vaya-confidence-fill";
