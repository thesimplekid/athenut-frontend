<script>
    import { toast } from '$lib/stores/toast';
    import { fly } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
</script>

{#if $toast.visible}
    <div
        class="toast-container"
        transition:fly={{ y: -20, duration: 400, easing: quintOut }}
    >
        <div class="toast-message">
            {$toast.message}
        </div>
    </div>
{/if}

<style>
    .toast-container {
        position: fixed;
        top: 5rem;
        left: 50%;
        transform: translateX(-50%);
        background: rgba(255, 255, 255, 0.95);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        color: #1a1a1a;
        padding: 1rem 1.5rem;
        border-radius: 12px;
        z-index: 9999;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        border: 1px solid rgba(0, 0, 0, 0.08);
        min-width: 200px;
        max-width: 90vw;
        text-align: center;
    }

    :global(.dark) .toast-container {
        background: rgba(45, 45, 45, 0.95);
        color: #ffffff;
        border-color: rgba(255, 255, 255, 0.1);
    }

    .toast-message {
        font-size: 0.875rem;
        font-weight: 500;
        line-height: 1.4;
    }
</style> 