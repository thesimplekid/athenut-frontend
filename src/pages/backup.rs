use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{Meta, Title};

use crate::components::{copy_to_clipboard, Footer, Navbar};
use crate::state::AppState;
use crate::wallet;

#[component]
pub fn BackupPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    let seed = wallet::seed_phrase();
    let words: Vec<String> = seed.split_whitespace().map(str::to_string).collect();
    let words = StoredValue::new(words);
    let seed = StoredValue::new(seed);

    let is_blurred = RwSignal::new(true);
    let encoded_token = RwSignal::new(String::new());
    let content_ready = RwSignal::new(false);

    let refresh_token = move || {
        spawn_local(async move {
            match wallet::export_token().await {
                Ok(token) => encoded_token.set(token),
                Err(e) => state.show_toast(format!("Failed to generate token: {e}")),
            }
        });
    };

    spawn_local(async move {
        state.balance.set(wallet::balance().await);
        match wallet::export_token().await {
            Ok(token) => encoded_token.set(token),
            Err(e) => state.show_toast(format!("Failed to generate token: {e}")),
        }
        TimeoutFuture::new(100).await;
        content_ready.set(true);
    });

    view! {
        <Title text="Athenut"/>
        <Meta name="description" content="privacy-preserving web search powered by Kagi and Cashu."/>

        <div class="page-backup min-h-dvh flex flex-col">
            <Navbar/>
            <main class="flex-grow flex flex-col justify-start items-center px-4 pt-24 pb-8">
                <Show when=move || content_ready.get()>
                    <div class="relative w-full max-w-800 anim-fade">
                        <div class="anim-fly-up" style="--anim-delay: 90ms">
                            <h1 class="text-4xl font-bold mb-2 text-center">"Backup"</h1>

                            <div class="absolute" style="right: 0; top: 0;">
                                <button
                                    class="visibility-toggle"
                                    on:click=move |_| is_blurred.update(|b| *b = !*b)
                                    aria-label=move || if is_blurred.get() { "Show recovery phrase" } else { "Hide recovery phrase" }
                                >
                                    {move || if is_blurred.get() {
                                        view! {
                                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="eye-icon">
                                                // Closed eye
                                                <path d="m15 18-.722-3.25"/>
                                                <path d="M2 8a10.645 10.645 0 0 0 20 0"/>
                                                <path d="m20 15-1.726-2.05"/>
                                                <path d="m4 15 1.726-2.05"/>
                                                <path d="m9 18 .722-3.25"/>
                                            </svg>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="eye-icon">
                                                // Open eye
                                                <path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"/>
                                                <circle cx="12" cy="12" r="3"/>
                                            </svg>
                                        }.into_any()
                                    }}
                                </button>
                            </div>
                        </div>

                        <div class="text-2xl font-semibold mt-2 mb-4 text-center tabular-nums anim-fly-up" style="--anim-delay: 130ms">
                            "You have " {move || state.balance.get()} " searches left"
                            <button
                                class="refresh-balance-button"
                                on:click=move |_| state.refresh_balance()
                                aria-label="Refresh balance"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                                </svg>
                            </button>
                        </div>

                        <p class="text-xl text-gray-600 mb-6 text-center anim-fly-up" style="--anim-delay: 160ms">
                            "Save your secret recovery phrase in a secure place that only you control."
                        </p>

                        <div class="seed-container anim-scale" style="--anim-delay: 200ms">
                            {move || words.get_value().into_iter().enumerate().map(|(i, word)| view! {
                                <div class="seed-word">
                                    <span class="word-number">{i + 1}</span>
                                    <span class="word-text" class:blurred=move || is_blurred.get()>{word}</span>
                                    <div class="seed-underline"></div>
                                </div>
                            }).collect_view()}
                        </div>

                        <div class="button-container-right">
                            <button
                                class="recovery-button anim-scale"
                                style="--anim-delay: 220ms"
                                on:click=move |_| {
                                    copy_to_clipboard(&seed.get_value());
                                    state.show_toast("Recovery phrase copied to clipboard.");
                                }
                            >
                                "Copy Recovery Phrase"
                            </button>
                        </div>

                        <div class="divider my-8 anim-fade" style="--anim-delay: 240ms">"OR"</div>

                        <div class="token-section w-full flex flex-col items-center anim-fly-up" style="--anim-delay: 260ms">
                            <h2 class="text-2xl font-bold mb-4 text-center">"Export Search Token"</h2>

                            <div class="token-input-container seed-container" style="display: block; padding: 1rem;">
                                <input
                                    type="text"
                                    class="word-text"
                                    readonly
                                    prop:value=move || encoded_token.get()
                                    class:blurred=move || is_blurred.get()
                                />
                            </div>

                            <div class="button-container-right">
                                <button
                                    class="recovery-button-secondary"
                                    on:click=move |_| {
                                        copy_to_clipboard(&encoded_token.get_untracked());
                                        state.show_toast("Token copied to clipboard");
                                    }
                                >
                                    "Copy Token"
                                </button>

                                <button
                                    class="recovery-button-secondary"
                                    on:click=move |_| {
                                        refresh_token();
                                        state.show_toast("Token refreshed");
                                    }
                                >
                                    "Refresh Token"
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>
            </main>

            <Footer/>
        </div>
    }
}
