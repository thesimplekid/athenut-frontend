<script>
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { getBalance } from "$lib/shared/utils";
  import wordmark from "/src/wordmark.png";
  import Footer from "../components/Footer.svelte";
  import Navbar from "../components/Navbar.svelte";
  import bgl from "/src/bgl.png";

  /** @type {number} */
  let balance = 0;

  let search_query = "";

  let isLoading = true;

  onMount(async () => {
    balance = await getBalance();
    isLoading = false;
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

<div class="min-h-screen flex flex-col text-gray-800 relative">
  <!-- Move Navbar outside the main content area and ensure it has highest z-index -->
  <div class="nav-wrapper">
    <Navbar {balance} />
  </div>

  <!-- Main content -->
  <div class="flex-grow flex flex-col justify-center items-center p-4 min-h-0 relative">
    <div class="container mx-auto text-center relative w-full max-w-4xl px-4">
      <div class="background-image"></div>

      <img src="{wordmark}" alt="X-Cashu Search" class="wordmark" />

      <h2 class="text-xl md:text-2xl font-normal text-gray-500 mb-8 px-4">
        Search smarter. Pay in sats for results that matter.
      </h2>

      <div class="search-container mx-auto">
        {#if isLoading}
          <div class="spinner-container">
            <div class="spinner"></div>
          </div>
        {:else if balance === undefined || balance <= 0}
          <div class="empty-state">
            <h3 class="empty-state-title">Top Up Required</h3>
            <p class="empty-state-description">
              You need to add funds to start searching.
            </p>
            <a href="/topup" class="empty-state-button">Top Up Now</a>
          </div>
        {:else}
          <div class="flex flex-col items-center space-y-8">
            <div class="search-input-wrapper">
              <div class="bg-white p-2 rounded-input-container shadow-md w-full">
                <input
                  type="text"
                  autocomplete="off"
                  placeholder="Ask whatever you want..."
                  class="w-full rounded-input border-none focus:outline-none"
                  bind:value="{search_query}"
                  on:keyup="{handleKeyup}"
                />
              </div>
            </div>
            <button class="search-button" on:click="{handleSearch}">
              <span class="search-button-text">Search</span>
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <Footer />
</div>

<style>
  /* Replace the top global styles with these */
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

  /* Keep your existing dark mode styles */
  :global(.dark) {
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --text-primary: #ffffff;
    --text-secondary: #a0aec0;
  }

  /* Base styles */
  .container {
    position: relative;
    width: 100%;
    max-width: 4xl;
    margin: 0 auto;
    overflow: visible;
    z-index: 1;
  }

  .wordmark {
    position: relative;
    width: min(400px, 80vw);
    height: auto;
    margin: 0 auto 1rem;
    z-index: 1;
  }

  .search-container {
    width: 100%;
    max-width: min(32rem, 90vw);
    margin: 0 auto;
  }

  /* Ensure search input is mobile-friendly */
  .search-input-wrapper {
    width: 100%;
    max-width: min(32rem, 90vw);
    margin: 0 auto;
  }

  .rounded-input-container {
    border-radius: 9999px;
    overflow: hidden;
  }

  .rounded-input {
    width: 100%;
    padding: 8px 16px;
    font-size: 16px;
    border-radius: 9999px;
  }

  /* Update button styles for better mobile display */
  .search-button {
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

  .search-button:hover {
    background-color: #2a2a2a;
    box-shadow: 0 4px 8px rgba(26, 26, 26, 0.3);
  }

  .search-button:focus {
    outline: none;
    box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.5),
      0 4px 8px rgba(26, 26, 26, 0.3);
  }

  .search-button::before {
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

  .search-button:hover::before {
    opacity: 0.5;
  }

  .search-button-text {
    position: relative;
    z-index: 1;
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
    backdrop-filter: blur(8px);
    border: 1px solid rgba(226, 232, 240, 0.6);
    border-radius: 12px;
    padding: 32px;
    text-align: center;
    width: 100%;
    max-width: 600px;
    margin: 0 auto;
  }

  .empty-state-title {
    font-size: 24px;
    font-weight: 600;
    color: #1a1a1a;
    margin-bottom: 16px;
  }

  .empty-state-description {
    font-size: 18px;
    color: #4a5568;
    margin-bottom: 24px;
  }

  .empty-state-button {
    display: inline-block;
    background-color: #1a1a1a;
    color: white;
    border: none;
    border-radius: 9999px;
    padding: 14px 28px;
    font-size: 18px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    text-decoration: none;
  }

  .empty-state-button:hover {
    background-color: #2a2a2a;
  }

  .empty-state-button:focus {
    outline: none;
    box-shadow: 0 0 0 2px rgba(26, 26, 26, 0.3);
  }

  /* Background image styles */
  .background-image {
    position: absolute;
    top: 10%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 800px;
    height: 800px;
    background-image: url("/src/bgl.png");
    background-size: contain;
    background-repeat: no-repeat;
    background-position: center;
    opacity: 0.5;
    z-index: 0;
    pointer-events: none; /* Prevent background from intercepting clicks */
    -webkit-mask-image: linear-gradient(
      to right,
      transparent,
      black 20%,
      black 80%,
      transparent
    );
    mask-image: linear-gradient(
      to right,
      transparent,
      black 20%,
      black 80%,
      transparent
    );
  }

  /* Dark mode background image */
  :global(.dark) .background-image {
    background-image: url("/src/bgld.png");
  }

  /* Ensure content stays above the blur but below navbar */
  .wordmark,
  .search-container {
    position: relative;
    z-index: 1;
  }

  /* Dark mode styles */
  :global(.dark) {
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --text-primary: #ffffff;
    --text-secondary: #a0aec0;
  }

  :global(.dark) .min-h-screen {
    background-color: var(--bg-primary);
    color: var(--text-primary);
  }

  :global(.dark) .search-button {
    background-color: #2d2d2d;
    color: white;
    border: 2px solid #ffffff;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  :global(.dark) .search-button:hover {
    background-color: #3a3a3a;
    border-color: #f0f0f0;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.4);
  }

  :global(.dark) .search-button:focus {
    outline: none;
    box-shadow: 0 0 0 2px #1a1a1a,
      0 0 0 4px rgba(255, 255, 255, 0.5);
  }

  :global(.dark) .search-button::before {
    background: linear-gradient(45deg, #2d2d2d, #3a3a3a, #4a4a4a, #5a5a5a);
  }

  :global(.dark) .rounded-input-container {
    background-color: var(--bg-secondary);
  }

  :global(.dark) .rounded-input {
    background-color: var(--bg-secondary);
    color: var(--text-primary);
  }

  /* Dark mode styles for empty state */
  :global(.dark) .empty-state {
    border-color: rgba(255, 255, 255, 0.1);
  }

  :global(.dark) .empty-state-title {
    color: #ffffff;
  }

  :global(.dark) .empty-state-description {
    color: #a0aec0;
  }

  :global(.dark) .empty-state-button {
    background-color: #ffffff;
    color: #1a1a1a;
  }

  :global(.dark) .empty-state-button:hover {
    background-color: #f0f0f0;
  }

  /* Adjust main content spacing */
  .container {
    margin-top: 1rem;
  }

  /* Update main wrapper */
  .min-h-screen {
    position: relative;
    overflow-x: hidden;
  }

  /* Update the main content container */
  .flex-grow {
    position: relative;
    z-index: 1;
  }

  /* Add new nav-wrapper styles */
  .nav-wrapper {
    position: relative;
    z-index: 100; /* Ensure navbar is always on top */
  }
</style>