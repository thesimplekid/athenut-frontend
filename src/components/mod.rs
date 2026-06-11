pub mod footer;
pub mod navbar;
pub mod toast;

pub use footer::Footer;
pub use navbar::Navbar;
pub use toast::Toast;

use wasm_bindgen_futures::JsFuture;

/// Copy text to the system clipboard.
pub fn copy_to_clipboard(text: &str) {
    if let Some(window) = web_sys::window() {
        let promise = window.navigator().clipboard().write_text(text);
        leptos::task::spawn_local(async move {
            let _ = JsFuture::from(promise).await;
        });
    }
}

/// Read text from the system clipboard.
pub async fn read_clipboard() -> Result<String, String> {
    let window = web_sys::window().ok_or("no window")?;
    let promise = window.navigator().clipboard().read_text();
    let value = JsFuture::from(promise)
        .await
        .map_err(|_| "Unable to access clipboard. Please grant clipboard permission.")?;
    Ok(value.as_string().unwrap_or_default())
}
