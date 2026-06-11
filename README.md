# Athenut Frontend

A privacy-preserving web search engine frontend powered by [Kagi](https://kagi.com/) search results and [Cashu](https://cashu.space/) ecash tokens. Users pay for searches with Bitcoin Lightning-funded Cashu tokens -- no accounts, no tracking.

Backend: [athenut-mint](https://github.com/thesimplekid/athenut-mint)

## How it works

1. Top up your balance by paying a Lightning invoice, which mints Cashu tokens stored in your browser
2. Each search spends one token, sent to the backend in an `X-Cashu` header
3. The backend validates the token and returns search results
4. No accounts, no cookies, no server-side state

## Development

This project is a Rust WebAssembly app built with [Leptos](https://book.leptos.dev/) (client-side rendered), using [CDK](https://docs.rs/cdk) for the Cashu wallet and [Trunk](https://trunkrs.dev/) for bundling. Wallet state is persisted in the browser's localStorage behind CDK's `WalletDatabase` trait.

### With Nix (recommended)

Enter the dev shell, which provides the Rust toolchain (with the `wasm32-unknown-unknown` target), Trunk, and wasm tooling:

```bash
nix develop
```

Then:

```bash
trunk serve    # start dev server (localhost:5173)
trunk build    # development build into dist/
trunk build --release   # production build
cargo check --target wasm32-unknown-unknown   # type-check
cargo test     # native test suite (cdk WalletDatabase conformance + unit tests)
```

### With just

A [justfile](justfile) wraps the common commands (each recipe runs inside the dev shell, so they work from any shell):

```bash
just            # list recipes
just dev        # trunk serve (localhost:5173)
just build      # development build
just release    # production build
just check      # cargo check (wasm target)
just test       # native test suite
just clippy     # lint
just fmt        # format
just nix-build  # nix build .#athenut-frontend
just serve      # serve dist/ with SPA fallback on :3000
```

### Configuration

| Variable         | Default | Description                                                                             |
| ---------------- | ------- | --------------------------------------------------------------------------------------- |
| `PUBLIC_API_URL` | `""`    | Backend API URL. Empty string means same-origin (frontend and backend on same domain). |

`PUBLIC_API_URL` is read at compile time (`option_env!`), so export it before running trunk:

```bash
PUBLIC_API_URL="https://athenut.com" trunk serve
```

## Building with Nix

Build the package (a static site in the store path):

```bash
nix build .#athenut-frontend
```

Serve the result locally:

```bash
static-web-server --port 3000 --root ./result --page-fallback ./result/index.html
```

## NixOS module

The flake exports a NixOS module at `nixosModules.default` that serves the built static site as a systemd service (via `static-web-server`).

### Basic usage

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    athenut-frontend.url = "github:thesimplekid/athenut-frontend";
  };

  outputs = { nixpkgs, athenut-frontend, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        athenut-frontend.nixosModules.default
        {
          services.athenut-frontend = {
            enable = true;
            publicApiUrl = "https://v2.athenut.com";
          };
        }
      ];
    };
  };
}
```

### Module options

| Option         | Type    | Default           | Description                                               |
| -------------- | ------- | ----------------- | --------------------------------------------------------- |
| `enable`       | bool    | `false`           | Enable the athenut-frontend service                       |
| `publicApiUrl` | string  | `"https://athenut.com"` | Backend API URL, also the default mint URL (build-time; changing triggers a rebuild). Set to `""` for same-origin. |
| `port`         | port    | `3000`            | Port the HTTP server listens on                           |
| `host`         | string  | `"127.0.0.1"`     | Address the server binds to                               |
| `package`      | package | built from source | Override the package derivation                           |

### With a reverse proxy

The service binds to `127.0.0.1:3000` by default. A typical setup puts nginx or caddy in front:

```nix
services.athenut-frontend = {
  enable = true;
  publicApiUrl = "";  # same origin -- proxy handles both frontend and backend
};

services.nginx.virtualHosts."search.example.com" = {
  forceSSL = true;
  enableACME = true;
  locations."/" = {
    proxyPass = "http://127.0.0.1:3000";
    proxyWebsockets = true;
  };
};
```

## License

[MIT](LICENSE)
