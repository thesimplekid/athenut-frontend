use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_navigate;

use crate::components::{read_clipboard, Footer, Navbar};
use crate::state::AppState;
use crate::wallet;

#[component]
pub fn RecoveryPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let navigate = use_navigate();

    let words: Vec<RwSignal<String>> = (0..12).map(|_| RwSignal::new(String::new())).collect();
    let words = StoredValue::new(words);

    let error_message = RwSignal::new(String::new());
    let token_input = RwSignal::new(String::new());
    let token_error = RwSignal::new(String::new());
    let is_restoring = RwSignal::new(false);
    let token_restoring = RwSignal::new(false);
    let content_ready = RwSignal::new(false);

    let is_complete = Memo::new(move |_| {
        words
            .get_value()
            .iter()
            .all(|word| !word.get().trim().is_empty())
    });

    spawn_local(async move {
        TimeoutFuture::new(100).await;
        content_ready.set(true);
    });

    let handle_paste_phrase = move |_| {
        let words = words.get_value();
        spawn_local(async move {
            match read_clipboard().await {
                Ok(text) => {
                    let pasted: Vec<&str> = text.split_whitespace().collect();
                    if pasted.len() != 12 {
                        error_message.set("Please paste exactly 12 words".to_string());
                        return;
                    }
                    error_message.set(String::new());
                    for (signal, word) in words.iter().zip(pasted) {
                        signal.set(word.to_string());
                    }
                }
                Err(e) => error_message.set(e),
            }
        });
    };

    let handle_restore = {
        let navigate = navigate.clone();
        move |_| {
            if !is_complete.get_untracked() || is_restoring.get_untracked() {
                return;
            }
            let phrase = words
                .get_value()
                .iter()
                .map(|word| word.get_untracked().trim().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let navigate = navigate.clone();
            is_restoring.set(true);
            error_message.set(String::new());
            spawn_local(async move {
                let result = async {
                    wallet::set_seed_phrase(&phrase)?;
                    wallet::restore().await
                }
                .await;

                match result {
                    Ok(restored) => {
                        state.refresh_balance();
                        let unspent = u64::from(restored.unspent);
                        if unspent > 0 {
                            state.show_toast_for(format!("Restored {unspent} searches"), 5000);
                            TimeoutFuture::new(1000).await;
                            navigate("/", Default::default());
                        } else {
                            state.show_toast("No proofs were found to restore");
                        }
                    }
                    Err(e) => {
                        error_message.set(format!("Failed to restore wallet: {e}"));
                        state.show_toast_for(format!("Restore failed: {e}"), 5000);
                    }
                }
                is_restoring.set(false);
            });
        }
    };

    let handle_paste_token = move |_| {
        spawn_local(async move {
            match read_clipboard().await {
                Ok(text) => {
                    token_input.set(text);
                    token_error.set(String::new());
                }
                Err(e) => token_error.set(e),
            }
        });
    };

    let handle_token_redeem = {
        let navigate = navigate.clone();
        move |_| {
            let token = token_input.get_untracked().trim().to_string();
            if token.is_empty() || token_restoring.get_untracked() {
                if token.is_empty() {
                    token_error.set("Please enter a valid token".to_string());
                }
                return;
            }
            if !token.starts_with("cashu") {
                token_error
                    .set("Invalid token format. Token should start with 'cashu'.".to_string());
                return;
            }
            let navigate = navigate.clone();
            token_restoring.set(true);
            token_error.set(String::new());
            spawn_local(async move {
                match wallet::receive_token(&token).await {
                    Ok(amount) => {
                        state.refresh_balance();
                        state.show_toast_for(
                            format!("Redeemed {} searches", u64::from(amount)),
                            5000,
                        );
                        token_input.set(String::new());
                        TimeoutFuture::new(1000).await;
                        navigate("/", Default::default());
                    }
                    Err(e) => {
                        let lower = e.to_lowercase();
                        let message = if lower.contains("spent") {
                            "This token has already been spent.".to_string()
                        } else if lower.contains("token") {
                            "Invalid token format. Please check that you've entered a valid Cashu token.".to_string()
                        } else {
                            format!("Failed to redeem token: {e}")
                        };
                        token_error.set(message.clone());
                        state.show_toast_for(message, 5000);
                    }
                }
                token_restoring.set(false);
            });
        }
    };

    view! {
        <Title text="Athenut"/>
        <Meta name="description" content="privacy-preserving web search powered by Kagi and Cashu."/>

        <div class="page-recovery min-h-dvh flex flex-col relative">
            <Navbar/>

            <main class="flex-grow flex flex-col justify-start items-center px-4 pt-24 pb-8">
                <Show when=move || content_ready.get()>
                    <div class="anim-fade w-full max-w-800">
                        <h1 class="text-4xl font-bold mb-2 text-center anim-fly-up" style="--anim-delay: 90ms">
                            "Recovery"
                        </h1>

                        <p class="text-xl text-gray-600 mb-6 anim-fly-up" style="--anim-delay: 130ms">
                            "Enter your 12-word recovery phrase to restore your searches."
                        </p>

                        <div class="seed-container anim-scale" style="--anim-delay: 170ms">
                            {move || words.get_value().into_iter().enumerate().map(|(i, word)| {
                                view! {
                                    <div class="seed-word">
                                        <span class="word-number">{i + 1}</span>
                                        <input
                                            type="text"
                                            class="word-text"
                                            prop:value=move || word.get()
                                            on:input=move |ev| word.set(event_target_value(&ev))
                                            placeholder="Enter word"
                                        />
                                        <div class="seed-underline"></div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>

                        <Show when=move || !error_message.with(String::is_empty)>
                            <p class="text-red-500 mt-2 text-center">{move || error_message.get()}</p>
                        </Show>

                        <div class="button-container-right">
                            <button
                                class="recovery-button-secondary anim-scale"
                                style="--anim-delay: 200ms"
                                on:click=handle_paste_phrase
                            >
                                "Paste Recovery Phrase"
                            </button>

                            <button
                                class="recovery-button anim-scale"
                                class:disabled=move || !is_complete.get() || is_restoring.get()
                                style="--anim-delay: 230ms"
                                on:click=handle_restore.clone()
                                disabled=move || !is_complete.get() || is_restoring.get()
                            >
                                {move || if is_restoring.get() {
                                    view! {
                                        <div class="spinner-container">
                                            <div class="spinner"></div>
                                            <span class="ml-2">"Restoring..."</span>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { "Restore Wallet" }.into_any()
                                }}
                            </button>
                        </div>

                        <div class="divider my-8 anim-fade" style="--anim-delay: 250ms">"OR"</div>

                        <div class="token-section w-full flex flex-col items-center anim-fly-up" style="--anim-delay: 270ms">
                            <h2 class="text-2xl font-bold mb-4 text-center">"Redeem Search Token"</h2>

                            <div class="token-input-container seed-container" style="display: block; padding: 1rem;">
                                <input
                                    type="text"
                                    class="word-text"
                                    prop:value=move || token_input.get()
                                    on:input=move |ev| token_input.set(event_target_value(&ev))
                                    placeholder="Enter your Cashu token"
                                />
                                <Show when=move || !token_error.with(String::is_empty)>
                                    <p class="text-red-500 mt-2 text-center">{move || token_error.get()}</p>
                                </Show>
                            </div>

                            <div class="button-container-right">
                                <button
                                    class="recovery-button-secondary"
                                    on:click=handle_paste_token
                                >
                                    "Paste Search Token"
                                </button>

                                <button
                                    class="recovery-button"
                                    class:disabled=move || token_input.with(|t| t.trim().is_empty()) || token_restoring.get()
                                    on:click=handle_token_redeem.clone()
                                    disabled=move || token_input.with(|t| t.trim().is_empty()) || token_restoring.get()
                                >
                                    {move || if token_restoring.get() {
                                        view! {
                                            <div class="spinner-container">
                                                <div class="spinner"></div>
                                                <span class="ml-2">"Redeeming..."</span>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { "Redeem Token" }.into_any()
                                    }}
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
