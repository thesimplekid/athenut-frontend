<script>
    import { theme } from '$lib/stores/theme';
    import logomark from '/src/logomark.png';
    import { getBalance, forceBalanceRefresh } from '$lib/shared/utils';
    import { onMount } from 'svelte';
  
    // If balance prop is provided, use it. Otherwise, will be set in onMount
    export let balance = 0;
  
    let isDropdownOpen = false;
  
    function toggleDropdown() {
      isDropdownOpen = !isDropdownOpen;
    }
  
    function toggleTheme() {
      theme.update((current) => (current === 'light' ? 'dark' : 'light'));
    }
    
    // Always refresh the balance when the component is mounted
    onMount(async () => {
      // Use forceBalanceRefresh to get directly from localStorage
      balance = forceBalanceRefresh();
      console.log('Navbar - balance after forceBalanceRefresh:', balance);
      
      // Set up a listener for storage changes
      const handleStorageChange = (e) => {
        if (e.key === 'proofs') {
          console.log('Navbar - Storage event - proofs changed');
          balance = forceBalanceRefresh();
          console.log('Navbar - Updated balance:', balance);
        }
      };
      
      // Add storage listener
      window.addEventListener('storage', handleStorageChange);
      
      // Also periodically refresh the balance every few seconds
      const intervalId = setInterval(() => {
        balance = forceBalanceRefresh();
      }, 3000); // Refresh every 3 seconds
      
      return () => {
        // Clean up listener and interval when component is destroyed
        window.removeEventListener('storage', handleStorageChange);
        clearInterval(intervalId);
      };
    });
    
    // Function to manually refresh the balance
    function refreshBalance() {
      balance = forceBalanceRefresh();
      console.log('Navbar - Balance manually refreshed:', balance);
    }
  </script>
  
  <!-- Navbar HTML -->
  <div class="h-20 w-full relative">
    <!-- Logo -->
    <a href="/" class="home-link pl-4">
      <img src={logomark} alt="X-Cashu Search Logo" />
    </a>
  
    <!-- Top right info -->
    <div class="absolute top-4 right-4">
      <div class="top-right-info">
        <span class="searches-left">
          Searches left: <span class="searches-count">{balance}</span>
        </span>
        <a href="/topup" class="top-up-button">Top Up</a>
  
        <!-- Theme toggle button -->
        <button
          class="theme-toggle"
          on:click={toggleTheme}
          aria-label="Toggle theme"
        >
          {#if $theme === 'light'}
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
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
            </svg>
          {:else}
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
              <circle cx="12" cy="12" r="5"></circle>
              <line x1="12" y1="1" x2="12" y2="3"></line>
              <line x1="12" y1="21" x2="12" y2="23"></line>
              <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
              <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
              <line x1="1" y1="12" x2="3" y2="12"></line>
              <line x1="21" y1="12" x2="23" y2="12"></line>
              <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
              <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
            </svg>
          {/if}
        </button>
  
        <!-- Dropdown menu -->
        <div class="dropdown-container">
          <button class="more-options-button" on:click={toggleDropdown}>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="currentColor"
            >
              <circle cx="12" cy="12" r="2" />
              <circle cx="12" cy="5" r="2" />
              <circle cx="12" cy="19" r="2" />
            </svg>
          </button>
  
          {#if isDropdownOpen}
            <div class="dropdown-menu" on:blur={() => (isDropdownOpen = false)}>
              <a href="/backup" class="dropdown-item">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="dropdown-icon"
                >
                  <path d="M15 12h-5" />
                  <path d="M15 8h-5" />
                  <path d="M19 17V5a2 2 0 0 0-2-2H4" />
                  <path d="M8 21h12a2 2 0 0 0 2-2v-1a1 1 0 0 0-1-1H11a1 1 0 0 0-1 1v1a2 2 0 1 1-4 0V5a2 2 0 1 0-4 0v2a1 1 0 0 0 1 1h3" />
                </svg>
                Back Up
              </a>
              <a href="/faq" class="dropdown-item">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="dropdown-icon"
                >
                  <circle cx="12" cy="12" r="10"/>
                  <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/>
                  <path d="M12 17h.01"/>
                </svg>
                FAQs
              </a>
              <a href="/recovery" class="dropdown-item">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="dropdown-icon"
                >
                  <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                  <path d="M3 3v5h5" />
                </svg>
                Recovery
              </a>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
  
  <style>
    /* Navbar-specific styles */
  
    .home-link {
      position: absolute;
      top: 1rem;
      left: 1rem;
      z-index: 10;
      display: flex;
      align-items: center;
    }
  
    .home-link img {
      height: 40px;
      width: auto;
    }
  
    .top-right-info {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 8px 16px;
      z-index: 51;
      flex-wrap: wrap;
    }
  
    .searches-left {
      font-weight: 600;
      color: #4a5568;
      background-color: #f3f4f6;
      padding: 8px;
      border-radius: 8px;
      font-size: 16px;
    }
  
    .searches-count {
      font-size: 1.1em;
      color: #1a1a1a;
    }
  
    .top-up-button {
      background-color: transparent;
      color: #4a5568;
      border: 2px solid #4a5568;
      border-radius: 6px;
      padding: 6px 12px;
      font-size: 14px;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.3s ease;
      text-decoration: none;
    }
  
    .top-up-button:hover {
      background-color: #f3f4f6;
      color: #2d3748;
      border-color: #2d3748;
    }
  
    .theme-toggle,
    .more-options-button {
      background: none;
      border: none;
      color: #4a5568;
      padding: 4px;
      cursor: pointer;
      border-radius: 4px;
      transition: background-color 0.2s;
      display: flex;
      align-items: center;
      justify-content: center;
    }
  
    .theme-toggle:hover,
    .more-options-button:hover {
      background-color: #f3f4f6;
    }
  
    .dropdown-container {
      position: relative;
      display: inline-block;
    }
  
    .dropdown-menu {
      position: absolute;
      top: 100%;
      right: 0;
      margin-top: 12px;
      background: white;
      border-radius: 8px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
      min-width: 160px;
      z-index: 52;
      border: 1px solid rgba(0, 0, 0, 0.1);
      padding: 4px 0;
    }
  
    .dropdown-item {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 12px 16px;
      color: #4a5568;
      text-decoration: none;
      transition: all 0.2s ease;
      font-size: 14px;
    }
  
    .dropdown-item:hover {
      background-color: #f3f4f6;
      color: #1a1a1a;
    }
  
    .dropdown-item:first-child {
      border-radius: 8px 8px 0 0;
    }
  
    .dropdown-item:last-child {
      border-radius: 0 0 8px 8px;
    }
  
    /* Dark mode styles */
    :global(.dark) .top-right-info {
      color: #ffffff;
    }
  
    :global(.dark) .searches-left {
      background-color: rgba(255, 255, 255, 0.1);
      color: #ffffff;
    }
  
    :global(.dark) .searches-count {
      color: #ffffff;
    }
  
    :global(.dark) .top-up-button {
      color: #ffffff;
      border-color: #ffffff;
      background-color: #2d2d2d;
    }
  
    :global(.dark) .top-up-button:hover {
      background-color: #3a3a3a;
      border-color: #f0f0f0;
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.4);
    }
  
    :global(.dark) .theme-toggle,
    :global(.dark) .more-options-button {
      color: #ffffff;
    }
  
    :global(.dark) .theme-toggle:hover,
    :global(.dark) .more-options-button:hover {
      background-color: rgba(255, 255, 255, 0.1);
    }
  
    :global(.dark) .dropdown-menu {
      background-color: #2d2d2d;
      border-color: rgba(255, 255, 255, 0.1);
    }
  
    :global(.dark) .dropdown-item {
      color: #ffffff;
    }
  
    :global(.dark) .dropdown-item:hover {
      background-color: rgba(255, 255, 255, 0.1);
    }
  
    /* Mobile adjustments */
    @media (max-width: 640px) {
      .top-right-info {
        padding: 6px;
        gap: 8px;
      }
  
      .searches-left {
        font-size: 14px;
        padding: 6px;
      }
  
      .top-up-button {
        padding: 4px 8px;
        font-size: 12px;
      }
  
      .home-link {
        top: 50%;
        transform: translateY(-50%);
        left: 0.75rem;
        height: auto;
      }
  
      .home-link img {
        height: 32px;
        display: block;
      }
    }
  </style>