Summary of Changes Made to Token Decoding in Recovery Page:
1. Imported getDecodedToken from cashu-ts package
2. Removed manual token decoding and fallback mechanism
3. Used getDecodedToken function for token decoding
4. Added validation for proof count vs. token value
5. Enhanced error handling for token format issues
6. Added warning for tokens with non-unit denominations
