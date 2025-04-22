import { browser } from '$app/environment';
import { writable } from 'svelte/store';
import { generateMnemonic } from '@scure/bip39';
import { wordlist } from '@scure/bip39/wordlists/english';

// Use generateMnemonic from @scure/bip39 to create a mnemonic phrase
function generateWalletMnemonic() {
  return generateMnemonic(wordlist);
}

const defaultValue = generateWalletMnemonic();
/** @type {String} */
// Initialize with seed from localStorage or generate a new one if not found
const initialValue = browser ? window.localStorage.getItem('seed') ?? defaultValue : defaultValue;

const seed = writable(initialValue);

// Subscribe to seed changes and automatically save to local storage
seed.subscribe((value) => {
	if (browser) {
		window.localStorage.setItem('seed', value);
	}
});

export default seed;
