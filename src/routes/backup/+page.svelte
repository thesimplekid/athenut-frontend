<script>
  import { goto } from "$app/navigation";
  import Footer from "../../components/Footer.svelte";
  import { copyToClipboard } from "@svelte-put/copy";
  import { showToast } from "$lib/stores/toast";
  import Toast from "../../components/Toast.svelte";
  import seed from "$lib/shared/store/wallet";
  import { onMount } from "svelte";
  import { theme } from "$lib/stores/theme";
  import Navbar from "../../components/Navbar.svelte";

  // Sample words (these should come from your app's logic later)
  const words = $seed.trim().split(/\s+/);

  let isBlurred = true;

  function toggleBlur() {
    isBlurred = !isBlurred;
  }

  function handleCopyPhrase() {
    copyToClipboard($seed);
    showToast("Recovery phrase copied to clipboard.");
  }

  onMount(() => {
    // Get theme from localStorage
    const savedTheme = localStorage.getItem("theme") || "light";
    // Update the theme store
    theme.set(savedTheme);
    // Apply dark mode class if needed
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
  <meta name="description" content="privacy-preserving web search powered by Kagi and Cashu." />
</svelte:head>

<div
  class="min-h-screen flex flex-col text-gray-800 bg-white dark:bg-[var(--bg-primary)] dark:text-white"
>
  <Navbar />
  <main class="flex-grow flex flex-col justify-start items-center px-4 py-8">
    <div class="relative w-full max-w-[800px]">
      <h1 class="text-4xl font-bold mb-2 text-center text-gray-800 dark:text-white">
        Backup
      </h1>

      <div class="absolute right-0 top-0">
        <button class="visibility-toggle" on:click={toggleBlur}>
          <svg 
            xmlns="http://www.w3.org/2000/svg" 
            width="24" 
            height="24" 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            class="eye-icon"
          >
            {#if isBlurred}
              <!-- Closed eye -->
              <path d="m15 18-.722-3.25"/>
              <path d="M2 8a10.645 10.645 0 0 0 20 0"/>
              <path d="m20 15-1.726-2.05"/>
              <path d="m4 15 1.726-2.05"/>
              <path d="m9 18 .722-3.25"/>
            {:else}
              <!-- Open eye -->
              <path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"/>
              <circle cx="12" cy="12" r="3"/>
            {/if}
          </svg>
        </button>
      </div>
    </div>

    <p class="text-xl text-gray-600 dark:text-[#a0aec0] mb-6">
      Save your secret recovery phrase in a secure place that only you control.
    </p>

    <div class="seed-container">
      {#each words as word, i}
        <div class="seed-word">
          <span class="word-number">{i + 1}</span>
          <span class="word-text" class:blurred={isBlurred}>{word}</span>
          <div class="underline"></div>
        </div>
      {/each}
    </div>

    <button class="recovery-button-secondary" on:click={handleCopyPhrase}>
      Copy Recovery Phrase
    </button>

    <div class="divider my-8">OR</div>

    <div class="token-section w-full max-w-800px flex flex-col items-center">
      <h2 class="text-2xl font-bold mb-4 text-center text-gray-800 dark:text-white">
        Export Search Token
      </h2>
      
      <div class="token-input-container seed-container" style="display: block; padding: 1rem;">
        <input
          type="text"
          value="FAKETOKEN"
          class="word-text"
          readonly
          class:blurred={isBlurred}
        />
      </div>
      
      <button 
        class="recovery-button-secondary mb-4" 
        on:click={() => {
          copyToClipboard("FAKETOKEN");
          showToast("Token copied to clipboard");
        }}
      >
        Copy Token
      </button>
    </div>
  </main>

  <Footer />
  <Toast />
</div>

<style>
  .controls-container {
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .visibility-toggle {
    background: none;
    border: none;
    padding: 0.5rem;
    cursor: pointer;
    color: inherit;
  }

  .eye-icon {
    color: var(--text-primary);
  }

  .blurred {
    filter: blur(5px);
    transition: filter 0.2s ease;
  }

  .word-text {
    transition: filter 0.2s ease;
  }

  :global(.dark) .eye-icon {
    color: var(--text-primary);
  }

  .seed-container {
    position: relative;
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
    padding: 0.5rem 0;
    position: relative;
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .word-number {
    color: #666;
    font-size: 1.1rem;
    font-weight: 500;
    min-width: 1.5rem;
  }

  .word-text {
    font-size: 1.2rem;
    color: var(--text-primary, #1f2937);
  }

  .underline {
    position: absolute;
    bottom: 0;
    left: 2.5rem;
    right: 0;
    height: 1px;
    background: #d1d5db;
  }

  .copy-button {
    /* background-color: var(--bg-secondary); */
    color: var(--text-primary);
    padding: 16px 32px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    width: auto;
    max-width: none;
    border-radius: 9999px;
  }


  .copy-button:focus {
    outline: none;
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
  }

  :global(.dark) main {
    background-color: var(--bg-primary);
  }

  :global(.dark) .seed-container {
    background-color: var(--bg-secondary) !important;
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

  :global(.dark) .copy-button {
    color: #a0aec0 !important;
  }

  :global(.dark) .copy-button:hover {
    color: #ffffff !important;
  }

  :global(.dark) .text-gray-600 {
    color: #a0aec0 !important;
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

  .token-section {
    margin-bottom: 2rem;
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

  :global(.dark) .token-input-container {
    background-color: var(--bg-secondary) !important;
  }

  :global(.dark) .divider::before,
  :global(.dark) .divider::after {
    background: #4B5563;
  }

  :global(.dark) .divider {
    color: #6b7280;
  }

  .token-input {
    width: 100%;
    padding: 0.5rem;
    font-size: 1rem;
    border: none;
    background: transparent;
    color: var(--text-primary);
  }

  .token-input:focus {
    outline: none;
  }

  :global(.dark) .token-input {
    color: var(--text-primary);
  }

  .word-text {
    font-size: 1.2rem;
    font-weight: 500;
    background: transparent;
    border: none;
    width: 100%;
  }

  .word-text:focus {
    outline: none;
  }

  :global(.dark) .token-input-container {
    background-color: var(--bg-secondary) !important;
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

  :global(.dark) .recovery-button {
    background-color: #2d2d2d;
    color: white;
  }

  :global(.dark) .recovery-button-secondary {
    color: #a0aec0;
  }

  :global(.dark) .recovery-button-secondary:hover {
    color: #ffffff;
  }

  :global(.dark) h1,
  :global(.dark) h2 {
    color: #ffffff !important;
  }

  :global(.dark) .text-gray-600 {
    color: #6b7280 !important;
  }

  h1, h2 {
    color: #1f2937;
  }

  .text-gray-600 {
    color: #4B5563;
  }

  :global(.dark) .text-gray-600 {
    color: #a0aec0 !important;
  }

  :global(.dark) .underline {
    background: #4B5563;
  }
</style>
