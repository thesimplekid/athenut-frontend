<script>
  import { onMount } from "svelte";
  // API dependency removed to focus on UI fixes only
  import { goto } from "$app/navigation";
  import mint_url from "$lib/shared/store/mint_url";
  import {
    addSpentProof,
    getBalance,
    getKeysetCounts,
    getProofs,
    setKeysetCounts,
    writeProofs,
  } from "$lib/shared/utils";
  import { getEncodedTokenV4 } from "@cashu/cashu-ts";
  import { page } from "$app/stores";
  import Footer from "../../components/Footer.svelte";
  import { theme } from "$lib/stores/theme";
  import Navbar from "../../components/Navbar.svelte";

  /** @type {import("@cashu/cashu-ts").Token} */

  /** @type {number} */
  let balance = getBalance();

  let search_query = "";

  /**
   * @typedef {Object} SearchResult
   * @property {string} title
   * @property {string} url
   * @property {string} description
   * @property {string} age
   */

  /**
   * Save search results to session storage
   * @param {Array.<SearchResult>} results
   */
  function saveSearchResults(results) {
    sessionStorage.setItem(
      "searchResults",
      JSON.stringify({
        results,
        timestamp: Date.now(),
        query: search_query,
      }),
    );
  }

  /**
   * Get search results from session storage
   * @returns {{results: Array.<SearchResult>, timestamp: number, query: string} | null}
   */
  function getStoredSearchResults() {
    const stored = sessionStorage.getItem("searchResults");
    return stored ? JSON.parse(stored) : null;
  }

  /** @type {Array.<SearchResult>} */
  let search_results = [];

  let isLoading = false;
  let summary = "";

  let searchTime = "0";

  let searchPerformed = false;

  onMount(async () => {
    let q = $page.url.searchParams.get("q");
    if (q != null) {
      search_query = q;

      let stored_search_results = getStoredSearchResults();

      if (
        stored_search_results != null &&
        search_query === stored_search_results.query
      ) {
        search_results = stored_search_results.results;
      } else {
        await handleSearch();
      }
    } else {
      alert("Unknown search param");
    }

    balance = getBalance();
  });

  async function handleSearch() {
    searchPerformed = true;
    isLoading = true;
    search_results = [];
    const startTime = Date.now();

    let proofs = getProofs();

    let proof = proofs.pop();
    
    // Preserve the keyset ID from the proof
    const keysetId = proof.id ? proof.id : "default";
    console.log(`Using keyset ID: ${keysetId} for search token redemption`);

    try {
      /** @type {import("@cashu/cashu-ts").Token} */
      let token = {
        mint: $mint_url,
        proofs: [proof],
        unit: "XSR"
      };

      let encoded_token = getEncodedTokenV4(token);

      // API call removed to focus on UI fixes only
      // let response = await fetch(`${PUBLIC_API_URL}/search?q=${search_query}`, {
      //   headers: { "X-Cashu": `${encoded_token}` },
      // });
      // 
      // if (!response.ok) {
      //   console.error(`Error: ${response.status} ${response.statusText}`);
      //   throw new Error(`Search failed with status ${response.status}`);
      // }
      // 
      // search_results = await response.json();
      
      // Placeholder for UI testing
      search_results = [];

      // Update keyset counts to preserve the keyset ID
      let keyset_counts = getKeysetCounts();
      let keyset_count = keyset_counts[keysetId] || 0;
      keyset_counts[keysetId] = keyset_count + 1; // Increment counter for this keyset
      setKeysetCounts(keyset_counts);

      /// Add spent proof to store
      addSpentProof(proof);
      // Write the updated proofs back to storage
      writeProofs(proofs);

      console.log("Updated proofs after removing the used one: ", proofs);
      searchTime = ((Date.now() - startTime) / 1000).toFixed(2);
      balance = getBalance();
    } catch (error) {
      console.error("Search failed with error: ", error);

      alert("Search failed");

      balance = getBalance();
      if (balance == 0) {
        goto("/topup");
      } else {
        goto("/");
      }
    } finally {
      isLoading = false;
      balance = getBalance();
      saveSearchResults(search_results);
    }
  }

  /**
   * Handles keyboard keyup events and triggers search if Enter key is pressed and balance is positive
   * @param {KeyboardEvent} e - The keyboard event object
   * @returns {Promise<void>}
   */
  async function handleKeyup(e) {
    if (e.key === "Enter" && balance === 0) {
      goto("/topup");
    } else if (e.key === "Enter" && balance > 0) {
      await handleSearch();
    }
  }
</script>

<svelte:head>
  <title>Athenut</title>
  <meta name="description" content="privacy-preserving web search powered by Kagi and Cashu." />
</svelte:head>

<div
  class="min-h-screen flex flex-col relative gradient-background"
  style="background-color: var(--bg-primary); color: var(--text-primary)"
>
  <Navbar {balance} />

  <header class="p-4 flex items-center" class:search-active={searchPerformed}>
    <div
      class="search-container flex-grow"
      class:search-active={searchPerformed}
    >
      <div class="flex items-center">
        <div class="search-input-wrapper flex-grow mr-2 relative">
          <div
            class="bg-white dark:bg-[var(--bg-secondary)] p-2 rounded-input-container shadow-md w-full"
          >
            <input
              type="text"
              autocomplete="off"
              placeholder="Ask whatever you want..."
              class="w-full rounded-input border-none focus:outline-none pr-10"
              style="background-color: var(--bg-secondary); color: var(--text-primary);"
              bind:value={search_query}
              on:keyup={handleKeyup}
            />
            <button
              class="search-button absolute right-2 top-1/2 transform -translate-y-1/2"
              on:click={handleSearch}
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <circle cx="11" cy="11" r="8"></circle>
                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </header>

  <div class="flex-grow flex flex-col relative">
    {#if !isLoading && search_results.length > 0}
      <p
        class="text-sm mb-4 search-aligned"
        style="color: var(--text-secondary)"
      >
        Found {search_results.length} results in {searchTime} seconds
      </p>

      {#if summary}
        <div class="w-full mb-6 search-aligned">
          <div class="max-w-3xl w-full">
            <div class="bg-gray-50 p-6 rounded-lg">
              <h3 class="text-lg font-medium mb-3">Nutbrief</h3>
              <p class="text-base text-gray-700 leading-relaxed">{summary}</p>
            </div>
          </div>
        </div>
      {/if}
    {/if}

    <div class="flex-grow">
      <main class="search-aligned">
        {#if isLoading}
          <div class="spinner-container">
            <div class="spinner"></div>
          </div>
        {:else if search_results.length === 0}
          <p class="text-center text-gray-400">
            No results found. Try a different search query.
          </p>
        {:else}
          <div class="space-y-6">
            {#each search_results as search_result}
              <!-- Search result item -->
              <div
                class="py-4 border-b"
                style="border-color: var(--border-color)"
              >
                <h3 class="text-xl mb-2">
                  <a
                    href={search_result.url}
                    class="font-medium underline"
                    style="color: var(--text-primary)"
                  >
                    {search_result.title}
                  </a>
                </h3>
                <p class="text-sm mb-2" style="color: var(--text-secondary)">
                  {search_result.url}
                </p>
                <p style="color: var(--text-primary)">
                  {search_result.description}
                </p>
                {#if search_result.age && search_result.age !== "null"}
                  <span
                    class="inline-block mt-2 px-3 py-1 text-sm rounded-full"
                    style="background-color: var(--bg-secondary); color: var(--text-secondary)"
                  >
                    {search_result.age}
                  </span>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </main>
    </div>
  </div>

  <Footer />
</div>

<style lang="postcss">
  .gradient-background {
    background: white;
    position: relative;
  }



  .rounded-input-container {
    border-radius: 9999px;
    overflow: hidden;
  }

  .rounded-input {
    border-radius: 9999px;
    padding: 10px 16px;
    height: 44px;
    padding-right: 40px; /* Make room for the search button */
  }

  .search-button {
    background-color: transparent;
    color: #1a1a1a;
    border: none;
    padding: 8px;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .search-button:hover {
    color: #4a5568;
  }

  .search-button:focus {
    outline: none;
  }

  .spinner-container {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: var(--bg-primary);
    opacity: 0.8;
    z-index: 1000;
  }

  .spinner {
    width: 50px;
    height: 50px;
    border: 4px solid var(--text-secondary);
    border-top: 4px solid var(--text-primary);
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

  /* Update header and search container layout */
  header {
    padding: 1rem;
    display: flex;
    justify-content: center;
    width: 100%;
    max-width: 1200px;
    margin: 0 auto;
  }

  .search-container {
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
    transition: all 0.3s ease;
  }

  .search-container.search-active {
    width: 800px;
    max-width: 800px;
  }

  /* Keep search results alignment consistent */
  .search-aligned {
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
    padding: 0 1rem;
  }

  /* Mobile responsiveness */
  @media (max-width: 1024px) {
    header {
      padding: 1rem;
    }

    .search-container {
      max-width: 90%;
    }

    .search-aligned {
      max-width: 90%;
    }
  }

  @media (max-width: 768px) {
    header {
      padding: 1rem;
    }

    .search-container,
    .search-aligned {
      max-width: 95%;
    }
  }

  /* Removed unused theme-toggle styles */

  /* Dark mode styles */
  :global(.dark) {
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --text-primary: #ffffff;
    --text-secondary: #a0aec0;
  }

  /* Add dark mode styles */
  :global(.dark) .search-button {
    color: #a0aec0; /* Update color for dark mode */
  }
</style>
