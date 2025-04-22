import * as cashu from "@cashu/cashu-ts";

// Print available exports
console.log('Available exports:', Object.keys(cashu));

// Test CashuMint and CashuWallet if they exist
if (cashu.CashuMint) {
  const mint = new cashu.CashuMint("https://example.com");
  console.log('CashuMint:', !!mint);
}

// Check MintQuoteState enum if it exists
if (cashu.MintQuoteState) {
  console.log('MintQuoteState.PAID:', cashu.MintQuoteState.PAID);
}

// Check getEncodedTokenV4 if it exists
if (cashu.getEncodedTokenV4) {
  console.log('getEncodedTokenV4 exists:', !!cashu.getEncodedTokenV4);
}