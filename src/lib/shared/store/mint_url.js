import { browser } from "$app/environment";
import { writable } from "svelte/store";

// Use relative path for same-origin mint (e.g., "/mint" or just window.location.origin)
// Falls back to hardcoded URL during SSR or if localStorage is empty
const defaultValue = browser ? window.location.origin : "";
/** @type {string | undefined} */
const initialValue = browser
  ? (window.localStorage.getItem("mint_url") ?? defaultValue)
  : defaultValue;

const mint_url = writable(initialValue);

mint_url.subscribe((value) => {
  if (browser) {
    window.localStorage.setItem("mint_url", value);
  }
});

export default mint_url;
