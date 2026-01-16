<script>
  // This implementation strictly enforces using only proofs with denomination 1 for all operations
  // by explicitly setting outputAmounts.sendAmounts to be an array of 1's when minting proofs
  import { onMount } from "svelte";
  import SvgQR from "@svelte-put/qr/svg/QR.svelte";
  import { copyToClipboard } from "@svelte-put/copy";
  import bolt11Decoder from "light-bolt11-decoder";
  import { goto } from "$app/navigation";
  import mint_url from "$lib/shared/store/mint_url";
  import {
    addPendingQuote,
    getBalance,
    getKeysetCounts,
    getPendingQuotes,
    getProofs,
    setKeysetCounts,
    updateQuoteState,
    writeProofs,
    debugProofs,
    forceBalanceRefresh,
  } from "$lib/shared/utils";
  import { CashuMint, CashuWallet, MintQuoteState } from "@cashu/cashu-ts";
  import Footer from "../../components/Footer.svelte";
  import { showToast } from "$lib/stores/toast";
  import Toast from "../../components/Toast.svelte";
  import seed from "$lib/shared/store/wallet";
  import { generateMnemonic, mnemonicToSeedSync } from "@scure/bip39";
  import { wordlist } from "@scure/bip39/wordlists/english";
  import { theme } from "$lib/stores/theme";
  import Navbar from "../../components/Navbar.svelte";
  import { fade, fly, scale } from 'svelte/transition';
  import { quintOut, elasticOut } from 'svelte/easing';

  // Function to generate a mnemonic (matching the one in wallet.js)
  function generateWalletMnemonic() {
    return generateMnemonic(wordlist);
  }

  // AmountPreference type has been removed in cashu-ts v2

  /** @type {string} */
  let data = "";

  /** @type {number} */
  let balance = 0;

  /**
   * @typedef {Object} InfoResult
   * @property {string} mint
   */

  /** @type {number} */
  let selectedSearches = 0;

  /** @type {number} */
  let invoice_amount = 0;

  let pendingInvoices = getPendingQuotes();

  async function getInfo() {
    // API call removed to focus on UI fixes only
    // let info = await fetch(`${PUBLIC_API_URL}/info`, {}).then((r) => r.json());
    // $mint_url = info.mint;
  }

  let isLoading = false;
  let contentReady = false;

  onMount(async () => {
    // Debug the proofs to see what's going on
    console.log("Topup Page - onMount");
    debugProofs();
    setTimeout(() => contentReady = true, 100);

    // Initialize the wallet balance - use both approaches
    balance = await getBalance();
    console.log("Balance after getBalance():", balance);

    // Force a direct refresh from localStorage
    balance = forceBalanceRefresh();
    console.log("Balance after forceBalanceRefresh():", balance);

    // Log the actual proofs for debugging
    const proofs = getProofs();
    console.log("Actual proofs array:", proofs);

    if ($mint_url != undefined) {
      await getInfo();
    }

    // Create the storage event handler
    const handleStorageChange = (e) => {
      if (e.key === "proofs") {
        console.log("Storage event - proofs changed");
        balance = forceBalanceRefresh();
        console.log("Updated balance:", balance);
      }
    };

    // Add storage listener to update balance when proofs change
    window.addEventListener("storage", handleStorageChange);

    return () => {
      // Clean up event listener when component is destroyed
      window.removeEventListener("storage", handleStorageChange);
    };
  });

  /**
   * Extracts the amount in satoshis from a BOLT11 lightning invoice
   * @param {string} invoice - The BOLT11 encoded lightning invoice
   * @returns {number} The amount in satoshis (converted from millisatoshis)
   * @throws {Error} If the invoice cannot be decoded or is invalid
   * @example
   */
  export const getAmountFromInvoice = (invoice) => {
    // Decode the invoice
    const decodedInvoice = bolt11Decoder.decode(invoice);
    // Extract the amount from the decoded invoice and convert from msats to sats
    const amount = Number(decodedInvoice.sections[2].value / 1000);
    return Math.ceil(amount);
  };

  /**
   * Creates and initializes a Cashu wallet
   * @param {string} mintUrl - The URL of the mint
   * @param {string} seed - The wallet seed (mnemonic string)
   * @returns {Promise<{wallet: CashuWallet, keys: any}>} The initialized wallet and its keys
   */
  async function initializeWallet(mintUrl, seed) {
    const mint = new CashuMint(mintUrl);
    const keysets = await mint.getKeys();
    const matchingKeyset = keysets.keysets.find((key) => key.unit === "xsr");

    if (!matchingKeyset) {
      console.error("No matching keyset found", keysets);
      throw new Error("No matching keyset found");
    }

    // Log whether the keyset has an ID
    if (!matchingKeyset.id) {
      console.warn("Warning: The keyset is missing an ID", matchingKeyset);
    } else {
      console.log("Using keyset with id:", matchingKeyset.id);
    }

    // Convert the seed string (mnemonic) to a 256-bit seed using SHA-256
    let seedBytes = mnemonicToSeedSync(seed);

    // Create the wallet with the bip39seed parameter
    const wallet = new CashuWallet(mint, {
      unit: "xsr",
      keys: matchingKeyset,
      // Use the properly sized seed
      bip39seed: seedBytes,
      // Set denomination target to 1 to ensure we only use denomination 1
      denominationTarget: 1,
    });

    return { wallet, keys: matchingKeyset };
  }

  /**
   * @param {number} searches
   */
  async function handleTopUp(searches) {
    if (mint_url != null) {
      console.log("Attempting to top up for searches ", searches);
      selectedSearches = searches;
      isLoading = true;

      try {
        // Check if we need to generate a seed
        if (!$seed) {
          console.log("No seed found, generating new mnemonic...");
          $seed = generateWalletMnemonic();
          console.log("Generated new seed (mnemonic)");
        }

        console.log(
          "Initializing wallet with seed:",
          $seed ? $seed.substring(0, 3) + "..." : "undefined",
        );
        const { wallet, keys } = await initializeWallet($mint_url, $seed);

        // Create the mint quote
        /** @type {import("@cashu/cashu-ts").MintQuoteResponse} */
        let mintQuote = await wallet.createMintQuote(searches);
        isLoading = false; // Hide the spinner once we have the invoice

        const quote = {
          id: mintQuote.quote,
          amount: searches,
          date: new Date().toISOString(),
          mint: $mint_url,
          expiry: mintQuote.expiry,
          invoice: mintQuote.request,
          state: "pending",
        };

        addPendingQuote(quote);
        pendingInvoices = getPendingQuotes();

        data = mintQuote.request;
        invoice_amount = getAmountFromInvoice(data);

        // Rest of polling and minting logic...
        const mintQuoteChecked = await pollMintQuote(wallet, mintQuote.quote);

        if (mintQuoteChecked.state === MintQuoteState.PAID) {
          let keyset_counts = getKeysetCounts();

          // Get the stored counter, handling the case where keys.id might be undefined
          const keysetId = keys && keys.id ? keys.id : "default";
          let keyset_count = keyset_counts[keysetId] || 0;
          console.log(
            "Using keyset count:",
            keyset_count,
            "for keyset ID:",
            keysetId,
          );

          const options = {
            // Set outputAmounts to ensure only denomination 1 proofs
            outputAmounts: {
              // Create an array of 1's with length equal to the amount being minted
              sendAmounts: Array(searches).fill(1),
              keepAmounts: Array(searches).fill(1),
            },
            keysetId: keysetId, // Use the keyset ID if available, otherwise use "default"
            counter: keyset_count, // Counter for blind signatures
            // Not using these options for basic minting:
            // outputData: undefined,  // For additional data
            // p2pk: undefined,        // For P2PK locks
            // proofsWeHave: undefined, // For splitting existing proofs
            // pubkey: undefined        // For pubkey operations
          };

          console.log("Minting proofs with options:", JSON.stringify(options));
          // In cashu-ts v2, mintTokens is now mintProofs and returns proofs directly, not in an object
          const proofs = await wallet.mintProofs(
            searches,
            mintQuote.quote,
            options,
          );

          console.log("Successfully minted proofs:", proofs.length);
          // Verify that all proofs have a denomination of 1
          const allDenom1 = proofs.every((proof) => proof.amount === 1);
          console.log("All proofs have denomination 1:", allDenom1);
          if (!allDenom1) {
            console.warn("Warning: Some proofs do not have denomination 1");
          }

          // Update the keyset count with how many proofs we created
          let new_count = keyset_count + proofs.length;
          keyset_counts[keysetId] = new_count;
          setKeysetCounts(keyset_counts);

          let current_proofs = getProofs();
          const combinedList = [...current_proofs, ...proofs];
          writeProofs(combinedList);
          updateQuoteState(mintQuote.quote, "paid");

          // Update the balance right after proofs are written
          balance = forceBalanceRefresh();
          console.log("Updated balance after mintProofs:", balance);

          pendingInvoices = getPendingQuotes();
          goto("/");
        }
      } catch (error) {
        console.error("Error details:", error);

        if (
          error.message
            ?.toLowerCase()
            .includes("cannot create deterministic messages without seed")
        ) {
          // Handle the specific error we're fixing
          showToast(
            "Wallet initialization error. Please refresh the page and try again.",
          );
          console.error(
            "Seed format issue - this should be fixed with our code update.",
          );
        } else {
          // For any other error
          showToast("Error topping up: " + (error.message || "Unknown error"));
        }

        console.error("Error while topping up: ", error);
      }
    }
    isLoading = false;
  }

  /**
   * @param {string} quoteId
   */
  async function handleRefresh(quoteId) {
    console.log("Refreshing quote:", quoteId);
    let mintQuote = pendingInvoices.find((quote) => quote.id === quoteId);
    if (!mintQuote) {
      throw new Error("Could not find mint quote");
    }

    // Get button reference once at the start
    const button = document.querySelector(`[data-quote-id="${quoteId}"]`);
    if (button) {
      button.classList.add("spinning");
    }

    try {
      // Check if we need to generate a seed
      if (!$seed) {
        console.log("No seed found, generating new mnemonic...");
        $seed = generateWalletMnemonic();
        console.log("Generated new seed (mnemonic)");
      }

      console.log(
        "Initializing wallet with seed:",
        $seed ? $seed.substring(0, 3) + "..." : "undefined",
      );
      const { wallet, keys } = await initializeWallet($mint_url, $seed);
      let keyset_counts = getKeysetCounts();

      // Get the stored counter, handling the case where keys.id might be undefined
      const keysetId = keys && keys.id ? keys.id : "default";
      let keyset_count = keyset_counts[keysetId] || 0;
      console.log(
        "Using keyset count:",
        keyset_count,
        "for keyset ID:",
        keysetId,
      );

      const options = {
        // Set outputAmounts to ensure only denomination 1 proofs
        outputAmounts: {
          // Create an array of 1's with length equal to the amount being minted
          sendAmounts: Array(mintQuote.amount).fill(1),
        },
        keysetId: keysetId, // Use the keyset ID if available, otherwise use "default"
        counter: keyset_count, // Counter for blind signatures
        // Not using these options for basic minting:
        // outputData: undefined,  // For additional data
        // p2pk: undefined,        // For P2PK locks
        // proofsWeHave: undefined, // For splitting existing proofs
        // pubkey: undefined        // For pubkey operations
      };

      console.log("Minting proofs with options:", JSON.stringify(options));
      // In cashu-ts v2, mintTokens is now mintProofs and returns proofs directly, not in an object
      let proofs = await wallet.mintProofs(mintQuote.amount, quoteId, options);

      console.log("Successfully minted proofs:", proofs.length);
      // Verify that all proofs have a denomination of 1
      const allDenom1 = proofs.every((proof) => proof.amount === 1);
      console.log("All proofs have denomination 1:", allDenom1);
      if (!allDenom1) {
        console.warn("Warning: Some proofs do not have denomination 1");
      }
      let new_count = keyset_count + proofs.length;
      keyset_counts[keysetId] = new_count;
      setKeysetCounts(keyset_counts);

      let current_proofs = getProofs();
      const combinedList = [...current_proofs, ...proofs];
      writeProofs(combinedList);
      updateQuoteState(mintQuote.id, "paid");

      // Update the balance right after proofs are written
      balance = forceBalanceRefresh();
      console.log("Updated balance after handleRefresh:", balance);
    } catch (error) {
      console.error("Error details:", error);

      if (error.message?.toLowerCase().includes("expired")) {
        updateQuoteState(quoteId, "expired");
        showToast("Quote Expired");
      } else if (error.message?.toLowerCase().includes("pending")) {
        showToast("Quote is not paid");
        if (
          mintQuote.expiry &&
          mintQuote.expiry < Math.floor(Date.now() / 1000)
        ) {
          updateQuoteState(quoteId, "expired");
        }
      } else if (error.message?.toLowerCase().includes("already signed")) {
        // HACK: Make this smarter
        console.log("Already signed");
        // Use the same generateWalletMnemonic if needed
        if (!$seed) {
          console.log("No seed found, generating new mnemonic...");
          $seed = generateWalletMnemonic();
          console.log("Generated new seed (mnemonic)");
        }
        const { wallet, keys } = await initializeWallet($mint_url, $seed);

        let keyset_counts = getKeysetCounts();

        // Get the stored counter, handling the case where keys.id might be undefined
        const keysetId = keys && keys.id ? keys.id : "default";
        let keyset_count = keyset_counts[keysetId] || 0;
        let new_count = keyset_count + 10;
        keyset_counts[keysetId] = new_count;
        setKeysetCounts(keyset_counts);
        showToast("Error minting, please try again");
      } else if (
        error.message
          ?.toLowerCase()
          .includes("cannot create deterministic messages without seed")
      ) {
        // Handle the specific error we're fixing
        showToast(
          "Wallet initialization error. Please refresh the page and try again.",
        );
        console.error(
          "Seed format issue - this should be fixed with our code update.",
        );
      } else {
        // For any other error
        showToast(
          "Error refreshing quote: " + (error.message || "Unknown error"),
        );
      }

      console.error("Error while refreshing quote: ", error);
    } finally {
      // Always update pending invoices and balance
      pendingInvoices = getPendingQuotes();
      balance = forceBalanceRefresh();
      console.log("Final balance after handleRefresh:", balance);

      // Always remove spinning class after a delay
      if (button) {
        setTimeout(() => button.classList.remove("spinning"), 1000);
      }
    }
  }

  /**
   * Polls the mint quote state until it's paid or maximum attempts are reached
   * @param {CashuWallet} wallet - The initialized wallet
   * @param {string} quote - The mint quote identifier to check
   * @param {number} [interval=3000] - Polling interval in milliseconds
   * @param {number} [maxAttempts=100] - Maximum number of polling attempts
   * @returns {Promise<import("@cashu/cashu-ts").MintQuoteResponse>} The checked mint quote object when paid
   * @throws {Error} If mint quote is not paid within max attempts
   */
  async function pollMintQuote(
    wallet,
    quote,
    interval = 3000,
    maxAttempts = 100,
  ) {
    let attempts = 0;

    while (attempts < maxAttempts) {
      const mintQuoteChecked = await wallet.checkMintQuote(quote);
      if (mintQuoteChecked.state === MintQuoteState.PAID) {
        return mintQuoteChecked;
      }
      await new Promise((resolve) => setTimeout(resolve, interval));
      attempts++;
    }

    throw new Error("Mint quote not paid within the allowed attempts.");
  }

  function getTimeAgo(date) {
    const diff = new Date().getTime() - new Date(date).getTime();
    const minutes = Math.floor(diff / 60000);
    if (minutes < 60) return `${minutes} minutes ago`;
    return `${Math.floor(minutes / 60)} hours ago`;
  }

  /**
   * @param {string} text
   */
  export function customCopy(text) {
    copyToClipboard(text);
    showToast("Invoice copied to clipboard.");
  }
</script>

<svelte:head>
  <title>Athenut</title>
  <meta
    name="description"
    content="privacy-preserving web search powered by Kagi and Cashu."
  />
</svelte:head>

<!-- Update the main container div and add Navbar -->
<div class="page-wrapper">
  <Navbar {balance} />

  <main class="main-content">
    {#if contentReady}
      <div class="content-container" in:fade={{ duration: 600, easing: quintOut }}>
        <div class="header-section" in:fly={{ y: 20, duration: 500, delay: 200, easing: quintOut }}>
          <h1 class="page-title">Top Up</h1>

          <div class="balance-display">
            <span class="balance-label">You have</span>
            <span class="balance-amount tabular-nums">{balance}</span>
            <span class="balance-label">searches left</span>
            <button
              class="refresh-balance-button"
              on:click={() => {
                balance = forceBalanceRefresh();
                console.log("Balance refreshed manually:", balance);
              }}
              aria-label="Refresh balance"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                />
              </svg>
            </button>
          </div>

          <p class="tagline">
            Zap your account with sats to unlock more premium searches.
          </p>
        </div>

        <div class="qr-container" in:scale={{ duration: 400, delay: 400, easing: quintOut }}>
          {#if isLoading}
            <div class="spinner-container">
              <div class="spinner"></div>
            </div>
          {:else if data !== ""}
            <div class="qr-section" transition:scale={{ duration: 300, easing: quintOut }}>
              <div class="qr-info">
                Purchasing <strong>{selectedSearches} searches</strong> for <strong>{invoice_amount} sats</strong>
              </div>
              <div class="qr-wrapper">
                <SvgQR {data} width="300" height="300" />
              </div>
              <button
                type="button"
                class="copy-invoice-button"
                on:click={() => customCopy(data)}
                aria-label="Copy invoice"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
                Copy Invoice
              </button>
            </div>
          {:else}
            <div class="top-up-grid">
              {#each [1, 5, 10, 20, 35, 50] as search_count, i}
                <button
                  on:click={() => handleTopUp(search_count)}
                  class="top-up-button"
                  in:scale={{ duration: 300, delay: 500 + (i * 50), start: 0.8, easing: elasticOut }}
                >
                  <div class="search-count">{search_count}</div>
                  <div class="search-label">{search_count === 1 ? 'Search' : 'Searches'}</div>
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Transaction History Table -->
        {#if pendingInvoices.length > 0}
          <div class="transaction-history-container" in:fly={{ y: 30, duration: 500, delay: 900, easing: quintOut }}>
            <h2 class="history-title">Recent Invoices</h2>
            <div class="transaction-table">
          {#each pendingInvoices.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime()) as quote}
            <!-- Transaction row -->
            <div class="transaction-row">
              <div class="amount-cell tabular-nums">
                {quote.amount}
                {quote.amount === 1 ? "search" : "searches"}
              </div>
              <div class="time-cell">
                {getTimeAgo(quote.date)}
              </div>
              <div class="status-cell">
                <span
                  class="status-badge {quote.state === 'paid'
                    ? 'Paid'
                    : 'Pending'}"
                >
                  {#if quote.state === "paid"}
                    Paid
                  {:else if quote.state === "pending"}
                    Unpaid
                  {:else}
                    Expired
                  {/if}
                </span>
              </div>
              <div class="action-buttons">
                <button
                  class="copy-button"
                  on:click={() => customCopy(quote.invoice)}
                  aria-label="Copy invoice"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="copy-icon"
                    viewBox="0 0 24 24"
                    width="24"
                    height="24"
                  >
                    <path
                      fill="none"
                      stroke="currentColor"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M8 4v12a2 2 0 002 2h8a2 2 0 002-2V7.242a2 2 0 00-.602-1.43L16.083 2.57A2 2 0 0014.685 2H10a2 2 0 00-2 2z M16 18v2a2 2 0 01-2 2H6a2 2 0 01-2-2V9a2 2 0 012-2h2"
                    />
                  </svg>
                </button>
                {#if quote.state === "pending" || quote.state === undefined}
                  <button
                    class="refresh-button"
                    on:click={() => handleRefresh(quote.id)}
                    aria-label="Refresh quote"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="refresh-icon"
                      viewBox="0 0 24 24"
                      width="24"
                      height="24"
                    >
                      <path
                        fill="none"
                        stroke="currentColor"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                      />
                    </svg>
                  </button>
                {/if}
              </div>
            </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </main>

  <Footer />
  <Toast />
</div>

<style>
  .page-wrapper {
    min-height: 100dvh;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-primary);
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    padding: 96px 1rem 2rem;
    min-height: calc(100vh - 80px);
  }

  .content-container {
    width: 100%;
    max-width: 1000px;
    margin: 0 auto;
  }

  .header-section {
    text-align: center;
    margin-bottom: 3rem;
    max-width: 800px;
    margin-left: auto;
    margin-right: auto;
  }

  .page-title {
    font-size: clamp(2rem, 5vw, 3rem);
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 1.5rem;
  }

  .balance-display {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    font-size: clamp(1.25rem, 3vw, 1.75rem);
    margin-bottom: 1rem;
  }

  .balance-label {
    color: var(--text-secondary);
    font-weight: 400;
  }

  .balance-amount {
    color: var(--text-primary);
    font-weight: 400;
    font-size: 1em;
  }

  .tagline {
    font-size: clamp(1rem, 2vw, 1.25rem);
    color: var(--text-secondary);
    font-weight: 400;
  }

  .top-up-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr); /* Always 3 columns on desktop */
    gap: 1rem;
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
    padding: 0;
    box-sizing: border-box;
  }

  .top-up-button {
    background: rgba(255, 255, 255, 0.8);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    color: var(--text-primary);
    border: 1px solid rgba(0, 0, 0, 0.06);
    border-radius: 16px;
    padding: 24px 20px;
    cursor: pointer;
    /* Only animate transform and border, not backdrop-filter */
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1), border-color 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    min-height: 120px;
  }

  :global(.dark) .top-up-button {
    background: rgba(45, 45, 45, 0.8);
    border-color: rgba(255, 255, 255, 0.08);
  }

  .search-count {
    font-size: 1.5rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .search-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .top-up-button:hover {
    transform: translateY(-1px) scale(1.01);
    border-color: rgba(0, 0, 0, 0.1);
    background: rgba(255, 255, 255, 0.95);
  }

  :global(.dark) .top-up-button:hover {
    border-color: rgba(255, 255, 255, 0.12);
    background: rgba(45, 45, 45, 0.95);
  }

  .top-up-button:active {
    transform: translateY(0) scale(0.99);
    transition: all 0.1s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .spinner-container {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    justify-content: center;
    align-items: center;
    background: rgba(255, 255, 255, 0.95);
  }

  .spinner-container::before {
    content: "";
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
    opacity: 0.2;
    mix-blend-mode: soft-light;
    pointer-events: none;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(26, 26, 26, 0.1);
    border-top: 3px solid #1a1a1a;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    position: relative;
    z-index: 1;
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  .header-container {
    position: relative;
    margin-bottom: 2rem;
    width: 100%;
    max-width: 800px;
    text-align: center;
  }

  /* Removed unused button and container styles */

  .copy-invoice-button {
    background: #1a1a1a;
    color: white;
    border: none;
    border-radius: 12px;
    padding: 12px 24px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }

  .copy-invoice-button:hover {
    background: #2a2a2a;
    transform: translateY(-2px);
  }

  .copy-invoice-button:active {
    transform: translateY(0);
  }

  :global(.dark) .copy-invoice-button {
    background: #ffffff;
    color: #1a1a1a;
    border: none;
  }

  :global(.dark) .copy-invoice-button:hover {
    background: #f0f0f0;
  }

  .qr-container {
    min-height: 300px; /* Reduced height */
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    justify-content: flex-start; /* Changed from center to flex-start */
    align-items: center;
    position: relative;
  }

  .qr-info {
    text-align: center;
    margin-bottom: 1rem;
    font-size: 1.2rem;
    font-weight: 600;
    color: #333333;
  }

  /* Dark mode styles */
  :global(.dark) {
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --bg-hover: #3a3a3a;
    --text-primary: #ffffff;
    --text-secondary: #a0aec0;
    --text-hover: #f0f0f0;
    --border-color: #333;
  }

  /* Removed problematic global text color overrides that were causing white text on white background */

  :global(.dark) .top-up-button {
    background-color: #2d2d2d;
    color: #ffffff;
  }

  :global(.dark) .top-up-button:hover {
    background-color: #3a3a3a;
  }

  :global(.dark) .transaction-table {
    background-color: #2d2d2d;
  }

  .refresh-balance-button {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 8px;
    margin-left: 8px;
    border-radius: 8px;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .refresh-balance-button:hover {
    background-color: rgba(102, 126, 234, 0.1);
    color: #667eea;
    transform: rotate(180deg);
  }

  .refresh-balance-button:active {
    transform: rotate(180deg) scale(0.9);
  }

  .qr-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
  }

  .qr-info {
    text-align: center;
    font-size: 1.125rem;
    color: var(--text-primary);
    font-weight: 400;
  }

  .qr-info strong {
    font-weight: 700;
    color: #667eea;
  }

  /* QR code specific styles */
  .qr-wrapper {
    background-color: white;
    padding: 1rem;
    border-radius: 8px;
    border: 1px solid #e5e7eb;
  }

  .qr-wrapper :global(svg) {
    color: black !important;
    fill: black !important;
  }

  /* Ensure QR code stays visible in dark mode */
  :global(.dark) .qr-wrapper {
    background-color: white !important;
    border: 1px solid #4b5563;
  }

  :global(.dark) .qr-wrapper :global(svg) {
    color: black !important;
    fill: black !important;
  }

  /* Add these transaction table styles */
  .transaction-history-container {
    width: 100%;
    max-width: 800px;
    margin: 2rem auto 0;
  }

  .history-title {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .transaction-table {
    background-color: #f0f2f5;
    border-radius: 12px;
    overflow: hidden;
  }

  .transaction-row {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr 1fr;
    padding: 1rem;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    align-items: center;
  }

  .transaction-row:last-child {
    border-bottom: none;
  }

  .amount-cell {
    font-weight: 600;
  }

  .time-cell {
    color: #6b7280;
    font-size: 0.9rem;
  }

  .status-cell {
    text-align: center;
  }

  .status-badge {
    display: inline-flex;
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .status-badge.Pending {
    background-color: #fff7ed;
    color: #c2410c;
    border: 1px solid #f97316;
  }

  .status-badge.Paid {
    background-color: #f0fdf4;
    color: #166534;
    border: 1px solid #22c55e;
  }

  .action-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  .copy-button,
  .refresh-button {
    background: none;
    border: none;
    padding: 0.5rem;
    cursor: pointer;
    color: #6b7280;
    transition: color 0.2s;
  }

  .copy-button:hover,
  .refresh-button:hover {
    color: #1a1a1a;
  }

  .copy-icon,
  .refresh-icon {
    width: 20px;
    height: 20px;
  }

  /* Dark mode styles for the transaction table */
  :global(.dark) .transaction-table {
    background-color: #2d2d2d;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  :global(.dark) .transaction-row {
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  :global(.dark) .amount-cell {
    color: #ffffff;
  }

  :global(.dark) .time-cell {
    color: #9ca3af;
  }

  :global(.dark) .copy-button,
  :global(.dark) .refresh-button {
    color: #9ca3af;
  }

  :global(.dark) .copy-button:hover,
  :global(.dark) .refresh-button:hover {
    color: #ffffff;
  }

  /* Status badges remain the same in dark mode for better visibility */

  /* Status badge styles */
  .status-badge {
    display: inline-flex;
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .status-badge.Pending {
    background-color: #fff7ed;
    color: #c2410c;
    border: 1px solid #f97316;
  }

  .status-badge.Paid {
    background-color: #f0fdf4;
    color: #166534;
    border: 1px solid #22c55e;
  }

  /* Dark mode styles */
  :global(.dark) .status-badge.Pending {
    background-color: #451a03;
    color: #fdba74;
    border: 1px solid #f97316;
  }

  :global(.dark) .status-badge.Paid {
    background-color: #052e16;
    color: #86efac;
    border: 1px solid #22c55e;
  }

  /* Add this to your dark mode styles */
  :global(.dark) .history-title {
    color: #ffffff;
  }

  /* Add these dark mode styles */
  :global(.dark) .qr-info {
    color: #ffffff !important;
  }

  /* Duplicate styles removed - using the ones defined earlier */


  :global(body) {
    overflow-x: hidden;
    background-color: #ffffff;
  }

  :global(body.dark) {
    background-color: #1a1a1a;
  }

  .min-h-screen {
    width: 100vw;
    max-width: 100%;
    overflow-x: hidden;
  }

  .top-up-button {
    min-height: 100px;
    height: auto;
    padding: 1rem;
    width: 100%;
  }

  @media (max-width: 640px) {
    .top-up-grid {
      grid-template-columns: repeat(2, 1fr);
      gap: 0.5rem;
      padding: 0 0.5rem;
    }

    .top-up-button {
      min-height: 80px;
      padding: 0.75rem;
    }
  }

  /* Add these mobile-specific styles at the bottom of your style block */
  @media (max-width: 640px) {
    .transaction-row {
      grid-template-columns: 1fr 1fr; /* Reduce to 2 columns */
      grid-template-areas:
        "amount status"
        "time actions";
      gap: 0.5rem;
      padding: 0.75rem;
    }

    .amount-cell {
      grid-area: amount;
      font-size: 0.9rem;
    }

    .time-cell {
      grid-area: time;
      font-size: 0.8rem;
    }

    .status-cell {
      grid-area: status;
      text-align: right;
    }

    .action-buttons {
      grid-area: actions;
      justify-content: flex-end;
    }

    .status-badge {
      padding: 0.15rem 0.5rem;
      font-size: 0.75rem;
    }

    .copy-icon,
    .refresh-icon {
      width: 18px;
      height: 18px;
    }

    .copy-button,
    .refresh-button {
      padding: 0.25rem;
    }
  }

  /* Update dark mode styles to use CSS variables consistently */
  :global(.dark) {
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --bg-hover: #3a3a3a;
    --text-primary: #ffffff;
    --text-secondary: #a0aec0;
    --text-hover: #f0f0f0;
    --border-color: #333;
  }
</style>
