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
  <!-- Animated orbs moving along grid lines -->
  <div class="grid-orbs">
    <div class="orb orb-blue"></div>
    <div class="orb orb-red"></div>
    <div class="orb orb-yellow"></div>
    <div class="orb orb-green"></div>
  </div>
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
          class="tagline text-balance"
          in:fly={{ y: 20, duration: 500, delay: 400, easing: quintOut }}
        >
          Search smarter. Pay in sats for results that matter.
        </h2>

        <div class="search-container" in:fly={{ y: 30, duration: 500, delay: 600, easing: quintOut }}>
          {#if isLoading}
            <div class="search-skeleton">
              <div class="skeleton-input"></div>
              <div class="skeleton-button"></div>
            </div>
          {:else if balance === undefined || balance <= 0}
            <div class="empty-state" transition:scale={{ duration: 300, easing: quintOut }}>
              <h3 class="empty-state-title">Top Up Required</h3>
              <p class="empty-state-description text-pretty">
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
              <button class="search-button" on:click="{handleSearch}" aria-label="Search">
                <span class="search-button-text">Search</span>
                <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
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
    min-height: 100dvh;
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
    border: 1px solid rgba(0, 0, 0, 0.05);
    /* Only animate transform and border, not backdrop-filter */
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1), border-color 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .search-input-container:focus-within {
    transform: translateY(-2px);
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
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .search-button:hover {
    background: #2a2a2a;
    transform: translateY(-2px);
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

  /* Skeleton loading states */
  .search-skeleton {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    align-items: center;
    width: 100%;
  }

  .skeleton-input {
    width: 100%;
    height: 60px;
    background: linear-gradient(
      90deg,
      rgba(0, 0, 0, 0.06) 0%,
      rgba(0, 0, 0, 0.1) 50%,
      rgba(0, 0, 0, 0.06) 100%
    );
    background-size: 200% 100%;
    border-radius: 16px;
    animation: skeleton-loading 1.5s ease-in-out infinite;
  }

  .skeleton-button {
    width: 200px;
    height: 48px;
    background: linear-gradient(
      90deg,
      rgba(0, 0, 0, 0.06) 0%,
      rgba(0, 0, 0, 0.1) 50%,
      rgba(0, 0, 0, 0.06) 100%
    );
    background-size: 200% 100%;
    border-radius: 12px;
    animation: skeleton-loading 1.5s ease-in-out infinite;
  }

  @keyframes skeleton-loading {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .skeleton-input,
    .skeleton-button {
      animation: none;
      background: rgba(0, 0, 0, 0.06);
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
    /* No transitions on backdrop-filter elements */
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
  }

  .empty-state-button:hover {
    background: #2a2a2a;
    transform: translateY(-2px);
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

  /* Animated orbs moving along grid lines */
  .grid-orbs {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    pointer-events: none;
    z-index: 1;
    overflow: hidden;
  }

  .orb {
    position: absolute;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    opacity: 0.3;
    filter: blur(0.5px);
    box-shadow: 0 0 6px currentColor, 0 0 12px currentColor;
  }

  /* Comet trail effect using pseudo-element */
  .orb::before {
    content: '';
    position: absolute;
    width: 60px;
    height: 3px;
    background: linear-gradient(
      to right,
      currentColor 0%,
      rgba(255, 255, 255, 0.6) 10%,
      currentColor 20%,
      transparent 100%
    );
    opacity: 0.35;
    filter: blur(3px);
    transform-origin: left center;
    pointer-events: none;
  }

  /* Horizontal orbs - trail behind (left side) */
  .orb-blue::before {
    left: -60px;
    top: 50%;
    transform: translateY(-50%);
  }

  /* Vertical orbs - trail above */
  .orb-red::before {
    top: -60px;
    left: 50%;
    transform: translateX(-50%) rotate(90deg);
  }

  /* Reverse direction trails */
  .orb-yellow::before {
    left: auto;
    right: -60px;
    background: linear-gradient(
      to left,
      currentColor 0%,
      rgba(255, 255, 255, 0.6) 10%,
      currentColor 20%,
      transparent 100%
    );
  }

  .orb-green::before {
    top: auto;
    bottom: -60px;
    left: 50%;
    transform: translateX(-50%) rotate(90deg);
    background: linear-gradient(
      to top,
      currentColor 0%,
      rgba(255, 255, 255, 0.6) 10%,
      currentColor 20%,
      transparent 100%
    );
  }

  /* Blue orb - moves horizontally along grid line at 150px (3 grid units) */
  .orb-blue {
    background: #3b82f6;
    color: #3b82f6;
    top: 150px;
    left: -20px;
    animation: moveHorizontal 25s linear infinite;
    animation-delay: 0s;
  }

  /* Red orb - moves vertically along grid line at 200px (4 grid units) */
  .orb-red {
    background: #ef4444;
    color: #ef4444;
    top: -20px;
    left: 200px;
    animation: moveVertical 22s linear infinite;
    animation-delay: 3s;
  }

  /* Yellow orb - moves horizontally along grid line at 300px (6 grid units) */
  .orb-yellow {
    background: #fbbf24;
    color: #fbbf24;
    top: 300px;
    right: -20px;
    animation: moveHorizontalReverse 28s linear infinite;
    animation-delay: 6s;
  }

  /* Green orb - moves vertically along grid line at 450px (9 grid units) */
  .orb-green {
    background: #10b981;
    color: #10b981;
    bottom: -20px;
    left: 450px;
    animation: moveVerticalReverse 24s linear infinite;
    animation-delay: 9s;
  }

  @keyframes moveHorizontal {
    0% {
      transform: translateX(0);
    }
    100% {
      transform: translateX(calc(100vw + 40px));
    }
  }

  @keyframes moveHorizontalReverse {
    0% {
      transform: translateX(0);
    }
    100% {
      transform: translateX(calc(-100vw - 40px));
    }
  }

  @keyframes moveVertical {
    0% {
      transform: translateY(0);
    }
    100% {
      transform: translateY(calc(100vh + 40px));
    }
  }

  @keyframes moveVerticalReverse {
    0% {
      transform: translateY(0);
    }
    100% {
      transform: translateY(calc(-100vh - 40px));
    }
  }

  /* Pause animations when off-screen or reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .orb {
      animation: none !important;
      opacity: 0;
    }
  }

  /* Dark mode orb adjustments - slightly more visible */
  :global(.dark) .orb {
    opacity: 0.25;
  }

  :global(.dark) .orb::before {
    opacity: 0.3;
  }

</style>