<script>
  import { theme } from '$lib/stores/theme';
  import '$lib/styles/darkMode.css';
  import "../app.css";

  $: if (typeof document !== 'undefined') {
    document.body.classList.toggle('dark-mode', $theme === 'dark');
  }
</script>

<div class="layout-container">
    <slot />
</div>

<style>
    .layout-container {
        position: relative;
        min-height: 100vh;
    }
</style>
