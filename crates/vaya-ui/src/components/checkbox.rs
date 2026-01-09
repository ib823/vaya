//! Checkbox Component
//!
//! A styled checkbox with label, validation states, and accessibility support.

use leptos::*;

/// Checkbox component with animated checkmark
#[component]
pub fn Checkbox(
    /// Label text displayed next to checkbox
    #[prop(into)]
    label: String,
    /// Controlled checked state
    checked: RwSignal<bool>,
    /// Whether the checkbox is disabled
    #[prop(optional)]
    disabled: bool,
    /// Whether the checkbox is required
    #[prop(optional)]
    required: bool,
    /// Error message to display
    #[prop(optional, into)]
    error: Option<String>,
    /// Callback when checked state changes
    #[prop(optional)]
    on_change: Option<Callback<bool>>,
) -> impl IntoView {
    let has_error = error.is_some();

    let wrapper_class = move || {
        let mut classes = vec!["checkbox-wrapper"];
        if disabled {
            classes.push("checkbox-disabled");
        }
        if has_error {
            classes.push("checkbox-error");
        }
        classes.join(" ")
    };

    let checkbox_id = format!(
        "checkbox-{}",
        label
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .take(20)
            .collect::<String>()
    );
    let error_id = format!("{}-error", checkbox_id);
    let error_id_attr = error_id.clone();

    view! {
        <div class=wrapper_class>
            <label class="checkbox-label" for=checkbox_id.clone()>
                <input
                    id=checkbox_id
                    type="checkbox"
                    class="checkbox-input"
                    checked=move || checked.get()
                    disabled=disabled
                    required=required
                    aria-invalid=has_error
                    aria-describedby=move || if has_error { Some(error_id_attr.clone()) } else { None }
                    on:change=move |ev| {
                        let is_checked = event_target_checked(&ev);
                        checked.set(is_checked);
                        if let Some(cb) = on_change {
                            cb.call(is_checked);
                        }
                    }
                />
                <span class="checkbox-box">
                    <svg class="checkbox-checkmark" viewBox="0 0 12 12" fill="none">
                        <path d="M2.5 6L5 8.5L9.5 3.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                </span>
                <span class="checkbox-text">
                    {label}
                    {required.then(|| view! { <span class="checkbox-required">" *"</span> })}
                </span>
            </label>
            {error.map(|err| {
                view! {
                    <span class="checkbox-error-text" id=error_id role="alert">
                        {err}
                    </span>
                }
            })}
        </div>
    }
}

/// Checkbox group for multiple related options
#[component]
pub fn CheckboxGroup(
    /// Group label
    #[prop(into)]
    label: String,
    /// Children checkboxes
    children: Children,
    /// Error message for the group
    #[prop(optional, into)]
    error: Option<String>,
) -> impl IntoView {
    let has_error = error.is_some();
    let error_id = format!(
        "checkbox-group-{}-error",
        label.to_lowercase().replace(' ', "-")
    );
    let error_id_attr = error_id.clone();

    view! {
        <fieldset class="checkbox-group" aria-describedby=move || if has_error { Some(error_id_attr.clone()) } else { None }>
            <legend class="checkbox-group-label">{label}</legend>
            <div class="checkbox-group-items">
                {children()}
            </div>
            {error.map(|err| {
                view! {
                    <span class="checkbox-group-error" id=error_id.clone() role="alert">
                        {err}
                    </span>
                }
            })}
        </fieldset>
    }
}

/// Terms and conditions checkbox with link
#[component]
pub fn TermsCheckbox(
    /// Controlled checked state
    checked: RwSignal<bool>,
    /// Terms URL
    #[prop(optional, into)]
    terms_url: Option<String>,
    /// Privacy URL
    #[prop(optional, into)]
    privacy_url: Option<String>,
    /// Error message
    #[prop(optional, into)]
    error: Option<String>,
) -> impl IntoView {
    let has_error = error.is_some();
    let terms_href = terms_url.unwrap_or_else(|| "/terms".to_string());
    let privacy_href = privacy_url.unwrap_or_else(|| "/privacy".to_string());
    let error_id = "terms-checkbox-error".to_string();
    let error_id_attr = error_id.clone();

    view! {
        <div class="terms-checkbox-wrapper">
            <label class="checkbox-label" for="terms-checkbox">
                <input
                    id="terms-checkbox"
                    type="checkbox"
                    class="checkbox-input"
                    checked=move || checked.get()
                    required=true
                    aria-invalid=has_error
                    aria-describedby=move || if has_error { Some(error_id_attr.clone()) } else { None }
                    on:change=move |ev| {
                        checked.set(event_target_checked(&ev));
                    }
                />
                <span class="checkbox-box">
                    <svg class="checkbox-checkmark" viewBox="0 0 12 12" fill="none">
                        <path d="M2.5 6L5 8.5L9.5 3.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                </span>
                <span class="checkbox-text">
                    "I agree to the "
                    <a href=terms_href target="_blank" class="terms-link">"Terms & Conditions"</a>
                    " and "
                    <a href=privacy_href target="_blank" class="terms-link">"Privacy Policy"</a>
                </span>
            </label>
            {error.map(|err| {
                view! {
                    <span class="checkbox-error-text" id=error_id role="alert">
                        {err}
                    </span>
                }
            })}
        </div>
    }
}
