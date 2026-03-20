# AGENTS.md

This file provides guidelines for agentic coding agents operating in this repository.

## Project Overview

Athenut is a privacy-preserving web search frontend powered by Kagi and Cashu ecash tokens. Built with SvelteKit, Tailwind CSS, and Vite.

## Build/Lint/Test Commands

### Development

```bash
npm run dev        # Start dev server (localhost:5173)
npm run build     # Production build
npm run preview   # Preview production build
```

### Type Checking

```bash
npm run check          # Run svelte-check (type checking)
npm run check:watch   # Watch mode for type checking
```

### Linting & Formatting

```bash
npm run lint      # Check formatting (prettier) and linting (eslint)
npm run format    # Auto-fix formatting
```

### Testing

```bash
# No formal test suite exists yet (no Jest tests found in codebase)
# For manual testing, use the dev server: npm run dev
```

### Nix Development (recommended)

```bash
nix develop       # Enter dev shell with Node.js, Bun
bundev            # Start dev server
bunbuild          # Production build
bunstart          # Run production build
buntest           # Run tests
```

### Single File/Component Work

```bash
# Format a single file with prettier
npx prettier --write src/routes/+page.svelte

# Type-check a single file
npx svelte-check --tsconfig ./jsconfig.json src/routes/+page.svelte
```

## Code Style Guidelines

### General

- This is a **SvelteKit** application with **Tailwind CSS** for styling
- Uses **JS with JSDoc annotations** for type safety (not .ts files, but jsconfig.json with strict mode enabled)
- JavaScript files use **ES modules** (`"type": "module"` in package.json)

### Formatting

- **Prettier** handles formatting with the `prettier-plugin-svelte` plugin
- Svelte files use **2-space indentation**
- CSS within `<style>` blocks uses **2-space indentation**
- Trailing commas in multiline expressions
- Single quotes for strings in JavaScript

### Naming Conventions

- **Variables/functions**: camelCase (`getBalance`, `forceBalanceRefresh`)
- **Constants**: camelCase or SCREAMING_SNAKE_CASE as appropriate
- **Svelte components**: PascalCase (`Navbar.svelte`, `Footer.svelte`)
- **Files**: kebab-case for utilities (`utils.js`, `theme.js`)
- **CSS classes**: kebab-case in HTML, camelCase in CSS-in-JS style blocks

### Svelte Component Structure

```svelte
<script>
  import { onMount } from "svelte";
  import { fade, fly } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';

  export let propName = defaultValue;  // Props use export

  let localVar = value;
  // ... script logic
</script>

<!-- HTML template -->

<style>
  /* Scoped component styles */
  .component-class {
    /* Use CSS custom properties for theming */
    color: var(--text-primary);
  }
</style>
```

### Styling Conventions

- **Tailwind CSS** for utility classes in templates
- **CSS custom properties** for theme colors (defined globally in `:root` and `.dark`)
- **Transitions**: Use `cubic-bezier(0.4, 0, 0.2, 1)` for smooth animations
- **Dark mode**: Apply `.dark` class to `html` element; target with `:global(.dark) .selector`
- Prefer `backdrop-filter: blur()` for glass effects
- Use `@media (prefers-reduced-motion: reduce)` for accessibility

### Theme/CSS Custom Properties

```css
:global(:root) {
  --bg-primary: #ffffff;
  --bg-secondary: #f7f7f7;
  --text-primary: #1a1a1a;
  --text-secondary: #4a5568;
}

:global(.dark) {
  --bg-primary: #1a1a1a;
  --bg-secondary: #2d2d2d;
  --text-primary: #ffffff;
  --text-secondary: #a0aec0;
}
```

### Store Patterns

- Svelte stores in `$lib/stores/`
- Use `writable` from `svelte/store`
- Browser detection via `import { browser } from '$app/environment'`
- Example:

```javascript
import { writable } from 'svelte/store';
import { browser } from '$app/environment';

const initialValue = browser ? localStorage.getItem('key') || default : default;
export const store = writable(initialValue);

if (browser) {
  store.subscribe(value => {
    localStorage.setItem('key', value);
  });
}
```

### Utility Functions

- Located in `$lib/shared/utils.js`
- Use JSDoc for type annotations
- Wrap localStorage access in try/catch
- Return sensible defaults (empty array, 0, etc.) on error

### Error Handling

- Wrap localStorage/JSON.parse in try/catch
- Log errors with `console.error()`
- Return default values on failure (don't throw)
- Use `console.log()` for debugging (remove before committing)

### Imports

- SvelteKit built-ins: `$app/environment`, `$app/navigation`, `$app/stores`
- Library imports: Use package names directly
- Local imports: Use `$lib/` alias for `src/lib/`
- Static assets: Use absolute paths like `/src/asset.png`

### Transitions & Animations

- Use Svelte's built-in transitions: `fade`, `fly`, `scale`
- Easing: `quintOut` for smooth deceleration
- Typical timing: 150-400ms for UI transitions
- Store transitions on components, not on elements with backdrop-filter

### SvelteKit Routing

- Pages: `+page.svelte` (component) and `+page.js` (load function)
- Layouts: `+layout.svelte` and `+layout.js`
- Disable SSR when needed: `export const ssr = false;` in `+page.js`
- Route-specific data loading via `+page.js` load functions

### File Organization

```
src/
├── lib/
│   ├── stores/          # Svelte stores
│   │   ├── theme.js
│   │   ├── toast.js
│   │   └── wallet.js
│   └── shared/
│       └── utils.js     # Shared utilities
├── routes/
│   ├── +layout.svelte   # Root layout
│   ├── +page.svelte     # Home page
│   ├── search/          # Search route
│   ├── topup/           # Top up route
│   └── ...
├── app.css              # Global styles
├── app.html             # HTML template
└── app.d.ts             # TypeScript definitions
```

### Type Checking

- Uses `jsconfig.json` extending `.svelte-kit/tsconfig.json`
- `strict: true` enabled
- `allowJs: true` and `checkJs: true` for gradual typing
- Use JSDoc `@type` annotations for types
- Use `@param` and `@returns` for function documentation

### Environment Variables

- Prefix public variables with `PUBLIC_` (exposed to client)
- Use `import { env } from '$env/dynamic/public'` for runtime public env vars
- Copy `.env.example` to `.env` for local development

### Common Patterns

#### Event Handlers

```javascript
function handleEvent(e) {
  // Handle event
}
```

Use `on:eventname={handler}` in templates (Svelte 4 syntax).

#### Reactive Statements

```javascript
$: doubled = count * 2; // Reactive statement
```

#### Async Operations

```javascript
onMount(async () => {
  const data = await fetchData();
  // ...
});
```
