use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::{use_navigate, use_query_map};
use serde::{Deserialize, Serialize};

use crate::components::{Footer, Navbar};
use crate::state::AppState;
use crate::{storage, wallet};

const RESULTS_KEY: &str = "searchResults";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub age: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct StoredResults {
    results: Vec<SearchResult>,
    timestamp: f64,
    query: String,
}

fn save_results(query: &str, results: &[SearchResult]) {
    let stored = StoredResults {
        results: results.to_vec(),
        timestamp: js_sys::Date::now(),
        query: query.to_string(),
    };
    if let Ok(json) = serde_json::to_string(&stored) {
        storage::session_set(RESULTS_KEY, &json);
    }
}

fn stored_results() -> Option<StoredResults> {
    serde_json::from_str(&storage::session_get(RESULTS_KEY)?).ok()
}

async fn perform_search(query: &str) -> Result<Vec<SearchResult>, String> {
    let (y, token) = wallet::take_search_proof().await?;

    let encoded = js_sys::encode_uri_component(query);
    let url = format!("{}/search?q={}", wallet::api_base(), encoded);
    let response = gloo_net::http::Request::get(&url)
        .header("X-Cashu", &token)
        .send()
        .await;

    let response = match response {
        Ok(response) => response,
        Err(e) => {
            // Network failure: the mint never saw the proof, put it back.
            wallet::settle_search_proof(y, false).await;
            return Err(format!("Search failed: {e}"));
        }
    };

    if !response.ok() {
        if response.status() == 402 {
            // The backend rejected the proof, but a 402 does not prove it
            // was redeemed (it could be a keyset mismatch or backend
            // misconfiguration). Leave it PendingSpent and let the mint
            // decide: spent proofs are dropped, the rest recovered.
            wallet::reclaim_pending_proofs().await;
            return Err("Payment Required: Proof rejected".to_string());
        }
        wallet::settle_search_proof(y, false).await;
        return Err(format!("Search failed with status {}", response.status()));
    }

    match response.json::<Vec<SearchResult>>().await {
        Ok(results) => {
            wallet::settle_search_proof(y, true).await;
            Ok(results)
        }
        Err(e) => {
            wallet::settle_search_proof(y, true).await;
            Err(format!("Failed to parse search results: {e}"))
        }
    }
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let query_map = use_query_map();
    let navigate = use_navigate();

    let search_query = RwSignal::new(String::new());
    let search_results = RwSignal::new(Vec::<SearchResult>::new());
    let is_loading = RwSignal::new(false);
    let search_time = RwSignal::new("0".to_string());
    let search_performed = RwSignal::new(false);

    let run_search = {
        let navigate = navigate.clone();
        move || {
            let navigate = navigate.clone();
            spawn_local(async move {
                if is_loading.get_untracked() {
                    return;
                }
                let query = search_query.get_untracked().trim().to_string();
                if query.is_empty() {
                    return;
                }
                if wallet::balance().await == 0 {
                    state.balance.set(0);
                    navigate("/topup", Default::default());
                    return;
                }

                search_performed.set(true);
                is_loading.set(true);
                search_results.set(Vec::new());
                let start = js_sys::Date::now();

                match perform_search(&query).await {
                    Ok(results) => {
                        search_time.set(format!("{:.2}", (js_sys::Date::now() - start) / 1000.0));
                        save_results(&query, &results);
                        search_results.set(results);
                    }
                    Err(message) => {
                        state.show_toast(message);
                        save_results(&query, &[]);
                        if wallet::balance().await == 0 {
                            navigate("/topup", Default::default());
                        }
                    }
                }

                is_loading.set(false);
                state.refresh_balance();
            });
        }
    };

    // Initial search from the ?q= query parameter
    {
        let run_search = run_search.clone();
        let query = query_map.with_untracked(|q| q.get("q"));
        match query {
            Some(q) if !q.is_empty() => {
                search_query.set(q.clone());
                search_performed.set(true);
                match stored_results() {
                    Some(stored) if stored.query == q => {
                        search_results.set(stored.results);
                    }
                    _ => run_search(),
                }
            }
            _ => state.show_toast("No search query provided"),
        }
    }

    let run_search_key = run_search.clone();
    let run_search_btn = run_search.clone();

    view! {
        <Title text="Athenut"/>
        <Meta name="description" content="privacy-preserving web search powered by Kagi and Cashu."/>

        <div class="page-search min-h-dvh flex flex-col relative">
            <Navbar/>

            <header class="p-4 flex items-center" class:search-active=move || search_performed.get()>
                <div class="search-container flex-grow" class:search-active=move || search_performed.get()>
                    <div class="flex items-center">
                        <div class="search-input-wrapper flex-grow mr-2 relative">
                            <div class="rounded-input-container p-2 shadow-md w-full relative">
                                <input
                                    type="text"
                                    autocomplete="off"
                                    placeholder="Ask whatever you want..."
                                    class="rounded-input border-none w-full pr-10"
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| search_query.set(event_target_value(&ev))
                                    on:keyup=move |ev| {
                                        if ev.key() == "Enter" {
                                            run_search_key();
                                        }
                                    }
                                />
                                <button
                                    class="search-button"
                                    on:click=move |_| run_search_btn()
                                    aria-label="Search"
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <circle cx="11" cy="11" r="8"></circle>
                                        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                                    </svg>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </header>

            <div class="flex-grow flex flex-col relative search-results-container">
                <Show when=move || !is_loading.get() && !search_results.with(Vec::is_empty)>
                    <p class="text-sm mb-4 search-aligned tabular-nums" style="color: var(--text-secondary)">
                        "Found " {move || search_results.with(Vec::len)} " results in "
                        {move || search_time.get()} " seconds"
                    </p>
                </Show>

                <div class="flex-grow">
                    <main class="search-aligned">
                        {move || if is_loading.get() {
                            view! {
                                <div class="search-results-skeleton">
                                    {(0..3).map(|_| view! {
                                        <div class="skeleton-result-item">
                                            <div class="skeleton-title"></div>
                                            <div class="skeleton-url"></div>
                                            <div class="skeleton-description"></div>
                                            <div class="skeleton-description skeleton-description-short"></div>
                                        </div>
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        } else if search_results.with(Vec::is_empty) {
                            view! {
                                <p class="text-center text-gray-400">
                                    "No results found. Try a different search query."
                                </p>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-6">
                                    {move || search_results.get().into_iter().map(|result| {
                                        let show_age = result
                                            .age
                                            .as_deref()
                                            .map(|age| !age.is_empty() && age != "null")
                                            .unwrap_or(false);
                                        view! {
                                            // Search result item
                                            <div class="py-4 border-b">
                                                <h3 class="text-xl mb-2">
                                                    <a
                                                        href=result.url.clone()
                                                        class="font-medium underline"
                                                        style="color: var(--text-primary)"
                                                    >
                                                        {result.title.clone()}
                                                    </a>
                                                </h3>
                                                <p class="text-sm mb-2" style="color: var(--text-secondary)">
                                                    {result.url.clone()}
                                                </p>
                                                <p style="color: var(--text-primary)">
                                                    {result.description.clone()}
                                                </p>
                                                <Show when=move || show_age>
                                                    <span
                                                        class="inline-block mt-2 px-3 py-1 text-sm rounded-full"
                                                        style="background-color: var(--bg-secondary); color: var(--text-secondary)"
                                                    >
                                                        {result.age.clone().unwrap_or_default()}
                                                    </span>
                                                </Show>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }}
                    </main>
                </div>
            </div>

            <Footer/>
        </div>
    }
}
