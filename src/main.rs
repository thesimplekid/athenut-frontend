mod components;
mod db;
mod pages;
mod state;
mod storage;
mod wallet;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::provide_meta_context;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::state::{apply_theme_effect, AppState};

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    let state = AppState::new();
    provide_context(state);
    apply_theme_effect(state);

    spawn_local(async move {
        wallet::import_legacy_proofs().await;
        state.refresh_balance();
        wallet::finish_legacy_migration().await;
        // Recover proofs left PendingSpent by an interrupted search.
        wallet::reclaim_pending_proofs().await;
        state.refresh_balance();
    });

    // The `storage` event fires in every *other* tab when localStorage
    // changes, so this keeps the balance live while another tab spends,
    // mints or restores.
    let storage_handle = window_event_listener(leptos::ev::storage, move |ev| {
        if ev.key().is_none() || ev.key().as_deref() == Some(db::STORAGE_KEY) {
            state.refresh_balance();
        }
    });
    on_cleanup(move || storage_handle.remove());

    view! {
        <Router>
            <Routes fallback=pages::home::HomePage>
                <Route path=path!("/") view=pages::home::HomePage/>
                <Route path=path!("/search") view=pages::search::SearchPage/>
                <Route path=path!("/topup") view=pages::topup::TopupPage/>
                <Route path=path!("/backup") view=pages::backup::BackupPage/>
                <Route path=path!("/recovery") view=pages::recovery::RecoveryPage/>
                <Route path=path!("/faq") view=pages::faq::FaqPage/>
            </Routes>
        </Router>
        // Mounted once here so toasts show on every page.
        <components::Toast/>
    }
}
