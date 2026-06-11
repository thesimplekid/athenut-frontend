//! Thin wrappers around window.localStorage / sessionStorage.
//!
//! On non-wasm targets (native unit tests) a thread-local in-memory map
//! stands in for the browser storage.

pub use backend::*;

#[cfg(target_arch = "wasm32")]
mod backend {
    use web_sys::Storage;

    fn local_storage() -> Option<Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    fn session_storage() -> Option<Storage> {
        web_sys::window()?.session_storage().ok().flatten()
    }

    pub fn get(key: &str) -> Option<String> {
        local_storage()?.get_item(key).ok().flatten()
    }

    /// Returns false when the value could not be persisted (storage
    /// unavailable or quota exceeded).
    pub fn set(key: &str, value: &str) -> bool {
        local_storage()
            .map(|storage| storage.set_item(key, value).is_ok())
            .unwrap_or(false)
    }

    pub fn remove(key: &str) {
        if let Some(storage) = local_storage() {
            let _ = storage.remove_item(key);
        }
    }

    pub fn session_get(key: &str) -> Option<String> {
        session_storage()?.get_item(key).ok().flatten()
    }

    pub fn session_set(key: &str, value: &str) {
        if let Some(storage) = session_storage() {
            let _ = storage.set_item(key, value);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use std::cell::RefCell;
    use std::collections::HashMap;

    thread_local! {
        static LOCAL: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
        static SESSION: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    }

    pub fn get(key: &str) -> Option<String> {
        LOCAL.with(|s| s.borrow().get(key).cloned())
    }

    pub fn set(key: &str, value: &str) -> bool {
        LOCAL.with(|s| s.borrow_mut().insert(key.to_string(), value.to_string()));
        true
    }

    pub fn remove(key: &str) {
        LOCAL.with(|s| s.borrow_mut().remove(key));
    }

    pub fn session_get(key: &str) -> Option<String> {
        SESSION.with(|s| s.borrow().get(key).cloned())
    }

    pub fn session_set(key: &str, value: &str) {
        SESSION.with(|s| s.borrow_mut().insert(key.to_string(), value.to_string()));
    }
}
