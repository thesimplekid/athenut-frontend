use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_navigate;

use crate::components::{Footer, Navbar};
use crate::state::AppState;
use crate::wallet;

#[component]
pub fn HomePage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let search_query = RwSignal::new(String::new());
    let is_loading = RwSignal::new(true);
    let content_ready = RwSignal::new(false);

    spawn_local(async move {
        state.balance.set(wallet::balance().await);
        is_loading.set(false);
        TimeoutFuture::new(100).await;
        content_ready.set(true);
    });

    let navigate = use_navigate();
    let handle_search = move || {
        let query = search_query.get_untracked();
        let query = query.trim();
        if !query.is_empty() {
            let encoded = js_sys::encode_uri_component(query);
            navigate(&format!("/search?q={encoded}"), Default::default());
        }
    };
    let handle_search_click = handle_search.clone();

    view! {
        <Title text="Athenut"/>
        <Meta name="description" content="Privacy-preserving web search powered by Kagi and Cashu."/>
        <Meta name="keywords" content="search, kagi, ecash, cashu"/>
        <Meta name="robots" content="index, follow"/>

        <div class="page-home">
            <div class="page-wrapper">
                <Navbar/>

                // Main content with top padding for fixed navbar
                <div class="main-content">
                    <Show when=move || content_ready.get()>
                        <div class="container anim-fade">
                            <img
                                src="/assets/wordmark.png"
                                alt="X-Cashu Search"
                                class="wordmark anim-scale"
                                style="--anim-delay: 80ms"
                            />

                            <h2 class="tagline text-balance anim-fly-up" style="--anim-delay: 140ms">
                                "Search smarter. Pay in sats for results that matter."
                            </h2>

                            <div class="search-container anim-fly-up" style="--anim-delay: 180ms">
                                {
                                    let handle_search = handle_search_click.clone();
                                    move || {
                                        let handle_search_btn = handle_search.clone();
                                        let handle_search_key = handle_search.clone();
                                        if is_loading.get() {
                                            view! {
                                                <div class="search-skeleton">
                                                    <div class="skeleton-input"></div>
                                                    <div class="skeleton-button"></div>
                                                </div>
                                            }.into_any()
                                        } else if state.balance.get() == 0 {
                                            view! {
                                                <div class="empty-state anim-scale">
                                                    <h3 class="empty-state-title">"Top Up Required"</h3>
                                                    <p class="empty-state-description text-pretty">
                                                        "You need to add funds to start searching."
                                                    </p>
                                                    <a href="/topup" class="empty-state-button">"Top Up Now"</a>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="search-form">
                                                    <div class="search-input-wrapper">
                                                        <div class="search-input-container">
                                                            <input
                                                                type="text"
                                                                autocomplete="off"
                                                                placeholder="Ask whatever you want..."
                                                                class="search-input"
                                                                prop:value=move || search_query.get()
                                                                on:input=move |ev| search_query.set(event_target_value(&ev))
                                                                on:keyup=move |ev| {
                                                                    if ev.key() == "Enter" {
                                                                        handle_search_key();
                                                                    }
                                                                }
                                                            />
                                                        </div>
                                                    </div>
                                                    <button
                                                        class="search-button"
                                                        on:click=move |_| handle_search_btn()
                                                        aria-label="Search"
                                                    >
                                                        <span class="search-button-text">"Search"</span>
                                                        <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                                            <circle cx="11" cy="11" r="8"></circle>
                                                            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                                                        </svg>
                                                    </button>
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                }
                            </div>
                        </div>
                    </Show>
                </div>

                <Footer/>
            </div>
        </div>
    }
}
