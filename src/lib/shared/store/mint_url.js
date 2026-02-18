
import { browser } from '$app/environment';
import { writable } from 'svelte/store';


const defaultValue = "https://v2.athenut.com";
  /** @type {string | undefined} */
const initialValue = browser ? window.localStorage.getItem('mint_url') ?? defaultValue : defaultValue;

const mint_url = writable(initialValue);

mint_url.subscribe((value) => {
	if (browser) {
		window.localStorage.setItem('mint_url', value);
	}
});

export default mint_url;

