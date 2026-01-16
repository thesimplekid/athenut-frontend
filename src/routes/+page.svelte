<script>
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { getBalance } from "$lib/shared/utils";
  import wordmark from "/src/wordmark.png";
  import Footer from "../components/Footer.svelte";
  import Navbar from "../components/Navbar.svelte";
  import bgl from "/src/bgl.png";
  import { fade, fly, scale } from 'svelte/transition';
  import { quintOut, elasticOut } from 'svelte/easing';

  /** @type {number} */
  let balance = 0;

  let search_query = "";

  let isLoading = true;
  let contentReady = false;

  onMount(async () => {
    balance = await getBalance();
    isLoading = false;
    setTimeout(() => contentReady = true, 100);
  });

  /**
   * Handles the keyup event for the search input
   * @param {KeyboardEvent} e - The keyboard event object
   */
  function handleKeyup(e) {
    if (e.key === "Enter") {
      handleSearch();
    }
  }

  function handleSearch() {
    if (search_query.trim()) {
      const urlEncoded = encodeURIComponent(search_query);
      goto(`/search?q=${urlEncoded}`);
    }
  }
</script>

<svelte:head>
  <title>Athenut</title>
  <meta name="description" content="Privacy-preserving web search powered by Kagi and Cashu." />
  <meta name="keywords" content="search, kagi, ecash, cashu" />
  <meta name="robots" content="index, follow" />
  <link rel="canonical" href="https://athenut.com" />
</svelte:head>

<div class="page-wrapper">
  <Navbar {balance} />

  <!-- Main content with top padding for fixed navbar -->
  <div class="main-content">
    {#if contentReady}
      <div class="container" in:fade={{ duration: 600, easing: quintOut }}>
        <img
          src="{wordmark}"
          alt="X-Cashu Search"
          class="wordmark"
          in:scale={{ duration: 600, delay: 200, start: 0.8, easing: elasticOut }}
        />

        <h2
          class="tagline"
          in:fly={{ y: 20, duration: 500, delay: 400, easing: quintOut }}
        >
          Search smarter. Pay in sats for results that matter.
        </h2>

        <div class="search-container" in:fly={{ y: 30, duration: 500, delay: 600, easing: quintOut }}>
          {#if isLoading}
            <div class="spinner-container">
              <div class="spinner"></div>
            </div>
          {:else if balance === undefined || balance <= 0}
            <div class="empty-state" transition:scale={{ duration: 300, easing: quintOut }}>
              <h3 class="empty-state-title">Top Up Required</h3>
              <p class="empty-state-description">
                You need to add funds to start searching.
              </p>
              <a href="/topup" class="empty-state-button">Top Up Now</a>
            </div>
          {:else}
            <div class="search-form">
              <div class="search-input-wrapper">
                <div class="search-input-container">
                  <input
                    type="text"
                    autocomplete="off"
                    placeholder="Ask whatever you want..."
                    class="search-input"
                    bind:value="{search_query}"
                    on:keyup="{handleKeyup}"
                  />
                </div>
              </div>
              <button class="search-button" on:click="{handleSearch}">
                <span class="search-button-text">Search</span>
                <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="11" cy="11" r="8"></circle>
                  <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                </svg>
              </button>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  <Footer />
</div>

<style>
  /* Modern page layout */
  :global(:root) {
    --bg-primary: #ffffff;
    --bg-secondary: #f7f7f7;
    --text-primary: #1a1a1a;
    --text-secondary: #4a5568;
  }

  :global(body) {
    background-color: var(--bg-primary);
    color: var(--text-primary);
  }

  :global(.dark) {
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --text-primary: #ffffff;
    --text-secondary: #a0aec0;
  }

  .page-wrapper {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    position: relative;
    /* Grid background */
    background-image:
      linear-gradient(rgba(0, 0, 0, 0.03) 1px, transparent 1px),
      linear-gradient(90deg, rgba(0, 0, 0, 0.03) 1px, transparent 1px);
    background-size: 50px 50px;
    background-position: center center;
  }

  /* Radial gradient overlay to fade grid */
  .page-wrapper::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: radial-gradient(
      circle at center,
      transparent 0%,
      transparent 40%,
      var(--bg-primary) 70%,
      var(--bg-primary) 100%
    );
    pointer-events: none;
    z-index: 0;
  }

  :global(.dark) .page-wrapper {
    background-image:
      linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255, 255, 255, 0.03) 1px, transparent 1px);
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    padding: 120px 1rem 2rem;
    position: relative;
    z-index: 1;
  }

  .container {
    position: relative;
    width: 100%;
    max-width: 900px;
    margin: 0 auto;
    text-align: center;
    z-index: 1;
  }

  .tagline {
    font-size: clamp(1.25rem, 3vw, 1.5rem);
    font-weight: 400;
    color: var(--text-secondary);
    margin-bottom: 3rem;
    padding: 0 1rem;
    line-height: 1.6;
  }

  .wordmark {
    width: min(450px, 85vw);
    height: auto;
    margin: 0 auto 2rem;
    filter: drop-shadow(0 4px 12px rgba(0, 0, 0, 0.08));
  }

  .search-container {
    width: 100%;
    max-width: 600px;
    margin: 0 auto;
  }

  .search-form {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    align-items: center;
    width: 100%;
  }

  .search-input-wrapper {
    width: 100%;
  }

  .search-input-container {
    background: rgba(255, 255, 255, 0.8);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-radius: 16px;
    padding: 0.75rem 1.5rem;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
    border: 1px solid rgba(0, 0, 0, 0.05);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .search-input-container:focus-within {
    transform: translateY(-2px);
    box-shadow: 0 12px 24px -4px rgba(102, 126, 234, 0.2), 0 4px 8px -2px rgba(102, 126, 234, 0.1);
    border-color: rgba(102, 126, 234, 0.4);
  }

  :global(.dark) .search-input-container {
    background: rgba(45, 45, 45, 0.8);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .search-input {
    width: 100%;
    padding: 0.5rem 0;
    font-size: 16px;
    border: none;
    background: transparent;
    color: var(--text-primary);
    outline: none;
  }

  .search-input::placeholder {
    color: var(--text-secondary);
  }

  .search-button {
    background: #1a1a1a;
    color: white;
    border: none;
    border-radius: 12px;
    padding: 16px 48px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 2px 8px 0 rgba(0, 0, 0, 0.1);
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .search-button:hover {
    background: #2a2a2a;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px 0 rgba(0, 0, 0, 0.15);
  }

  .search-button:active {
    transform: translateY(0);
  }

  :global(.dark) .search-button {
    background: #ffffff;
    color: #1a1a1a;
  }

  :global(.dark) .search-button:hover {
    background: #f0f0f0;
  }

  /* Mobile adjustments */
  @media (max-width: 640px) {
    .search-button {
      width: 80%;
      max-width: 250px;
      padding: 14px 28px;
      font-size: 16px;
    }

    .wordmark {
      max-width: 280px;
    }

    /* Removing unused selector */

    .background-image {
      width: 100vw;
      height: 100vw;
      -webkit-mask-image: linear-gradient(
        to right,
        transparent,
        black 10%,
        black 90%,
        transparent
      );
      mask-image: linear-gradient(
        to right,
        transparent,
        black 10%,
        black 90%,
        transparent
      );
    }

    .container {
      margin-top: 0.5rem;
    }
  }

  /* Spinner styles */
  .spinner-container {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100px;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(26, 26, 26, 0.1);
    border-top: 3px solid #1a1a1a;
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

  /* Empty state styles */
  .empty-state {
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    background: rgba(255, 255, 255, 0.6);
    border: 1px solid rgba(226, 232, 240, 0.6);
    border-radius: 24px;
    padding: 48px 32px;
    text-align: center;
    width: 100%;
    max-width: 500px;
    margin: 0 auto;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
  }

  :global(.dark) .empty-state {
    background: rgba(45, 45, 45, 0.6);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .empty-state-title {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 16px;
  }

  .empty-state-description {
    font-size: 16px;
    color: var(--text-secondary);
    margin-bottom: 32px;
    line-height: 1.6;
  }

  .empty-state-button {
    display: inline-block;
    background: #1a1a1a;
    color: white;
    border: none;
    border-radius: 12px;
    padding: 16px 40px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    text-decoration: none;
    box-shadow: 0 2px 8px 0 rgba(0, 0, 0, 0.1);
  }

  .empty-state-button:hover {
    background: #2a2a2a;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px 0 rgba(0, 0, 0, 0.15);
  }

  .empty-state-button:active {
    transform: translateY(0);
  }

  :global(.dark) .empty-state-button {
    background: #ffffff;
    color: #1a1a1a;
  }

  :global(.dark) .empty-state-button:hover {
    background: #f0f0f0;
  }

  /* Ensure content stays above the background grid */
  .wordmark,
  .search-container {
    position: relative;
    z-index: 2;
  }

</style>