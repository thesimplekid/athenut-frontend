<script>
  import { goto } from "$app/navigation";
  import seed from "$lib/shared/store/wallet";
  import mint_url from "$lib/shared/store/mint_url";
  import {
    CashuMint,
    CashuWallet,
    CheckStateEnum,
    getDecodedToken,
  } from "@cashu/cashu-ts";
  import Footer from "../../components/Footer.svelte";
  import Toast from "../../components/Toast.svelte";
  import {
    getProofs,
    writeProofs,
    getKeysetCounts,
    setKeysetCounts,
  } from "$lib/shared/utils";
  import { onMount } from "svelte";
  import { theme } from "$lib/stores/theme";
  import Navbar from "../../components/Navbar.svelte";
  import { showToast } from "$lib/stores/toast";

  let words = Array(12).fill("");
  let errorMessage = "";
  let tokenInput = "";
  let tokenError = "";
  let isRestoring = false; // Track if wallet restoration is in progress
  let tokenRestoring = false; // Track if token redemption is in progress

  /** @type {CashuWallet|null} */
  let wallet = null;

  $: isComplete = words.every((word) => word.trim().length > 0);

  // Initialize wallet instance
  async function initializeWallet() {
    const mint = new CashuMint($mint_url);
    let keysets = await mint.getKeys();
    let matchingKeyset = keysets.keysets.find((key) => key.unit === "xsr");

    // Convert the seed string to a proper 256-bit seed using SHA-256
    const encoder = new TextEncoder();
    const data = encoder.encode($seed);
    const hashBuffer = await crypto.subtle.digest("SHA-256", data);
    const seedBuffer = new Uint8Array(hashBuffer);

    console.log("Seed buffer created with length:", seedBuffer.length, "bytes");

    wallet = new CashuWallet(mint, {
      unit: "xsr",
      keys: matchingKeyset,
      // Try both parameter names that might be accepted
      seed: seedBuffer,
      bip39seed: seedBuffer,
      // Configure wallet to only use denomination 1
      preferredDenominations: [1],
      // Set denomination target to 1 to ensure we only use denomination 1
      denominationTarget: 1,
    });
    return wallet;
  }

  /**
   * Checks the state of an array of proofs
   * @param {Array<import("@cashu/cashu-ts").Proof>} proofs - Vector of Proof objects to check
   * @returns {Promise<void>}
   */
  async function checkProofState(proofs) {
    if (!wallet) {
      await initializeWallet();
    }

    if (!wallet) {
      throw new Error("Failed to initialize wallet");
    }

    // In cashu-ts v2, checkProofsSpent has been replaced by checkProofsStates
    let proofStates = await wallet.checkProofsStates(proofs);
    // Filter the original proofs array to remove spent ones
    let unspentProofs = proofs.filter(
      (proof, i) => proofStates[i]?.state !== CheckStateEnum.SPENT,
    );
    console.log("Restored ", unspentProofs.length, " proofs");
    let current_proofs = getProofs();
    // Create combined list with unique proofs based on secret
    const combinedList = [
      ...current_proofs,
      ...unspentProofs.filter(
        (newProof) =>
          !current_proofs.some(
            (existingProof) => existingProof.secret === newProof.secret,
          ),
      ),
    ];

    writeProofs(combinedList);
  }

  async function handleRestore() {
    try {
      // Set the restoration state to true to show the loading spinner
      isRestoring = true;
      errorMessage = "";

      // Set the seed value in the store
      // This will automatically save the seed to local storage via the seed store subscription
      $seed = words.join(" ");

      // Initialize wallet with the proper seed
      if (!wallet) {
        await initializeWallet();
      }

      if (!wallet) {
        throw new Error("Failed to initialize wallet");
      }

      let empty_batches = 0;
      let start_counter = 0;
      let end_counter = 99;
      let totalProofsRestored = 0;
      let totalAmountRestored = 0;

      while (empty_batches < 3) {
        console.log("Restoring wallet with seed phrase");
        console.log(
          "Wallet has seed property set:",
          wallet._seed ? "Yes (" + wallet._seed.length + " bytes)" : "No",
        );
        // The restore method may require seed to derive outputs
        // Parameters: start, end, callback, secret_indices, customIds
        let restores = await wallet.restore(
          start_counter,
          end_counter,
          null,
          null,
          null,
        );
        console.log(
          "Received ",
          restores.proofs.length,
          " signatures from mint",
        );

        if (restores.proofs.length > 0) {
          // Calculate amount from this batch of proofs
          const batchAmount = restores.proofs.reduce(
            (sum, proof) => sum + (proof.amount || 0),
            0,
          );
          totalAmountRestored += batchAmount;
          totalProofsRestored += restores.proofs.length;
        }

        start_counter += 100;
        end_counter += 100;
        if (restores.proofs.length === 0) {
          empty_batches += 1;
        }
        await checkProofState(restores.proofs);
      }

      // Show success message with amount restored
      if (totalProofsRestored > 0) {
        showToast(
          `Restored ${totalProofsRestored} proofs (${totalAmountRestored} sats)`,
          5000,
        );

        // Navigate to home page after a short delay
        setTimeout(() => {
          goto("/");
        }, 1000);
      } else {
        showToast("No proofs were found to restore", 3000);
      }

      console.log(
        `Wallet restoration completed. Restored ${totalProofsRestored} proofs with ${totalAmountRestored} sats`,
      );
    } catch (error) {
      console.error("Restore error:", error);
      errorMessage = `Failed to restore wallet: ${error.message}`;

      // Show error in toast as well
      showToast(`Restore failed: ${error.message}`, 5000);
    } finally {
      // Reset the loading state when complete or on error
      isRestoring = false;
    }
  }

  async function handleRecover() {
    try {
      const clipboardText = await navigator.clipboard.readText();
      const pastedWords = clipboardText.trim().split(/\s+/);

      if (pastedWords.length !== 12) {
        errorMessage = "Please paste exactly 12 words";
        return;
      }

      words = pastedWords;
    } catch (error) {
      errorMessage =
        "Unable to access clipboard. Please grant clipboard permission.";
      console.error("Clipboard error:", error);
    }
  }

  async function handleTokenRedeem() {
    try {
      if (!tokenInput.trim()) {
        tokenError = "Please enter a valid token";
        return;
      }

      // Set loading state to true
      tokenRestoring = true;
      tokenError = "";

      // Initialize the wallet with the seed from local storage
      if (!wallet) {
        await initializeWallet();
      }

      if (!wallet) {
        throw new Error("Failed to initialize wallet");
      }

      // Simple validation - make sure the token has a valid format
      // Cashu tokens typically start with "cashuA" and have a valid encoding
      if (!tokenInput.trim().startsWith("cashu")) {
        console.error("Token doesn't start with 'cashu'");
        throw new Error(
          "Invalid token format. Token should start with 'cashu'.",
        );
      }

      // Log the token format for debugging (first 10 chars only for privacy)
      console.log(
        "Token input first 10 chars:",
        tokenInput.substring(0, 10) + "...",
      );

      // Get the keys information for proper counter tracking
      const keysets = await wallet.mint.getKeys();
      const matchingKeyset = keysets.keysets.find((key) => key.unit === "xsr");

      if (!matchingKeyset) {
        throw new Error("Could not find matching keyset in mint");
      }

      // Decode token to determine its total value
      // We need to do this first to create the right output amount configuration
      console.log("Decoding token to determine value...");

      // Use getDecodedToken from cashu-ts to decode the token
      // This function parses the token format and extracts the data
      const decodedToken = getDecodedToken(tokenInput);
      console.log("Decoded token structure:", decodedToken);

      // Calculate total token value
      const totalTokenValue = decodedToken.proofs.reduce(
        (sum, proof) => sum + (proof.amount || 0),
        0,
      );

      console.log("Total token value:", totalTokenValue);
      console.log("Number of proofs in token:", decodedToken.proofs.length);

      if (totalTokenValue <= 0) {
        throw new Error("Could not determine token value or token is empty");
      }

      // Verify that the number of outputs matches the number of proofs for unit denominations
      if (totalTokenValue !== decodedToken.proofs.length) {
        console.warn(
          `Token value (${totalTokenValue}) does not match number of proofs (${decodedToken.proofs.length}). ` +
            "This may indicate a non-unit denomination token.",
        );
      }

      // Create an array of 1s with length equal to total token value
      const unitDenominations = Array(totalTokenValue).fill(1);
      console.log(`Creating ${unitDenominations.length} proofs of value 1`);

      // Set up the receive options with unit denominations
      const receiveOptions = {
        outputAmounts: {
          sendAmounts: unitDenominations,
        },
      };

      // Now receive the token with our specified output amounts
      console.log("Receiving token with unit denominations...");
      const proofs = await wallet.receive(decodedToken, receiveOptions);

      // Log the result for debugging
      console.log("Token receive result - proofs array:", proofs);
      console.log(
        "Number of proofs received:",
        Array.isArray(proofs) ? proofs.length : "Not an array",
      );

      // Check if we received valid proofs
      if (Array.isArray(proofs) && proofs.length > 0) {
        // Update keyset counter after receiving tokens
        let keyset_counts = getKeysetCounts();
        let keyset_count = keyset_counts[matchingKeyset.id] || 0;
        let new_count = keyset_count + proofs.length;
        keyset_counts[matchingKeyset.id] = new_count;
        setKeysetCounts(keyset_counts);

        // Check proof state and save to storage
        await checkProofState(proofs);

        // Calculate total amount redeemed
        const totalAmount = proofs.reduce(
          (sum, proof) => sum + (proof.amount || 0),
          0,
        );

        // Verify all proofs have value 1
        const allUnit = proofs.every((proof) => proof.amount === 1);
        console.log("All proofs have unit value:", allUnit);

        // Show success message
        showToast(
          `Redeemed ${proofs.length} proofs (${totalAmount} sats)`,
          5000,
        );

        // Clear input and any errors
        tokenInput = "";
        tokenError = "";

        // Navigate to home page after a short delay
        setTimeout(() => {
          goto("/");
        }, 1000);
      } else {
        console.error("Received invalid result from wallet.receive", proofs);
        tokenError = "Failed to process token. No valid proofs received.";
        showToast(tokenError, 3000);
      }
    } catch (error) {
      console.error("Token redemption error:", error);

      // Provide more helpful error messages based on the error type
      if (
        error.message?.includes("Invalid token") ||
        error.message?.includes("Token version")
      ) {
        tokenError =
          "Invalid token format. Please check that you've entered a valid Cashu token.";
      } else if (error.message?.includes("Token is not supported")) {
        tokenError = "This token format is not supported.";
      } else if (error.message?.includes("spent")) {
        tokenError = "This token has already been spent.";
      } else {
        tokenError = `Failed to redeem token: ${error.message}`;
      }

      showToast(tokenError, 5000);
    } finally {
      // Reset loading state when complete or on error
      tokenRestoring = false;
    }
  }

  onMount(() => {
    const savedTheme = localStorage.getItem("theme") || "light";
    theme.set(savedTheme);
    if (savedTheme === "dark") {
      document.documentElement.classList.add("dark");
    }
  });

  // Subscribe to theme changes
  $: if ($theme === "dark") {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }
</script>

<svelte:head>
  <title>Athenut</title>
  <meta
    name="description"
    content="privacy-preserving web search powered by Kagi and Cashu."
  />
</svelte:head>

<div
  class="min-h-screen flex flex-col text-gray-800 bg-white relative dark:bg-[var(--bg-primary)] dark:text-white"
>
  <Navbar />

  <main
    class="flex-grow flex flex-col justify-start items-center px-4 py-8 dark:bg-[var(--bg-primary)]"
  >
    <h1
      class="text-4xl font-bold mb-2 text-center text-gray-800 dark:text-white"
    >
      Recovery
    </h1>

    <p class="text-xl text-gray-600 dark:text-gray-500 mb-6">
      Enter your 12-word recovery phrase to restore your searches.
    </p>

    <div class="seed-container">
      {#each words as word, i}
        <div class="seed-word">
          <span class="word-number">{i + 1}</span>
          <input
            type="text"
            class="word-text"
            bind:value={words[i]}
            placeholder="Enter word"
          />
          <div class="underline"></div>
        </div>
      {/each}
    </div>

    {#if errorMessage}
      <p class="text-red-500 mt-2 text-center">{errorMessage}</p>
    {/if}

    <button class="recovery-button-secondary mb-4" on:click={handleRecover}>
      Paste Recovery Phrase
    </button>

    <button
      class="recovery-button {!isComplete || isRestoring ? 'disabled' : ''}"
      on:click={handleRestore}
      disabled={!isComplete || isRestoring}
    >
      {#if isRestoring}
        <div class="spinner-container">
          <div class="spinner"></div>
          <span class="ml-2">Restoring...</span>
        </div>
      {:else}
        Restore Wallet
      {/if}
    </button>

    <div class="divider my-8">OR</div>

    <div class="token-section w-full max-w-800px flex flex-col items-center">
      <h2
        class="text-2xl font-bold mb-4 text-center text-gray-800 dark:text-white"
      >
        Redeem Search Token
      </h2>

      <div
        class="token-input-container seed-container"
        style="display: block; padding: 1rem;"
      >
        <input
          type="text"
          class="word-text"
          bind:value={tokenInput}
          placeholder="Enter your Cashu token"
        />
        {#if tokenError}
          <p class="text-red-500 mt-2 text-center">{tokenError}</p>
        {/if}
      </div>

      <button
        class="recovery-button-secondary mb-4"
        on:click={async () => {
          try {
            tokenInput = await navigator.clipboard.readText();
            tokenError = "";
          } catch (error) {
            tokenError =
              "Unable to access clipboard. Please grant clipboard permission.";
            console.error("Clipboard error:", error);
          }
        }}
      >
        Paste Search Token
      </button>

      <button
        class="recovery-button mt-4 {!tokenInput.trim() || tokenRestoring
          ? 'disabled'
          : ''}"
        on:click={handleTokenRedeem}
        disabled={!tokenInput.trim() || tokenRestoring}
      >
        {#if tokenRestoring}
          <div class="spinner-container">
            <div class="spinner"></div>
            <span class="ml-2">Redeeming...</span>
          </div>
        {:else}
          Redeem Token
        {/if}
      </button>
    </div>
  </main>

  <Footer />
  <Toast />
</div>

<style>
  .seed-container {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1.5rem;
    padding: 2rem;
    background: #f0f2f5;
    border-radius: 24px;
    margin-bottom: 2rem;
    width: 100%;
    max-width: 800px;
  }

  .seed-word {
    position: relative;
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 0;
  }

  .word-number {
    color: #666;
    font-size: 1.1rem;
    font-weight: 500;
    min-width: 1.5rem;
  }

  .word-text {
    color: var(--text-primary, #1f2937);
    font-size: 1.2rem;
    font-weight: 500;
    background: transparent;
    border: none;
    width: 100%;
  }

  .word-text:focus {
    outline: none;
  }

  .word-text::placeholder {
    color: #666;
    opacity: 0.5;
  }

  .underline {
    position: absolute;
    bottom: 0;
    left: 2.5rem;
    right: 0;
    height: 1px;
    background: #d1d5db;
  }

  .recovery-button {
    background-color: #1a1a1a;
    color: white;
    border: none;
    border-radius: 9999px;
    padding: 16px 32px;
    font-size: 18px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    box-shadow: 0 2px 4px rgba(26, 26, 26, 0.2);
    width: 100%;
    max-width: 300px;
    position: relative;
    overflow: hidden;
  }

  .recovery-button:hover {
    background-color: #2a2a2a;
    box-shadow: 0 4px 8px rgba(26, 26, 26, 0.3);
  }

  .recovery-button:focus {
    outline: none;
    box-shadow:
      0 0 0 2px rgba(255, 255, 255, 0.5),
      0 4px 8px rgba(26, 26, 26, 0.3);
  }

  .recovery-button::before {
    content: "";
    position: absolute;
    top: -2px;
    left: -2px;
    right: -2px;
    bottom: -2px;
    background: linear-gradient(45deg, #4285f4, #34a853, #fbbc05, #ea4335);
    z-index: -1;
    filter: blur(5px);
    opacity: 0;
    transition: opacity 0.3s;
  }

  .recovery-button:hover::before {
    opacity: 0.5;
  }

  .recovery-button.disabled {
    opacity: 0.5;
    cursor: not-allowed;
    background-color: #666;
  }

  .recovery-button.disabled:hover {
    background-color: #666;
    box-shadow: 0 2px 4px rgba(26, 26, 26, 0.2);
  }

  .recovery-button.disabled::before {
    display: none;
  }

  .mb-4 {
    margin-bottom: 1rem;
  }

  .recovery-button-secondary {
    background-color: transparent;
    color: #666;
    border: none;
    border-radius: 9999px;
    padding: 16px 32px;
    font-size: 18px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    width: 100%;
    max-width: 300px;
  }

  .recovery-button-secondary:hover {
    color: #1a1a1a;
  }

  @media (max-width: 640px) {
    .seed-container {
      grid-template-columns: repeat(2, 1fr);
      padding: 1rem;
      gap: 0.75rem;
    }

    .seed-word {
      padding: 0.25rem 0;
    }

    .word-number {
      font-size: 0.9rem;
      min-width: 1.2rem;
    }

    .word-text {
      font-size: 1rem;
    }

    .underline {
      left: 2rem;
    }

    .main-heading {
      font-size: 2rem;
    }

    .recovery-button {
      width: 80%;
      max-width: 250px;
      padding: 14px 28px;
      font-size: 16px;
    }

    .recovery-button-secondary {
      width: 80%;
      max-width: 250px;
      padding: 14px 28px;
      font-size: 16px;
    }
  }

  :global(.dark) {
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --bg-hover: #3a3a3a;
    --text-primary: #ffffff;
    --text-secondary: #a0aec0;
    --text-hover: #f0f0f0;
    --border-color: #333;
  }

  :global(.dark) .seed-container {
    background-color: #2d2d2d !important;
  }

  :global(.dark) .word-number {
    color: #a0aec0 !important;
  }

  :global(.dark) .word-text {
    color: #ffffff !important;
  }

  :global(.dark) .main-heading {
    color: #ffffff !important;
  }

  :global(.dark) .recovery-button {
    background-color: #2d2d2d;
  }

  :global(.dark) .recovery-button-secondary {
    color: #a0aec0;
  }

  :global(.dark) .recovery-button-secondary:hover {
    color: #ffffff;
  }

  :global(.dark) .back-button {
    color: #ffffff !important;
  }

  /* Add dark mode text color for headings */
  :global(.dark) h1 {
    color: #ffffff;
  }

  /* If you also want to ensure the description text below is properly colored */
  :global(.dark) .text-gray-600 {
    color: #a0aec0;
  }

  .divider {
    width: 100%;
    max-width: 800px;
    text-align: center;
    position: relative;
    color: #6b7280;
  }

  .divider::before,
  .divider::after {
    content: "";
    position: absolute;
    top: 50%;
    width: 45%;
    height: 1px;
    background: #d1d5db;
  }

  .divider::before {
    left: 0;
  }

  .divider::after {
    right: 0;
  }

  .token-input-container {
    width: 100%;
    max-width: 800px;
    padding: 2rem;
    background: #f0f2f5;
    border-radius: 24px;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .token-input {
    width: 100%;
    padding: 1rem;
    background: transparent;
    border: 1px solid #333;
    border-radius: 12px;
    color: white;
    font-size: 1.1rem;
  }

  .token-input:focus {
    outline: none;
    border-color: #4285f4;
  }

  /* Dark mode styles */
  :global(.dark) .token-input-container {
    background-color: #2d2d2d;
  }

  :global(.dark) .token-input {
    color: #ffffff;
    border-color: #333;
  }

  :global(.dark) .divider {
    color: #a0aec0;
  }

  :global(.dark) .divider::before,
  :global(.dark) .divider::after {
    background: #333;
  }

  :global(body) {
    background-color: white;
  }

  :global(.dark body) {
    background-color: var(--bg-primary);
  }

  :global(html.dark) {
    background-color: var(--bg-primary);
  }

  /* Remove or modify these styles that might be affecting the text color */
  :global(.dark) h1 {
    color: #ffffff; /* Only apply this in dark mode */
  }

  /* Add explicit light mode styles */
  h1,
  h2 {
    color: #1f2937; /* gray-800 */
  }

  /* Keep dark mode styles */
  :global(.dark) h1,
  :global(.dark) h2 {
    color: #ffffff;
  }

  /* Restore original dark mode styles */
  :global(.dark) .underline {
    background: #4b5563;
  }

  :global(.dark) .divider::before,
  :global(.dark) .divider::after {
    background: #4b5563;
  }

  :global(.dark) .divider {
    color: #6b7280;
  }

  /* Spinner styles */
  .spinner-container {
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top: 2px solid white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  .ml-2 {
    margin-left: 0.5rem;
  }

  /* Dark mode spinner */
  :global(.dark) .spinner {
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-top: 2px solid white;
  }
</style>
