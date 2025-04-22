/** @type {import("@cashu/cashu-ts").Proof} */

export function getProofs() {
  try {
    const proofs = localStorage.getItem('proofs');
    if (!proofs) return [];
    
    const parsedProofs = JSON.parse(proofs);
    if (!Array.isArray(parsedProofs)) {
      console.error('getProofs - parsed proofs is not an array');
      return [];
    }
    
    // Filter out invalid proofs
    return parsedProofs.filter(proof => {
      // Check that proof is an object and has a valid amount property
      return proof && typeof proof === 'object' && typeof proof.amount === 'number';
    });
  } catch (error) {
    console.error('Error in getProofs:', error);
    return [];
  }
}

/** @type {(proofs: import("@cashu/cashu-ts").Proof[]) => Promise<void>} */
export function writeProofs(proofs) {
  localStorage.setItem('proofs', JSON.stringify(proofs));
}


export function getSpentProofs() {
  const proofs = localStorage.getItem('spent_proofs');
  const parsedProofs = proofs ? JSON.parse(proofs) : [];
  return Array.isArray(parsedProofs) ? parsedProofs : [];
}

/**
* @param {string[]} proofs Array of spent proof IDs
*/
export function writeSpentProofs(proofs) {
 localStorage.setItem('spent_proofs', JSON.stringify(proofs));
}

/**
* @param {Proof} proof The proof to add to spent proofs
*/
export function addSpentProof(proof) {
 const proofs = getSpentProofs();
 proofs.push(proof);
 writeSpentProofs(proofs);
}

export function getBalance() {
  try {
    const proofs = getProofs();
    console.log('getBalance - proofs:', proofs);
    
    // Make sure we have an array
    if (!Array.isArray(proofs)) {
      console.error('getBalance - proofs is not an array');
      return 0;
    }
    
    // Make sure all proofs have a valid amount property
    const total = proofs.reduce((total, proof) => {
      // If proof or proof.amount is undefined or not a number, use 0
      const amount = proof && typeof proof.amount === 'number' ? proof.amount : 0;
      return total + amount;
    }, 0);
    
    console.log('getBalance - calculated total:', total);
    return total;
  } catch (error) {
    console.error('Error in getBalance:', error);
    return 0;
  }
}

/**
 * Force a refresh of the balance by reading directly from localStorage
 * @returns {number} The current balance
 */
export function forceBalanceRefresh() {
  try {
    // Get fresh data directly from localStorage
    const rawProofs = localStorage.getItem('proofs');
    if (!rawProofs) return 0;
    
    const parsedProofs = JSON.parse(rawProofs);
    if (!Array.isArray(parsedProofs)) return 0;
    
    // Calculate balance
    const total = parsedProofs.reduce((total, proof) => {
      const amount = proof && typeof proof.amount === 'number' ? proof.amount : 0;
      return total + amount;
    }, 0);
    
    console.log('forceBalanceRefresh - calculated total:', total);
    return total;
  } catch (error) {
    console.error('Error in forceBalanceRefresh:', error);
    return 0;
  }
}

/**
 * Debug function to log the localStorage proofs and analyze them
 * @returns {void}
 */
export function debugProofs() {
  const rawProofs = localStorage.getItem('proofs');
  console.log('Raw proofs from localStorage:', rawProofs);
  
  try {
    const parsedProofs = JSON.parse(rawProofs || '[]');
    console.log('Parsed proofs:', parsedProofs);
    
    if (Array.isArray(parsedProofs)) {
      console.log('Number of proofs:', parsedProofs.length);
      console.log('Calculated balance:', parsedProofs.reduce((total, proof) => total + (proof.amount || 0), 0));
      
      // Check for missing amount properties
      const missingAmounts = parsedProofs.filter(proof => typeof proof.amount !== 'number');
      if (missingAmounts.length > 0) {
        console.log('Proofs with missing or invalid amount:', missingAmounts);
      }
    } else {
      console.log('Parsed proofs is not an array!');
    }
  } catch (error) {
    console.error('Error parsing proofs:', error);
  }
}


/**
* @returns {Object.<string, number>} Map of keyset IDs to their counts
*/
export function getKeysetCounts() {
 const counts = localStorage.getItem('keysetCounts');
 const parsedCounts = counts ? JSON.parse(counts) : {};
 return typeof parsedCounts === 'object' ? parsedCounts : {};
}

/**
* @param {Object.<string, number>} counts Map of keyset IDs to their counts
*/
export function setKeysetCounts(counts) {
 localStorage.setItem('keysetCounts', JSON.stringify(counts));
}

/**
* @typedef {Object} MintQuote
* @property {string} id - The quote identifier
* @property {number} amount - The amount in sats
* @property {number} expiry
* @property {string} date - The date the quote was created
* @property {string} mint - The mint URL
* @property {string} invoice - The bolt11 invoice
 * @property {'pending' | 'paid' | 'expired'} state - The current state of the quote
*/

/**
* @returns {MintQuote[]} List of mint quotes
*/
export function getPendingQuotes() {
 const quotes = localStorage.getItem('pendingQuotes');
 const parsedQuotes = quotes ? JSON.parse(quotes) : [];
 return Array.isArray(parsedQuotes) ? parsedQuotes : [];
}

/**
* @param {MintQuote[]} quotes List of mint quotes
*/
export function writePendingQuotes(quotes) {
 localStorage.setItem('pendingQuotes', JSON.stringify(quotes));
}

/** 
 * Updates the state of a pending quote in localStorage
 * @param {string} quoteId - The ID of the quote to update
 * @property {'pending' | 'paid' | 'expired'} state - The current state of the quote
 */
export function updateQuoteState(quoteId, newState) {
    const quotes = getPendingQuotes();
    const updatedQuotes = quotes.map(quote => 
        quote.id === quoteId 
            ? { ...quote, state: newState }
            : quote
    );

    writePendingQuotes(updatedQuotes);
}

/**
* @param {string} id Quote ID to remove
* @returns {void}
*/
export function removePendingQuote(id) {
 const quotes = getPendingQuotes();
 const filteredQuotes = quotes.filter(quote => quote.id !== id);
 writePendingQuotes(filteredQuotes);
}

/**
* @param {MintQuote} quote The quote to add
* @returns {void}
*/
export function addPendingQuote(quote) {
 const quotes = getPendingQuotes();
 quotes.push(quote);
 writePendingQuotes(quotes);
}
