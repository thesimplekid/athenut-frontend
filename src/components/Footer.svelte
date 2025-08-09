<script>
  import { theme } from "$lib/stores/theme";
</script>

<footer
  class="footer {$theme === 'dark' ? 'dark' : ''} {$theme === 'dark'
    ? 'dark-mode'
    : ''}"
>
  <div class="horizontal-line-container">
    <div class="fading-line"></div>
  </div>

  <div class="footer-content">
    <div class="footer-left">
      <div class="footer-text">
        <span class="footer-brand-name">Athenut</span>
      </div>
      <p class="footer-description">
        &copy; {new Date().getFullYear()} All rights reserved.
      </p>
    </div>
    <div class="footer-right">
      <a
        href="https://github.com/thesimplekid/athenut-mint"
        target="_blank"
        rel="noopener noreferrer"
        class="github-link"
      >
        <svg
          height="32"
          aria-hidden="true"
          viewBox="0 0 16 16"
          version="1.1"
          width="32"
          data-view-component="true"
          class="github-icon"
          fill={$theme === "dark" ? "#ffffff" : "#000000"}
        >
          <path
            fill-rule="evenodd"
            d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"
          ></path>
        </svg>
      </a>
    </div>
  </div>
  <!-- Search count removed to focus on UI fixes only -->
  <div class="footer-disclaimer">
    Athenut is experimental and in beta. Use with caution.
  </div>
</footer>

<style>
  .footer {
    background-color: #ffffff;
    color: #1a1a1a;
    padding: 32px 0;
    font-weight: 400;
    position: relative;
    margin-top: 0;
  }

  .footer-content {
    max-width: 1200px;
    margin: 0 auto;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 0 24px;
  }

  .footer-left {
    max-width: 300px;
  }

  .footer-text {
    margin-bottom: 16px;
  }

  .footer-brand-name {
    font-size: 18px;
    font-weight: 500;
    color: #1a1a1a;
  }

  .footer-description {
    font-size: 14px;
    line-height: 1.5;
    color: #4a5568;
  }

  .footer-right {
    display: flex;
    align-items: center;
  }

  .github-link {
    display: flex;
    align-items: center;
    justify-content: center;
    transition: opacity 0.3s ease;
  }

  .github-link:hover {
    opacity: 0.7;
  }

  .github-icon {
    /* Keep only non-color related styles if any */
  }

  .footer-disclaimer {
    text-align: center;
    font-size: 12px;
    color: #6b7280;
    margin-top: 16px;
    padding: 0 24px;
  }

  .horizontal-line-container {
    width: 100%;
    display: flex;
    justify-content: center;
    margin-bottom: 32px;
  }

  .fading-line {
    width: 80%;
    max-width: 1000px;
    height: 1px;
    background: linear-gradient(
      to right,
      transparent,
      rgba(229, 231, 235, 1) 20%,
      rgba(229, 231, 235, 1) 80%,
      transparent
    );
  }

  .footer.dark .fading-line {
    background: linear-gradient(
      to right,
      transparent,
      rgba(255, 255, 255, 0.05) 20%,
      rgba(255, 255, 255, 0.05) 80%,
      transparent
    );
  }

  .footer.dark {
    background-color: #1a1a1a;
    border-top-color: #2d2d2d;
  }

  .footer.dark .footer-brand-name {
    color: #ffffff;
  }

  .footer.dark .footer-description {
    color: #a0aec0;
  }

  /* Removed unused footer-links selectors */

  /* Removed .footer.dark .separator - not used in the template */

  .search-counter {
    text-align: center;
    font-size: 14px;
    color: #6b7280;
    margin-top: 12px;
    padding: 0 24px;
    font-weight: 500;
  }

  .footer.dark .search-counter {
    color: #a0aec0;
  }
</style>
