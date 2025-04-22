<script>
  import { onMount } from "svelte";
  import SvgQR from "@svelte-put/qr/svg/QR.svelte";
  import { copyToClipboard } from "@svelte-put/copy";
  import bolt11Decoder from "light-bolt11-decoder";
  import { PUBLIC_API_URL } from "$env/static/public";
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
  } from "$lib/shared/utils";
  import { CashuMint, CashuWallet, MintQuoteState } from "@cashu/cashu-ts";
  import Footer from "../../components/Footer.svelte";
  import { showToast } from "$lib/stores/toast";
  import Toast from "../../components/Toast.svelte";
  import seed from "$lib/shared/store/wallet";
  import { theme } from "$lib/stores/theme";
  import Navbar from "../../components/Navbar.svelte";

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
    /** @type {InfoResult} */
    let info = await fetch(`${PUBLIC_API_URL}/info`, {}).then((r) => r.json());

    $mint_url = info.mint;
  }

  let isLoading = false;

  onMount(async () => {
    if ($mint_url != undefined) {
      balance = getBalance();
      await getInfo();
    }
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
   * @param {string} seed - The wallet seed
   * @returns {Promise<{wallet: CashuWallet, keys: any}>} The initialized wallet and its keys
   */
  async function initializeWallet(mintUrl, seed) {
    const mint = new CashuMint(mintUrl);
    const keysets = await mint.getKeys();
    const matchingKeyset = keysets.keysets.find((key) => key.unit === "xsr");

    const wallet = new CashuWallet(mint, {
      unit: "xsr",
      keys: matchingKeyset,
      // In cashu-ts v2, mnemonicOrSeed has been replaced; using seeds directly
      seed: seed
    });

    return { wallet, keys: wallet.keys };
  }

  /**
   * @param {number} searches
   */
  async function handleTopUp(searches) {
    if (mint_url != null) {
      console.log("Attempting to top up for searches ", searches);
      selectedSearches = searches;

      try {
        // Pass the $seed value from the store to the function
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
          let keyset_count = keyset_counts[keys.id] || 0;

          const options = {
            // In cashu-ts v2, preference is now outputAmounts with sendAmounts and keepAmounts arrays
            outputAmounts: {
              sendAmounts: Array(searches).fill(1) // Creates an array of 1's with length of 'searches'
            },
            keysetId: keys.id,
            counter: keyset_count,
          };

          // In cashu-ts v2, mintTokens is now mintProofs and returns proofs directly, not in an object
          const proofs = await wallet.mintProofs(
            searches,
            mintQuote.quote,
            options,
          );

          let new_count = keyset_count + proofs.length;
          keyset_counts[keys.id] = new_count;
          setKeysetCounts(keyset_counts);

          let current_proofs = getProofs();
          const combinedList = [...current_proofs, ...proofs];
          writeProofs(combinedList);
          updateQuoteState(mintQuote.quote, "paid");
          pendingInvoices = getPendingQuotes();
          goto("/");
        }
      } catch (error) {
        console.error("Error while topping up: ", error);
      }
    }
    isLoading = false;
  }

  /**
   * @param {string} quoteId
   */
  async function handleRefresh(quoteId) {
    console.log(quoteId);
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
      // Pass the $seed value from the store to the function
      const { wallet, keys } = await initializeWallet($mint_url, $seed);
      let keyset_counts = getKeysetCounts();
      let keyset_count = keyset_counts[keys.id] || 0;
      const options = {
        // In cashu-ts v2, preference is now outputAmounts with sendAmounts and keepAmounts arrays
        outputAmounts: {
          sendAmounts: Array(mintQuote.amount).fill(1) // Array of 1's with length of 'mintQuote.amount'
        },
        keysetId: keys.id,
        counter: keyset_count,
      };
      // In cashu-ts v2, mintTokens is now mintProofs and returns proofs directly, not in an object
      let proofs = await wallet.mintProofs(
        mintQuote.amount,
        quoteId,
        options,
      );
      let new_count = keyset_count + proofs.length;
      keyset_counts[keys.id] = new_count;
      setKeysetCounts(keyset_counts);
      let current_proofs = getProofs();
      const combinedList = [...current_proofs, ...proofs];
      writeProofs(combinedList);
      updateQuoteState(mintQuote.id, "paid");
    } catch (error) {
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
        console.log("Alreasy signed");
        // Pass the $seed value from the store to the function
        const { wallet, keys } = await initializeWallet($mint_url, $seed);
        let keyset_counts = getKeysetCounts();
        let keyset_count = keyset_counts[keys.id] || 0;
        let new_count = keyset_count + 10;
        keyset_counts[keys.id] = new_count;
        setKeysetCounts(keyset_counts);
        showToast("Error minting, please try agian");
      }
      console.error("Error while refreshing quote: ", error);
    } finally {
      // Always update pending invoices and balance
      pendingInvoices = getPendingQuotes();
      balance = getBalance();

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
  <meta name="description" content="privacy-preserving web search powered by Kagi and Cashu." />
</svelte:head>

<!-- Update the main container div and add Navbar -->
<div class="min-h-screen flex flex-col text-gray-800 bg-white dark:bg-[var(--bg-primary)] dark:text-white">
  <Navbar />
  <main class="flex-grow flex flex-col justify-start items-center px-4 py-8">
    <div class="header-container">
      <h1 class="text-4xl font-bold mb-2 text-gray-800 dark:text-white">
        Top Up
      </h1>
    </div>

    <div class="text-2xl font-semibold text-gray-900 dark:text-white mt-2 mb-4">
      You have {balance} searches left
    </div>

    <p class="text-xl text-gray-600 mb-6">
      Zap your account with sats to unlock more premium searches.
    </p>

    <div class="qr-container mt-2">
      {#if isLoading}
        <div class="spinner-container">
          <div class="spinner"></div>
        </div>
      {:else if data !== ""}
        <div class="flex flex-col items-center space-y-4">
          <div class="qr-info">
            Purchasing {selectedSearches} searches for {invoice_amount} sats
          </div>
          <div class="qr-wrapper">
            <SvgQR {data} width="300" height="300" />
          </div>
          <button
            type="button"
            class="copy-invoice-button"
            on:click={() => customCopy(data)}>Copy Invoice</button
          >
        </div>
      {:else}
        <div class="top-up-grid">
          {#each [1, 5, 10, 20, 35, 50] as search_count}
            <button
              on:click={() => handleTopUp(search_count)}
              class="top-up-button"
            >
              <div class="text-lg">{search_count} Searches</div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Transaction History Table -->
    {#if true}
      <div class="transaction-history-container">
        <h2 class="history-title">Recent Invoices</h2>
        <div class="transaction-table">
          {#each pendingInvoices.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime()) as quote}
            <!-- Transaction row -->
            <div class="transaction-row">
              <div class="amount-cell">
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
  </main>

  <Footer />
  <Toast />
</div>

<style>
  .top-up-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr); /* Always 3 columns on desktop */
    gap: 1rem;
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
    padding: 0 1rem;
  }

  .top-up-button {
    background-color: #f0f2f5; /* Lighter shade, closer to white */
    color: #374151;
    border: none;
    border-radius: 8px;
    padding: 20px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 120px;
  }

  .top-up-button:hover {
    background-color: #e5e7eb; /* Slightly darker on hover, but still light */
  }

  .top-up-button:focus {
    outline: none;
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.5); /* Light blue focus ring */
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
    background: linear-gradient(135deg, #ffffff 0%, #f8f9fa 100%);
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

  .main-heading {
    display: inline-block;
    position: relative;
  }

  .controls-container {
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .back-button {
    background: none;
    border: none;
    font-size: 2rem;
    cursor: pointer;
    padding: 0 0.5rem;
    color: var(--text-primary);
    transition: color 0.3s ease;
  }

  :global(.dark-mode) .back-button {
    color: #ffffff !important;
  }

  .dark-mode .back-button {
    color: #ffffff !important;
  }

  .copy-invoice-button {
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

  .copy-invoice-button:hover {
    background-color: #2a2a2a;
    box-shadow: 0 4px 8px rgba(26, 26, 26, 0.3);
  }

  .copy-invoice-button:focus {
    outline: none;
    box-shadow:
      0 0 0 2px rgba(255, 255, 255, 0.5),
      0 4px 8px rgba(26, 26, 26, 0.3);
  }

  .copy-invoice-button::before {
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

  .copy-invoice-button:hover::before {
    opacity: 0.5;
  }

  .qr-container {
    min-height: 300px; /* Reduced height */
    width: 100%;
    max-width: 800px;
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
    color: var(--text-primary);
  }

  /* Dark mode styles */
  :global(.dark-mode) {
    background-color: #1a1a1a;
  }

  :global(.dark-mode) .main-heading {
    color: #ffffff !important;
  }

  :global(.dark-mode) .text-gray-900 {
    color: #ffffff;
  }

  :global(.dark-mode) .text-gray-600,
  :global(.dark-mode) .text-gray-800 {
    color: #a0aec0;
  }

  :global(.dark-mode) .top-up-button {
    background-color: #2d2d2d;
    color: #ffffff;
  }

  :global(.dark-mode) .top-up-button:hover {
    background-color: #3a3a3a;
  }

  :global(.dark-mode) .transaction-table {
    background-color: #2d2d2d;
  }

  /* QR code specific styles */
  .qr-wrapper {
    background-color: white;
    padding: 1rem;
    border-radius: 8px;
  }

  /* Ensure QR code stays visible in dark mode */
  :global(.dark-mode) .qr-wrapper {
    background-color: white !important;
  }

  :global(.dark-mode) .qr-wrapper :global(svg) {
    color: black !important;
    fill: black !important;
  }

  /* Add these transaction table styles */
  .transaction-history-container {
    width: 100%;
    max-width: 800px;
    margin-top: 2rem;
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
  :global(.dark-mode) .transaction-table {
    background-color: #2d2d2d;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  :global(.dark-mode) .transaction-row {
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  :global(.dark-mode) .amount-cell {
    color: #ffffff;
  }

  :global(.dark-mode) .time-cell {
    color: #9ca3af;
  }

  :global(.dark-mode) .copy-button,
  :global(.dark-mode) .refresh-button {
    color: #9ca3af;
  }

  :global(.dark-mode) .copy-button:hover,
  :global(.dark-mode) .refresh-button:hover {
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
  :global(.dark-mode) .status-badge.Pending {
    background-color: #451a03;
    color: #fdba74;
    border: 1px solid #f97316;
  }

  :global(.dark-mode) .status-badge.Paid {
    background-color: #052e16;
    color: #86efac;
    border: 1px solid #22c55e;
  }

  /* Add this to your dark mode styles */
  :global(.dark-mode) .history-title {
    color: #ffffff;
  }

  /* Add these dark mode styles */
  :global(.dark-mode) .qr-info {
    color: #ffffff !important;
  }

  .dark-mode .qr-info {
    color: #ffffff !important;
  }

  /* Update the copy-invoice-button styles */
  .copy-invoice-button {
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

  /* Add dark mode styles for copy-invoice-button */
  :global(.dark-mode) .copy-invoice-button {
    background-color: #2d2d2d;
    color: white;
    border: 2px solid #ffffff;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  :global(.dark-mode) .copy-invoice-button:hover {
    background-color: #3a3a3a;
    border-color: #f0f0f0;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.4);
  }

  :global(.dark-mode) .copy-invoice-button:focus {
    outline: none;
    box-shadow: 0 0 0 2px #1a1a1a,
      0 0 0 4px rgba(255, 255, 255, 0.5);
  }

  :global(.dark-mode) .copy-invoice-button::before {
    background: linear-gradient(45deg, #2d2d2d, #3a3a3a, #4a4a4a, #5a5a5a);
  }

  :global(body) {
    overflow-x: hidden;
    background-color: #ffffff;
  }

  :global(body.dark-mode) {
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
