//! Shared reactive state: theme, toast notifications, wallet balance.

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::{storage, wallet};

const THEME_KEY: &str = "theme";

#[derive(Clone, Copy)]
pub struct AppState {
    /// "light" or "dark"
    pub theme: RwSignal<String>,
    pub toast: RwSignal<Option<String>>,
    toast_seq: RwSignal<u64>,
    pub balance: RwSignal<u64>,
}

impl AppState {
    pub fn new() -> Self {
        let theme = storage::get(THEME_KEY).unwrap_or_else(|| "light".to_string());
        Self {
            theme: RwSignal::new(theme),
            toast: RwSignal::new(None),
            toast_seq: RwSignal::new(0),
            balance: RwSignal::new(0),
        }
    }

    pub fn is_dark(&self) -> bool {
        self.theme.with(|t| t == "dark")
    }

    pub fn toggle_theme(&self) {
        self.theme.update(|t| {
            *t = if t == "dark" { "light" } else { "dark" }.to_string();
        });
    }

    pub fn show_toast(&self, message: impl Into<String>) {
        self.show_toast_for(message, 3000);
    }

    pub fn show_toast_for(&self, message: impl Into<String>, duration_ms: u32) {
        let seq = self.toast_seq.get_untracked() + 1;
        self.toast_seq.set(seq);
        self.toast.set(Some(message.into()));
        let toast = self.toast;
        let toast_seq = self.toast_seq;
        spawn_local(async move {
            TimeoutFuture::new(duration_ms).await;
            if toast_seq.get_untracked() == seq {
                toast.set(None);
            }
        });
    }

    pub fn refresh_balance(&self) {
        let balance = self.balance;
        spawn_local(async move {
            balance.set(wallet::balance().await);
        });
    }
}

/// Apply theme changes to the document and persist them.
pub fn apply_theme_effect(state: AppState) {
    Effect::new(move |_| {
        let theme = state.theme.get();
        storage::set(THEME_KEY, &theme);
        if let Some(root) = document().document_element() {
            let class_list = root.class_list();
            if theme == "dark" {
                let _ = class_list.add_1("dark");
            } else {
                let _ = class_list.remove_1("dark");
            }
        }
    });
}
