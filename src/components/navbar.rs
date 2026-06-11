use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn Navbar() -> impl IntoView {
    let state = expect_context::<AppState>();
    let is_dropdown_open = RwSignal::new(false);
    let is_scrolled = RwSignal::new(false);

    state.refresh_balance();

    let scroll_handle = window_event_listener(leptos::ev::scroll, move |_| {
        let scrolled = window().scroll_y().map(|y| y > 10.0).unwrap_or(false);
        is_scrolled.set(scrolled);
    });
    on_cleanup(move || scroll_handle.remove());

    view! {
        <nav class="navbar anim-fly-down" class:scrolled=move || is_scrolled.get()>
            // Logo
            <a href="/" class="home-link">
                <img src="/assets/logomark.png" alt="X-Cashu Search Logo"/>
            </a>

            // Top right info
            <div class="top-right-container">
                <div class="top-right-info">
                    <span class="searches-left">
                        "Searches left: "
                        <span class="searches-count tabular-nums">{move || state.balance.get()}</span>
                    </span>
                    <a href="/topup" class="top-up-button">"Top Up"</a>

                    // Theme toggle button
                    <button
                        class="theme-toggle"
                        on:click=move |_| state.toggle_theme()
                        aria-label="Toggle theme"
                    >
                        {move || if state.is_dark() {
                            view! {
                                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <circle cx="12" cy="12" r="5"></circle>
                                    <line x1="12" y1="1" x2="12" y2="3"></line>
                                    <line x1="12" y1="21" x2="12" y2="23"></line>
                                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                                    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                                    <line x1="1" y1="12" x2="3" y2="12"></line>
                                    <line x1="21" y1="12" x2="23" y2="12"></line>
                                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                                    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                                </svg>
                            }.into_any()
                        } else {
                            view! {
                                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                                </svg>
                            }.into_any()
                        }}
                    </button>

                    // Dropdown menu
                    <div class="dropdown-container">
                        <button
                            class="more-options-button"
                            on:click=move |_| is_dropdown_open.update(|open| *open = !*open)
                            aria-label="More options"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                                <circle cx="12" cy="12" r="2"/>
                                <circle cx="12" cy="5" r="2"/>
                                <circle cx="12" cy="19" r="2"/>
                            </svg>
                        </button>

                        <Show when=move || is_dropdown_open.get()>
                            <div class="dropdown-menu">
                                <a href="/backup" class="dropdown-item">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="dropdown-icon">
                                        <path d="M15 12h-5"/>
                                        <path d="M15 8h-5"/>
                                        <path d="M19 17V5a2 2 0 0 0-2-2H4"/>
                                        <path d="M8 21h12a2 2 0 0 0 2-2v-1a1 1 0 0 0-1-1H11a1 1 0 0 0-1 1v1a2 2 0 1 1-4 0V5a2 2 0 1 0-4 0v2a1 1 0 0 0 1 1h3"/>
                                    </svg>
                                    "Back Up"
                                </a>
                                <a href="/faq" class="dropdown-item">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="dropdown-icon">
                                        <circle cx="12" cy="12" r="10"/>
                                        <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/>
                                        <path d="M12 17h.01"/>
                                    </svg>
                                    "FAQs"
                                </a>
                                <a href="/recovery" class="dropdown-item">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="dropdown-icon">
                                        <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                                        <path d="M3 3v5h5"/>
                                    </svg>
                                    "Recovery"
                                </a>
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </nav>
    }
}
