//! Card Entry Screen
//!
//! Secure card details entry with formatting and validation.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::hooks::validation::{
    validate, card_number_rules, cvv_rules, expiry_rules,
    format_card_number, format_expiry, get_card_type,
};

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Card entry screen
#[component]
pub fn CardEntry() -> impl IntoView {
    let navigate = use_navigate();

    // Get total from storage
    let total: i64 = get_session_storage()
        .and_then(|s| s.get_item("booking_total").ok().flatten())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Form state
    let card_number = create_rw_signal(String::new());
    let card_name = create_rw_signal(String::new());
    let expiry = create_rw_signal(String::new());
    let cvv = create_rw_signal(String::new());

    // Errors
    let (card_error, set_card_error) = create_signal::<Option<String>>(None);
    let (name_error, set_name_error) = create_signal::<Option<String>>(None);
    let (expiry_error, set_expiry_error) = create_signal::<Option<String>>(None);
    let (cvv_error, set_cvv_error) = create_signal::<Option<String>>(None);

    // Card type detection
    let detected_card_type = move || get_card_type(&card_number.get());

    // Format card number as user types
    let handle_card_input = move |ev: web_sys::Event| {
        let raw = event_target_value(&ev);
        let formatted = format_card_number(&raw);
        // Limit to 19 chars (16 digits + 3 spaces)
        let limited: String = formatted.chars().take(19).collect();
        card_number.set(limited);
    };

    // Format expiry as user types
    let handle_expiry_input = move |ev: web_sys::Event| {
        let raw = event_target_value(&ev);
        let formatted = format_expiry(&raw);
        expiry.set(formatted);
    };

    // Validation
    let validate_card = move || {
        let raw: String = card_number.get().chars().filter(|c| c.is_ascii_digit()).collect();
        match validate(&raw, &card_number_rules()) {
            Ok(_) => { set_card_error.set(None); true }
            Err(e) => { set_card_error.set(Some(e)); false }
        }
    };

    let validate_name = move || {
        if card_name.get().trim().is_empty() {
            set_name_error.set(Some("Name is required".to_string()));
            false
        } else {
            set_name_error.set(None);
            true
        }
    };

    let validate_expiry_field = move || {
        match validate(&expiry.get(), &expiry_rules()) {
            Ok(_) => { set_expiry_error.set(None); true }
            Err(e) => { set_expiry_error.set(Some(e)); false }
        }
    };

    let validate_cvv_field = move || {
        match validate(&cvv.get(), &cvv_rules()) {
            Ok(_) => { set_cvv_error.set(None); true }
            Err(e) => { set_cvv_error.set(Some(e)); false }
        }
    };

    // Overall form validity
    let is_valid = move || {
        !card_number.get().is_empty()
            && !card_name.get().is_empty()
            && !expiry.get().is_empty()
            && !cvv.get().is_empty()
    };

    // Handle submit
    let handle_submit = {
        let nav = navigate.clone();
        move |_| {
            let card_ok = validate_card();
            let name_ok = validate_name();
            let expiry_ok = validate_expiry_field();
            let cvv_ok = validate_cvv_field();

            if card_ok && name_ok && expiry_ok && cvv_ok {
                // In production: tokenize card, never store raw details
                // For demo: go to 3DS or processing
                nav("/payment/3ds", Default::default());
            }
        }
    };

    // Handle back
    let handle_back = {
        let nav = navigate;
        move |_| {
            nav("/payment/method", Default::default());
        }
    };

    view! {
        <div class="screen-card-entry">
            // Header
            <header class="card-header">
                <button class="back-button" on:click=handle_back aria-label="Go back">
                    "← Back"
                </button>
                <h1 class="page-title">"Card Details"</h1>
            </header>

            // Card preview
            <div class="card-preview">
                <div class="card-preview-inner">
                    <div class="card-preview-type">
                        {move || detected_card_type().unwrap_or("Card")}
                    </div>
                    <div class="card-preview-number">
                        {move || {
                            let num = card_number.get();
                            if num.is_empty() { "•••• •••• •••• ••••".to_string() } else { num }
                        }}
                    </div>
                    <div class="card-preview-bottom">
                        <div class="card-preview-name">
                            {move || {
                                let name = card_name.get();
                                if name.is_empty() { "YOUR NAME".to_string() } else { name.to_uppercase() }
                            }}
                        </div>
                        <div class="card-preview-expiry">
                            {move || {
                                let exp = expiry.get();
                                if exp.is_empty() { "MM/YY".to_string() } else { exp }
                            }}
                        </div>
                    </div>
                </div>
            </div>

            // Card form
            <form class="card-form" on:submit=|ev| ev.prevent_default()>
                // Card number
                <div class="form-field">
                    <label class="input-label" for="card-number">"Card Number"</label>
                    <input
                        id="card-number"
                        type="text"
                        class="input card-number-input"
                        class:input-error=move || card_error.get().is_some()
                        placeholder="1234 5678 9012 3456"
                        inputmode="numeric"
                        autocomplete="cc-number"
                        value=move || card_number.get()
                        on:input=handle_card_input
                        on:blur=move |_| { validate_card(); }
                    />
                    {move || card_error.get().map(|e| view! {
                        <span class="input-error-text" role="alert">{e}</span>
                    })}
                </div>

                // Cardholder name
                <div class="form-field">
                    <label class="input-label" for="card-name">"Name on Card"</label>
                    <input
                        id="card-name"
                        type="text"
                        class="input"
                        class:input-error=move || name_error.get().is_some()
                        placeholder="JOHN DOE"
                        autocomplete="cc-name"
                        value=move || card_name.get()
                        on:input=move |ev| card_name.set(event_target_value(&ev).to_uppercase())
                        on:blur=move |_| { validate_name(); }
                    />
                    {move || name_error.get().map(|e| view! {
                        <span class="input-error-text" role="alert">{e}</span>
                    })}
                </div>

                // Expiry and CVV row
                <div class="card-row">
                    <div class="form-field">
                        <label class="input-label" for="card-expiry">"Expiry Date"</label>
                        <input
                            id="card-expiry"
                            type="text"
                            class="input expiry-input"
                            class:input-error=move || expiry_error.get().is_some()
                            placeholder="MM/YY"
                            inputmode="numeric"
                            autocomplete="cc-exp"
                            maxlength=5
                            value=move || expiry.get()
                            on:input=handle_expiry_input
                            on:blur=move |_| { validate_expiry_field(); }
                        />
                        {move || expiry_error.get().map(|e| view! {
                            <span class="input-error-text" role="alert">{e}</span>
                        })}
                    </div>

                    <div class="form-field">
                        <label class="input-label" for="card-cvv">"CVV"</label>
                        <input
                            id="card-cvv"
                            type="text"
                            class="input cvv-input"
                            class:input-error=move || cvv_error.get().is_some()
                            placeholder="123"
                            inputmode="numeric"
                            autocomplete="cc-csc"
                            maxlength=4
                            value=move || cvv.get()
                            on:input=move |ev| {
                                let val: String = event_target_value(&ev).chars().filter(|c| c.is_ascii_digit()).take(4).collect();
                                cvv.set(val);
                            }
                            on:blur=move |_| { validate_cvv_field(); }
                        />
                        {move || cvv_error.get().map(|e| view! {
                            <span class="input-error-text" role="alert">{e}</span>
                        })}
                    </div>
                </div>
            </form>

            // Security badges
            <div class="security-badges">
                <span class="security-badge">"🔒 Secure"</span>
                <span class="security-badge">"PCI Compliant"</span>
                <span class="security-badge">"3D Secure"</span>
            </div>

            // Submit button
            <div class="card-footer">
                <button
                    class="btn btn-primary btn-lg btn-full"
                    disabled=move || !is_valid()
                    on:click=handle_submit
                >
                    {format!("Pay RM {:.2}", total as f64 / 100.0)}
                </button>
            </div>
        </div>
    }
}
