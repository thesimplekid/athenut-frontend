<script>
    import { slide, fade, fly, scale } from 'svelte/transition';
    import { quintOut, elasticOut } from 'svelte/easing';
    import { onMount } from 'svelte';
    import { theme } from "$lib/stores/theme";
    import Footer from "../../components/Footer.svelte";
    import Navbar from "../../components/Navbar.svelte";

    let contentReady = false;

    // FAQ data structure
    const faqs = [
      {
        question: "What is Athenut?",
        answer: "Athenut is a privacy-preserving web search powered by Kagi and Cashu, allowing you to perform premium searches while maintaining your privacy."
      },
      {
        question: "How do searches work?",
        answer: "Each search costs 1 token. You can purchase tokens using Bitcoin Lightning Network payments, and these tokens are stored locally in your browser."
      },
      {
        question: "How do I top up my account?",
        answer: "You can top up your account by visiting the Top Up page, selecting the number of searches you want to purchase, and paying the Lightning invoice that appears."
      },
      {
        question: "Are my searches private?",
        answer: "Yes! Athenut uses Cashu tokens for payments, which are completely private. Your search history is not stored on any servers."
      },
      {
        question: "What happens to unused searches?",
        answer: "Your searches remain in your browser wallet until you use them. They don't expire and are stored locally on your device."
      },
      {
        question: "How do I backup my searches?",
        answer: "Your unused searches are stored as tokens in your browser's local storage. You can back them up by saving the 12-word recovery phrase found in the backup page."
      }
    ];
  
    // Track which questions are expanded
    let expandedQuestions = [];
  
    function toggleQuestion(index) {
      if (expandedQuestions.includes(index)) {
        expandedQuestions = expandedQuestions.filter(i => i !== index);
      } else {
        expandedQuestions = [...expandedQuestions, index];
      }
    }

    onMount(() => {
      setTimeout(() => contentReady = true, 100);
    });
  </script>
  
  <svelte:head>
    <title>FAQ - Athenut</title>
    <meta name="description" content="Frequently asked questions about Athenut's privacy-preserving web search." />
  </svelte:head>
  
  <div class="min-h-screen flex flex-col text-gray-800 relative">
    <Navbar />

    <main class="flex-grow flex flex-col items-center px-4 pt-24 pb-8">
      {#if contentReady}
        <div class="w-full max-w-3xl">
          <h1 class="text-4xl font-bold mb-2 text-center" in:fly={{ y: 20, duration: 500, delay: 200, easing: quintOut }}>
            Frequently Asked Questions
          </h1>

          <p class="text-xl text-gray-600 text-center mb-8" in:fly={{ y: 20, duration: 500, delay: 300, easing: quintOut }}>
            Find answers to common questions about using Athenut.
          </p>

          <div class="faq-container" in:fade={{ duration: 400, delay: 400, easing: quintOut }}>
            {#each faqs as faq, index}
              <div class="faq-item">
                <button
                  class="faq-question"
                  class:expanded={expandedQuestions.includes(index)}
                  on:click={() => toggleQuestion(index)}
                >
                  <span>{faq.question}</span>
                  <svg
                    class="arrow-icon"
                    class:rotated={expandedQuestions.includes(index)}
                    xmlns="http://www.w3.org/2000/svg"
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
                      d="M19 9l-7 7-7-7"
                    />
                  </svg>
                </button>
                {#if expandedQuestions.includes(index)}
                  <div class="faq-answer" transition:slide={{ duration: 300, easing: quintOut }}>
                    {faq.answer}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </main>

    <Footer />
  </div>
  
  <style>
    .faq-container {
      width: 100%;
      max-width: 800px;
      margin: 0 auto;
    }
  
    .faq-item {
      margin-bottom: 1rem;
      border-radius: 16px;
      background: rgba(255, 255, 255, 0.6);
      backdrop-filter: blur(12px);
      -webkit-backdrop-filter: blur(12px);
      border: 1px solid rgba(226, 232, 240, 0.6);
      overflow: hidden;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
      transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .faq-item:hover {
      transform: translateY(-2px);
      box-shadow: 0 8px 20px rgba(0, 0, 0, 0.1);
    }
  
    .faq-question {
      width: 100%;
      padding: 1.25rem;
      display: flex;
      justify-content: space-between;
      align-items: center;
      cursor: pointer;
      background: none;
      border: none;
      text-align: left;
      font-weight: 600;
      font-size: 1.1rem;
      color: var(--text-primary, #1f2937);
      transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .faq-question:hover {
      color: #667eea;
    }
  
    .faq-answer {
      padding: 0 1.25rem 1.25rem;
      color: var(--text-secondary);
      line-height: 1.6;
    }
  
    .arrow-icon {
      transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
      color: #667eea;
    }

    .arrow-icon.rotated {
      transform: rotate(180deg);
    }
  
    /* Dark mode styles */
    :global(.dark) .faq-item {
      background: rgba(45, 45, 45, 0.6);
      border-color: rgba(255, 255, 255, 0.1);
    }
  
    :global(.dark) .faq-question {
      color: #ffffff;
    }
  
    :global(.dark) .faq-answer {
      color: #a0aec0;
    }
  
    :global(.dark) .text-gray-600 {
      color: #a0aec0;
    }
  
    :global(.dark) h1 {
      color: #ffffff;
    }
  
    /* Mobile responsiveness */
    @media (max-width: 640px) {
      .faq-question {
        padding: 1rem;
        font-size: 0.95rem;
      }
  
      .faq-answer {
        padding: 0 1rem 1rem;
        font-size: 0.9rem;
      }
    }
  
    /* Add base dark mode variables to match home page */
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
  </style> 