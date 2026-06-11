use cdk::nuts::MintQuoteState;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_navigate;
use qrcode::render::svg;
use qrcode::QrCode;

use crate::components::{copy_to_clipboard, Footer, Navbar};
use crate::state::AppState;
use crate::wallet;

const SEARCH_OPTIONS: [u64; 6] = [1, 5, 10, 20, 35, 50];
const POLL_INTERVAL_MS: u32 = 3000;
const POLL_MAX_ATTEMPTS: u32 = 100;

#[derive(Clone, PartialEq)]
struct InvoiceRow {
    id: String,
    amount: u64,
    invoice: String,
    state: MintQuoteState,
    expired: bool,
    date_ms: Option<f64>,
}

async fn invoice_rows() -> Vec<InvoiceRow> {
    let now_secs = js_sys::Date::now() / 1000.0;
    let dates = wallet::quote_dates();
    wallet::pending_quotes()
        .await
        .into_iter()
        .map(|quote| InvoiceRow {
            amount: quote.amount.map(u64::from).unwrap_or_default(),
            invoice: quote.request.clone(),
            state: quote.state,
            expired: (quote.expiry as f64) < now_secs,
            date_ms: dates.get(&quote.id).copied(),
            id: quote.id,
        })
        .collect()
}

fn time_ago(date_ms: Option<f64>) -> String {
    let Some(date_ms) = date_ms else {
        return String::new();
    };
    let minutes = ((js_sys::Date::now() - date_ms) / 60000.0).floor() as i64;
    if minutes < 60 {
        format!("{minutes} minutes ago")
    } else {
        format!("{} hours ago", minutes / 60)
    }
}

fn qr_svg(data: &str) -> String {
    QrCode::new(data.as_bytes())
        .map(|code| code.render::<svg::Color>().min_dimensions(300, 300).build())
        .unwrap_or_default()
}

#[component]
pub fn TopupPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let navigate = use_navigate();

    let invoice_data = RwSignal::new(String::new());
    let selected_searches = RwSignal::new(0u64);
    let invoice_amount = RwSignal::new(0u64);
    let is_loading = RwSignal::new(false);
    let content_ready = RwSignal::new(false);
    let pending_invoices = RwSignal::new(Vec::<InvoiceRow>::new());
    let refreshing_quote = RwSignal::new(None::<String>);

    spawn_local(async move {
        state.balance.set(wallet::balance().await);
        pending_invoices.set(invoice_rows().await);
        TimeoutFuture::new(100).await;
        content_ready.set(true);
    });

    let handle_top_up = {
        let navigate = navigate.clone();
        move |searches: u64| {
            let navigate = navigate.clone();
            selected_searches.set(searches);
            is_loading.set(true);
            spawn_local(async move {
                let quote = match wallet::create_mint_quote(searches).await {
                    Ok(quote) => quote,
                    Err(e) => {
                        is_loading.set(false);
                        state.show_toast(format!("Error topping up: {e}"));
                        return;
                    }
                };
                // Hide the spinner once we have the invoice
                is_loading.set(false);
                invoice_data.set(quote.request.clone());
                invoice_amount.set(wallet::invoice_amount_sats(&quote.request).unwrap_or(0));
                pending_invoices.set(invoice_rows().await);

                // Poll until the invoice is paid, then mint the proofs
                let mut paid = false;
                for _ in 0..POLL_MAX_ATTEMPTS {
                    match wallet::check_mint_quote(&quote.id).await {
                        Ok(checked) if checked.state == MintQuoteState::Paid => {
                            paid = true;
                            break;
                        }
                        Ok(checked) if checked.state == MintQuoteState::Issued => return,
                        _ => {}
                    }
                    TimeoutFuture::new(POLL_INTERVAL_MS).await;
                }

                if !paid {
                    state.show_toast("Invoice was not paid in time");
                    return;
                }

                match wallet::mint_proofs(&quote.id).await {
                    Ok(_) => {
                        state.refresh_balance();
                        pending_invoices.set(invoice_rows().await);
                        navigate("/", Default::default());
                    }
                    Err(e) => state.show_toast(format!("Error minting: {e}")),
                }
            });
        }
    };

    let handle_refresh = move |quote_id: String| {
        refreshing_quote.set(Some(quote_id.clone()));
        spawn_local(async move {
            match wallet::check_mint_quote(&quote_id).await {
                Ok(checked) => match checked.state {
                    MintQuoteState::Paid => match wallet::mint_proofs(&quote_id).await {
                        Ok(count) => {
                            state.show_toast(format!("Minted {count} searches"));
                            state.refresh_balance();
                        }
                        Err(e) => {
                            let lower = e.to_lowercase();
                            if lower.contains("expired") {
                                state.show_toast("Quote Expired");
                            } else {
                                state.show_toast(format!("Error minting, please try again: {e}"));
                            }
                        }
                    },
                    MintQuoteState::Issued => {
                        state.show_toast("Quote already minted");
                        state.refresh_balance();
                    }
                    _ => {
                        let now_secs = js_sys::Date::now() / 1000.0;
                        if (checked.expiry as f64) < now_secs {
                            state.show_toast("Quote Expired");
                        } else {
                            state.show_toast("Quote is not paid");
                        }
                    }
                },
                Err(e) => state.show_toast(format!("Error refreshing quote: {e}")),
            }
            pending_invoices.set(invoice_rows().await);
            TimeoutFuture::new(1000).await;
            refreshing_quote.set(None);
        });
    };

    view! {
        <Title text="Athenut"/>
        <Meta name="description" content="privacy-preserving web search powered by Kagi and Cashu."/>

        <div class="page-topup">
            <div class="page-wrapper">
                <Navbar/>

                <main class="main-content">
                    <Show when=move || content_ready.get()>
                        <div class="content-container anim-fade">
                            <div class="header-section anim-fly-up" style="--anim-delay: 90ms">
                                <h1 class="page-title">"Top Up"</h1>

                                <div class="balance-display">
                                    <span class="balance-label">"You have"</span>
                                    <span class="balance-amount tabular-nums">{move || state.balance.get()}</span>
                                    <span class="balance-label">"searches left"</span>
                                    <button
                                        class="refresh-balance-button"
                                        on:click=move |_| state.refresh_balance()
                                        aria-label="Refresh balance"
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                                        </svg>
                                    </button>
                                </div>

                                <p class="tagline">
                                    "Zap your account with sats to unlock more premium searches."
                                </p>
                            </div>

                            <div class="qr-container anim-scale" style="--anim-delay: 130ms">
                                {
                                    let handle_top_up = handle_top_up.clone();
                                    move || {
                                        let handle_top_up = handle_top_up.clone();
                                        if is_loading.get() {
                                            view! {
                                                <div class="spinner-container">
                                                    <div class="spinner"></div>
                                                </div>
                                            }.into_any()
                                        } else if !invoice_data.with(String::is_empty) {
                                            view! {
                                                <div class="qr-section anim-scale">
                                                    <div class="qr-info">
                                                        "Purchasing " <strong>{move || selected_searches.get()} " searches"</strong>
                                                        " for " <strong>{move || invoice_amount.get()} " sats"</strong>
                                                    </div>
                                                    <div class="qr-wrapper" inner_html=move || qr_svg(&invoice_data.get())></div>
                                                    <button
                                                        type="button"
                                                        class="copy-invoice-button"
                                                        on:click=move |_| {
                                                            copy_to_clipboard(&invoice_data.get_untracked());
                                                            state.show_toast("Invoice copied to clipboard.");
                                                        }
                                                        aria-label="Copy invoice"
                                                    >
                                                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                                                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                                                        </svg>
                                                        "Copy Invoice"
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="top-up-grid">
                                                    {SEARCH_OPTIONS.iter().enumerate().map(|(i, &search_count)| {
                                                        let handle_top_up = handle_top_up.clone();
                                                        let label = if search_count == 1 { "Search" } else { "Searches" };
                                                        view! {
                                                            <button
                                                                on:click=move |_| handle_top_up(search_count)
                                                                class="top-up-button anim-scale"
                                                                style=format!("--anim-delay: {}ms", 180 + i * 35)
                                                            >
                                                                <div class="search-count">{search_count}</div>
                                                                <div class="search-label">{label}</div>
                                                            </button>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                }
                            </div>

                            // Transaction History Table
                            <Show when=move || !pending_invoices.with(Vec::is_empty)>
                                <div class="transaction-history-container anim-fly-up" style="--anim-delay: 260ms">
                                    <h2 class="history-title">"Recent Invoices"</h2>
                                    <div class="transaction-table">
                                        {move || pending_invoices.get().into_iter().map(|row| {
                                            let (badge_class, badge_label) = match row.state {
                                                MintQuoteState::Issued => ("paid", "Paid"),
                                                MintQuoteState::Unpaid if row.expired => ("pending", "Expired"),
                                                MintQuoteState::Paid => ("paid", "Paid"),
                                                _ => ("pending", "Unpaid"),
                                            };
                                            let can_refresh = row.state != MintQuoteState::Issued && !row.expired;
                                            let unit_label = if row.amount == 1 { "search" } else { "searches" };
                                            let copy_invoice = row.invoice.clone();
                                            let refresh_id = row.id.clone();
                                            let spinning_id = row.id.clone();
                                            view! {
                                                // Transaction row
                                                <div class="transaction-row">
                                                    <div class="amount-cell tabular-nums">
                                                        {row.amount} " " {unit_label}
                                                    </div>
                                                    <div class="time-cell">
                                                        {time_ago(row.date_ms)}
                                                    </div>
                                                    <div class="status-cell">
                                                        <span class=format!("status-badge {badge_class}")>
                                                            {badge_label}
                                                        </span>
                                                    </div>
                                                    <div class="action-buttons">
                                                        <button
                                                            class="copy-button"
                                                            on:click=move |_| {
                                                                copy_to_clipboard(&copy_invoice);
                                                                state.show_toast("Invoice copied to clipboard.");
                                                            }
                                                            aria-label="Copy invoice"
                                                        >
                                                            <svg xmlns="http://www.w3.org/2000/svg" class="copy-icon" viewBox="0 0 24 24" width="24" height="24">
                                                                <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 4v12a2 2 0 002 2h8a2 2 0 002-2V7.242a2 2 0 00-.602-1.43L16.083 2.57A2 2 0 0014.685 2H10a2 2 0 00-2 2z M16 18v2a2 2 0 01-2 2H6a2 2 0 01-2-2V9a2 2 0 012-2h2"/>
                                                            </svg>
                                                        </button>
                                                        <Show when=move || can_refresh>
                                                            {
                                                                let refresh_id = refresh_id.clone();
                                                                let spinning_id = spinning_id.clone();
                                                                view! {
                                                                    <button
                                                                        class="refresh-button"
                                                                        class:spinning=move || refreshing_quote.with(|r| r.as_deref() == Some(spinning_id.as_str()))
                                                                        on:click=move |_| handle_refresh(refresh_id.clone())
                                                                        aria-label="Refresh quote"
                                                                    >
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="refresh-icon" viewBox="0 0 24 24" width="24" height="24">
                                                                            <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                                                                        </svg>
                                                                    </button>
                                                                }
                                                            }
                                                        </Show>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            </Show>
                        </div>
                    </Show>
                </main>

                <Footer/>
            </div>
        </div>
    }
}
