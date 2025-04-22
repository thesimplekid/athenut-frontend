Summary of Changes Made to Token Decoding in Recovery Page:
1. Imported getDecodedToken from cashu-ts package
2. Removed manual token decoding and fallback mechanism
3. Used getDecodedToken function for token decoding
4. Added validation for proof count vs. token value
5. Enhanced error handling for token format issues
6. Added warning for tokens with non-unit denominations

Fix for Keyset ID Issue in Search Page:
1. Modified the search page's handleSearch function to preserve the keyset ID from proofs
2. Added code to extract the keyset ID from a proof before redemption
3. Updated the keyset counter storage with the correct keyset ID instead of using "default"
4. Added imports for getKeysetCounts and setKeysetCounts functions
5. Added logging to help with debugging keyset ID usage during token redemption
