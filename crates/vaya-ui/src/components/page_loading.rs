//! Loading Components
//!
//! Spinner and skeleton loading states for async content.

use leptos::*;

/// Spinner size variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SpinnerSize {
    /// 16px - Inline/button spinner
    Small,
    /// 24px - Default spinner
    #[default]
    Medium,
    /// 40px - Page loading
    Large,
    /// 64px - Full page loading
    XLarge,
}

impl SpinnerSize {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Small => "spinner-sm",
            Self::Medium => "spinner-md",
            Self::Large => "spinner-lg",
            Self::XLarge => "spinner-xl",
        }
    }
}

/// Loading spinner component
#[component]
pub fn LoadingSpinner(
    /// Size of the spinner
    #[prop(optional)]
    size: Option<SpinnerSize>,
    /// Optional label for accessibility
    #[prop(optional)]
    label: Option<String>,
) -> impl IntoView {
    let size = size.unwrap_or_default();
    let aria_label = label.unwrap_or_else(|| "Loading...".to_string());

    view! {
        <div
            class=format!("spinner {}", size.css_class())
            role="status"
            aria-label=aria_label.clone()
        >
            <span class="sr-only">{aria_label}</span>
        </div>
    }
}

/// Skeleton loading placeholder
#[component]
pub fn Skeleton(
    /// Width (CSS value)
    #[prop(optional)]
    width: Option<String>,
    /// Height (CSS value)
    #[prop(optional)]
    height: Option<String>,
    /// Border radius (CSS value)
    #[prop(optional)]
    radius: Option<String>,
) -> impl IntoView {
    let width = width.unwrap_or_else(|| "100%".to_string());
    let height = height.unwrap_or_else(|| "20px".to_string());
    let radius = radius.unwrap_or_else(|| "6px".to_string());

    let style = format!(
        "width: {}; height: {}; border-radius: {};",
        width, height, radius
    );

    view! {
        <div
            class="skeleton"
            style=style
            role="presentation"
            aria-hidden="true"
        ></div>
    }
}

/// Skeleton for text lines
#[component]
pub fn SkeletonText(
    /// Number of lines
    #[prop(optional)]
    lines: Option<usize>,
    /// Last line width percentage
    #[prop(optional)]
    last_line_width: Option<u8>,
) -> impl IntoView {
    let lines = lines.unwrap_or(3);
    let last_line_width = last_line_width.unwrap_or(60);

    view! {
        <div class="skeleton-text">
            {(0..lines)
                .map(|i| {
                    let width = if i == lines - 1 {
                        format!("{}%", last_line_width)
                    } else {
                        "100%".to_string()
                    };
                    view! {
                        <Skeleton width=width height="16px".to_string() />
                    }
                })
                .collect_view()}
        </div>
    }
}

/// Full page loading overlay
#[component]
pub fn PageLoading(
    /// Loading message to display
    #[prop(optional)]
    message: Option<String>,
) -> impl IntoView {
    let message = message.unwrap_or_else(|| "Loading...".to_string());

    view! {
        <div class="page-loading">
            <div class="page-loading-content">
                <LoadingSpinner size=SpinnerSize::XLarge />
                <p class="page-loading-message">{message}</p>
            </div>
        </div>
    }
}
