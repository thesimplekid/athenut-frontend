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
const initialValue = browser ? window.localStorage.getItem('seed') ?? defaultValue : defaultValue;

const seed = writable(initialValue);

seed.subscribe((value) => {
	if (browser) {
		window.localStorage.setItem('seed', value);
	}
});

export default seed;