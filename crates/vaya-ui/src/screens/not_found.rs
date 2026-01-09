//! 404 Not Found Screen
//!
//! Displayed when user navigates to a route that doesn't exist.

use leptos::*;

/// 404 Not Found screen component
#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="screen-not-found">
            <div class="not-found-content">
                <div class="not-found-code">"404"</div>
                <h1 class="not-found-title">"Page not found"</h1>
                <p class="not-found-message">
                    "The Oracle couldn't find what you're looking for. "
                    "The page may have moved or doesn't exist."
                </p>
                <div class="not-found-actions">
                    <a href="/" class="btn btn-primary">
                        "Return Home"
                    </a>
                </div>
            </div>
        </div>
    }
}
