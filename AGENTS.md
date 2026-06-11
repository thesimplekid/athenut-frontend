# AGENTS.md

This file provides guidelines for agentic coding agents operating in this repository.

## Project Overview

Athenut is a privacy-preserving web search frontend powered by Kagi and Cashu ecash tokens. It is a client-side rendered Rust WebAssembly app built with [Leptos](https://book.leptos.dev/), using [CDK](https://docs.rs/cdk) for the Cashu wallet and [Trunk](https://trunkrs.dev/) for bundling.

## Build/Lint/Test Commands

All tooling comes from the Nix dev shell — there are no globally installed toolchains:

```bash
nix develop                                      # enter the dev shell
trunk serve                                      # dev server (localhost:5173)
trunk build                                      # development build into dist/
trunk build --release                            # production build
cargo check --target wasm32-unknown-unknown      # type-check
cargo test                                       # native test suite (cdk WalletDatabase conformance + unit tests)
cargo clippy --target wasm32-unknown-unknown     # lint
cargo fmt                                        # format
nix build .#athenut-frontend                     # reproducible package build
```

One-off commands can be run without entering the shell: `nix develop -c cargo check --target wasm32-unknown-unknown`.

A `justfile` wraps these (each recipe runs via `nix develop -c`, so it works from any shell): `just dev`, `just build`, `just release`, `just check`, `just test`, `just clippy`, `just fmt`, `just nix-build`, `just serve`.

Note: this is a jj (Jujutsu) colocated repo. Prefer `jj` commands for VCS work. For `nix build` to see new files they must be visible to git (`git add` the paths, or build with `nix build path:.#...`).

## Architecture

```
index.html            # Trunk entry point (assets, css, wasm)
Trunk.toml            # Trunk config
styles/main.css       # All styling (global vars, utilities, per-page sections)
assets/               # Images (wordmark, logomark)
static/               # favicon.ico, opensearch.xml (copied into dist/)
src/
├── main.rs           # App component, router, routes
├── state.rs          # AppState (theme/toast/balance signals)
├── storage.rs        # localStorage/sessionStorage helpers
├── db.rs             # CDK WalletDatabase impl persisted to localStorage
├── wallet.rs         # Wallet construction + cashu operations (cdk)
├── components/       # Navbar, Footer, Toast + clipboard helpers
└── pages/            # home, search, topup, backup, recovery, faq
nix/module.nix        # NixOS module (serves dist/ via static-web-server)
```

### Key design points

- **Wallet**: `cdk::wallet::Wallet` with the currency unit `xsr` (1 token = 1 search) and denomination-1 proofs (`SplitTarget::Value(1)`). The seed phrase lives in localStorage under `seed` (bip39, 12 words).
- **Database**: `src/db.rs` implements `cdk_common::database::WalletDatabase` over a single localStorage key (`cdk_wallet_db`). The store is re-read on every access, and writes are read-modify-write cycles under a cross-tab Web Lock (`navigator.locks`), so multiple open tabs share one consistent wallet; search proofs are claimed atomically (`take_unspent_proof`) so two tabs never spend the same proof, and a `storage` event listener in `main.rs` keeps the balance live across tabs. Swapping in a different persistence layer (e.g. IndexedDB) only requires a new impl of the same trait.
- **Search payment**: one unspent proof is encoded as a cashu token and sent in the `X-Cashu` request header. On success it is marked spent; on a network failure it is returned to the unspent pool; on HTTP 402 (and for proofs left `PendingSpent` by an interrupted search) `wallet::reclaim_pending_proofs` asks the mint which proofs were actually redeemed — spent ones are dropped, the rest are swapped into fresh proofs (the old token already left the browser, so it is rotated rather than reused).
- **Tests**: `cargo test` runs natively — `storage.rs` swaps in a thread-local in-memory backend off wasm, so `db.rs` is exercised by cdk's own `wallet_db_test!` conformance suite plus local unit tests.
- **`PUBLIC_API_URL`**: compile-time env var (`option_env!`), empty = same origin. The mint URL defaults to the page origin and can be overridden via the `mint_url` localStorage key.
- **Legacy migration**: proofs written by the old cashu-ts app (localStorage `proofs` key) are imported into the cdk database on startup.

## Code Style Guidelines

- Rust 2021, `cargo fmt` formatting, no `unsafe`.
- Components are `#[component]` functions in PascalCase; pages live in `src/pages/`, one file per route.
- State flows through `AppState` (Copy struct of `RwSignal`s) provided via Leptos context; read it with `expect_context::<AppState>()`.
- Async work uses `leptos::task::spawn_local`; errors surface to the user via `state.show_toast(...)`, returning `Result<_, String>` from wallet helpers.
- Styling lives in `styles/main.css`. Page-specific rules are namespaced under a `.page-*` wrapper class (e.g. `.page-topup .spinner`). Theme colors use CSS custom properties (`--bg-primary`, `--text-primary`, ...) with dark mode driven by a `dark` class on `<html>`.
- Entry animations use the `.anim-*` classes with `--anim-delay` for staggering.
- Transitions: `cubic-bezier(0.4, 0, 0.2, 1)`; respect `prefers-reduced-motion`.
