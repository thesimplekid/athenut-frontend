use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{Meta, Title};

use crate::components::{Footer, Navbar};

const FAQS: [(&str, &str); 6] = [
    (
        "What is Athenut?",
        "Athenut is a privacy-preserving web search powered by Kagi and Cashu, allowing you to perform premium searches while maintaining your privacy.",
    ),
    (
        "How do searches work?",
        "Each search costs 1 token. You can purchase tokens using Bitcoin Lightning Network payments, and these tokens are stored locally in your browser.",
    ),
    (
        "How do I top up my account?",
        "You can top up your account by visiting the Top Up page, selecting the number of searches you want to purchase, and paying the Lightning invoice that appears.",
    ),
    (
        "Are my searches private?",
        "Yes! Athenut uses Cashu tokens for payments, which are completely private. Your search history is not stored on any servers.",
    ),
    (
        "What happens to unused searches?",
        "Your searches remain in your browser wallet until you use them. They don't expire and are stored locally on your device.",
    ),
    (
        "How do I backup my searches?",
        "Your unused searches are stored as tokens in your browser's local storage. You can back them up by saving the 12-word recovery phrase found in the backup page.",
    ),
];

#[component]
pub fn FaqPage() -> impl IntoView {
    let content_ready = RwSignal::new(false);
    let expanded = RwSignal::new(Vec::<usize>::new());

    spawn_local(async move {
        TimeoutFuture::new(100).await;
        content_ready.set(true);
    });

    let toggle_question = move |index: usize| {
        expanded.update(|expanded| {
            if let Some(pos) = expanded.iter().position(|&i| i == index) {
                expanded.remove(pos);
            } else {
                expanded.push(index);
            }
        });
    };

    view! {
        <Title text="FAQ - Athenut"/>
        <Meta name="description" content="Frequently asked questions about Athenut's privacy-preserving web search."/>

        <div class="page-faq min-h-dvh flex flex-col relative">
            <Navbar/>

            <main class="flex-grow flex flex-col items-center px-4 pt-24 pb-8">
                <Show when=move || content_ready.get()>
                    <div class="w-full max-w-3xl">
                        <h1 class="text-4xl font-bold mb-2 text-center text-balance anim-fly-up" style="--anim-delay: 90ms">
                            "Frequently Asked Questions"
                        </h1>

                        <p class="text-xl text-gray-600 text-center mb-8 text-pretty anim-fly-up" style="--anim-delay: 140ms">
                            "Find answers to common questions about using Athenut."
                        </p>

                        <div class="faq-container anim-fade" style="--anim-delay: 170ms">
                            {FAQS.iter().enumerate().map(|(index, (question, answer))| {
                                let is_expanded = move || expanded.with(|e| e.contains(&index));
                                view! {
                                    <div class="faq-item">
                                        <button
                                            class="faq-question"
                                            class:expanded=is_expanded
                                            on:click=move |_| toggle_question(index)
                                        >
                                            <span>{*question}</span>
                                            <svg
                                                class="arrow-icon"
                                                class:rotated=is_expanded
                                                xmlns="http://www.w3.org/2000/svg"
                                                viewBox="0 0 24 24"
                                                width="24"
                                                height="24"
                                            >
                                                <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                                            </svg>
                                        </button>
                                        <Show when=is_expanded>
                                            <div class="faq-answer text-pretty">
                                                {*answer}
                                            </div>
                                        </Show>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </Show>
            </main>

            <Footer/>
        </div>
    }
}
