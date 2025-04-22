import { browser } from '$app/environment';
import { writable } from 'svelte/store';

// Since generateNewMnemonic has been removed in cashu-ts v2, we'll create a random seed instead
function generateRandomSeed() {
  // Generate a random 16-byte (128-bit) seed as a hex string
  const array = new Uint8Array(16);
  if (browser) {
    window.crypto.getRandomValues(array);
  } else {
    // Fallback for non-browser environments
    for (let i = 0; i < 16; i++) {
      array[i] = Math.floor(Math.random() * 256);
    }
  }
  return Array.from(array).map(b => b.toString(16).padStart(2, '0')).join('');
}

const defaultValue = generateRandomSeed();
/** @type {String} */
const initialValue = browser ? window.localStorage.getItem('seed') ?? defaultValue : defaultValue;

const seed = writable(initialValue);

seed.subscribe((value) => {
	if (browser) {
		window.localStorage.setItem('seed', value);
	}
});

export default seed;