use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn Toast() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <Show when=move || state.toast.with(|t| t.is_some())>
            <div class="toast-container">
                <div class="toast-message">
                    {move || state.toast.get().unwrap_or_default()}
                </div>
            </div>
        </Show>
    }
}
