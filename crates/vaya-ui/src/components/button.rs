//! Button Component
//!
//! Implements button variants as specified in the Component Library.

use leptos::*;

/// Button visual variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonVariant {
    /// Mint background, black text - Primary CTA
    #[default]
    Primary,
    /// Mint border, transparent background - Secondary CTA
    Secondary,
    /// No border, transparent - Tertiary actions
    Ghost,
    /// Error red background - Destructive actions
    Danger,
}

impl ButtonVariant {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Primary => "btn-primary",
            Self::Secondary => "btn-secondary",
            Self::Ghost => "btn-ghost",
            Self::Danger => "btn-danger",
        }
    }
}

/// Button size variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonSize {
    /// 36px height - Compact
    Small,
    /// 44px height - Mobile default
    #[default]
    Medium,
    /// 52px height - Desktop default
    Large,
}

impl ButtonSize {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Small => "btn-sm",
            Self::Medium => "btn-md",
            Self::Large => "btn-lg",
        }
    }
}

/// Primary button component with all design system states
#[component]
pub fn Button(
    /// Button content (text)
    #[prop(into)]
    label: String,
    /// Visual variant
    #[prop(optional)]
    variant: Option<ButtonVariant>,
    /// Size variant
    #[prop(optional)]
    size: Option<ButtonSize>,
    /// Disabled state (prevents interaction)
    #[prop(optional)]
    disabled: Option<bool>,
    /// Loading state (shows spinner, prevents interaction)
    #[prop(optional)]
    loading: Option<bool>,
    /// Full width button
    #[prop(optional)]
    full_width: Option<bool>,
    /// Click handler
    #[prop(optional)]
    on_click: Option<Callback<ev::MouseEvent>>,
    /// Optional href to make button an anchor
    #[prop(optional)]
    href: Option<String>,
    /// Button type attribute
    #[prop(optional)]
    button_type: Option<String>,
) -> impl IntoView {
    // Apply defaults
    let variant = variant.unwrap_or_default();
    let size = size.unwrap_or_default();
    let disabled = disabled.unwrap_or(false);
    let loading = loading.unwrap_or(false);
    let full_width = full_width.unwrap_or(false);
    let button_type = button_type.unwrap_or_else(|| "button".to_string());

    let class = {
        let mut classes = vec!["btn", variant.css_class(), size.css_class()];
        if loading {
            classes.push("btn-loading");
        }
        if disabled {
            classes.push("btn-disabled");
        }
        if full_width {
            classes.push("btn-full");
        }
        classes.join(" ")
    };

    let handle_click = move |ev: ev::MouseEvent| {
        if !disabled && !loading {
            if let Some(callback) = on_click {
                callback.call(ev);
            }
        }
    };

    let display_text = if loading {
        "Loading...".to_string()
    } else {
        label.clone()
    };

    // If href is provided, render as anchor
    if let Some(url) = href {
        view! {
            <a
                class=class.clone()
                href=url
                role="button"
                aria-disabled=disabled || loading
            >
                {if loading {
                    view! {
                        <span class="btn-spinner" aria-hidden="true"></span>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
                <span class="btn-text">{display_text}</span>
            </a>
        }
        .into_view()
    } else {
        view! {
            <button
                class=class
                type=button_type
                disabled=disabled || loading
                aria-disabled=disabled || loading
                aria-busy=loading
                on:click=handle_click
            >
                {if loading {
                    view! {
                        <span class="btn-spinner" aria-hidden="true"></span>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
                <span class="btn-text">{display_text}</span>
            </button>
        }
        .into_view()
    }
}

/// Icon button component for icon-only buttons
#[component]
pub fn IconButton(
    /// Icon character or emoji
    #[prop(into)]
    icon: String,
    /// Accessible label (required for icon buttons)
    #[prop(into)]
    label: String,
    /// Visual variant
    #[prop(optional)]
    variant: Option<ButtonVariant>,
    /// Size variant
    #[prop(optional)]
    size: Option<ButtonSize>,
    /// Disabled state
    #[prop(optional)]
    disabled: Option<bool>,
    /// Click handler
    #[prop(optional)]
    on_click: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    // Apply defaults
    let variant = variant.unwrap_or_default();
    let size = size.unwrap_or_default();
    let disabled = disabled.unwrap_or(false);

    let class = {
        let mut classes = vec!["btn", "btn-icon", variant.css_class(), size.css_class()];
        if disabled {
            classes.push("btn-disabled");
        }
        classes.join(" ")
    };

    let handle_click = move |ev: ev::MouseEvent| {
        if !disabled {
            if let Some(callback) = on_click {
                callback.call(ev);
            }
        }
    };

    view! {
        <button
            class=class
            type="button"
            disabled=disabled
            aria-label=label.clone()
            title=label
            on:click=handle_click
        >
            {icon}
        </button>
    }
}
