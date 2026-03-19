# Athenut Frontend

A privacy-preserving web search engine frontend powered by [Kagi](https://kagi.com/) search results and [Cashu](https://cashu.space/) ecash tokens. Users pay for searches with Bitcoin Lightning-funded Cashu tokens -- no accounts, no tracking.

Backend: [athenut-mint](https://github.com/thesimplekid/athenut-mint)

## How it works

1. Top up your balance by paying a Lightning invoice, which mints Cashu tokens stored in your browser
2. Each search spends one token, sent to the backend in an `X-Cashu` header
3. The backend validates the token and returns search results
4. No accounts, no cookies, no server-side state

## Development

This project uses [SvelteKit](https://kit.svelte.dev/) with the Node.js adapter, [Tailwind CSS](https://tailwindcss.com/), and [Vite](https://vitejs.dev/).

### With Nix (recommended)

Enter the dev shell, which provides Node.js, Bun, and helper scripts:

```bash
nix develop
```

Then use the helper commands:

```bash
bundev     # start dev server (localhost:5173)
bunbuild   # production build
bunstart   # run production build
buntest    # run tests
```

### Without Nix

```bash
npm install
npm run dev       # start dev server
npm run build     # production build
npm run preview   # preview production build
```

### Configuration

Copy `.env.example` to `.env` and set `PUBLIC_API_URL`:

```bash
cp .env.example .env
```

| Variable         | Default | Description                                                                            |
| ---------------- | ------- | -------------------------------------------------------------------------------------- |
| `PUBLIC_API_URL` | `""`    | Backend API URL. Empty string means same-origin (frontend and backend on same domain). |

## Building with Nix

Build the package:

```bash
nix build .#athenut-frontend
```

Run the built server directly:

```bash
node ./result/lib/athenut-frontend/build/index.js
```

The server listens on `http://0.0.0.0:3000` by default. Control this with `PORT` and `HOST` environment variables.

## NixOS module

The flake exports a NixOS module at `nixosModules.default` that runs athenut-frontend as a systemd service.

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
| `publicApiUrl` | string  | `""`              | Backend API URL (build-time; changing triggers a rebuild) |
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
