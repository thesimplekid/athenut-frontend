{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    let
      # NixOS module (system-independent)
      nixosModule = import ./nix/module.nix { inherit self; };
    in
    {
      nixosModules.default = nixosModule;
    }
    //
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        # secp256k1-sys (C code) must be compiled for wasm32 with clang.
        # The wrapped NixOS clang injects native glibc flags, so use the
        # unwrapped one plus its builtin headers.
        wasmClangEnv = {
          CC_wasm32_unknown_unknown = "${pkgs.llvmPackages.clang-unwrapped}/bin/clang";
          CFLAGS_wasm32_unknown_unknown = "-I${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.lib.versions.major pkgs.llvmPackages.libclang.version}/include";
          AR_wasm32_unknown_unknown = "${pkgs.llvmPackages.bintools-unwrapped}/bin/llvm-ar";
        };

        # The athenut-frontend package: a Leptos (wasm) app bundled with trunk
        # into a static site.
        mkAthenutFrontend = { publicApiUrl ? "" }:
          rustPlatform.buildRustPackage {
            pname = "athenut-frontend";
            version = "0.1.0";

            src = pkgs.lib.cleanSource ./.;

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [
              trunk
              wasm-bindgen-cli
              binaryen
            ];

            # PUBLIC_API_URL is a build-time static env var (option_env!)
            env = {
              PUBLIC_API_URL = publicApiUrl;
              TRUNK_SKIP_VERSION_CHECK = "true";
              TRUNK_OFFLINE = "true";
            } // wasmClangEnv;

            buildPhase = ''
              runHook preBuild
              XDG_CACHE_HOME=$TMPDIR/cache trunk build --release --dist dist
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              cp -r dist $out
              runHook postInstall
            '';

            doCheck = false;

            passthru = {
              override = args: mkAthenutFrontend ({
                inherit publicApiUrl;
              } // args);
            };

            meta = with pkgs.lib; {
              description = "Athenut - Privacy-preserving search engine frontend";
              license = licenses.mit;
              maintainers = [ ];
            };
          };
      in
      {
        packages = {
          athenut-frontend = mkAthenutFrontend { };
          default = self.packages.${system}.athenut-frontend;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # formatting for .nix files
            nixpkgs-fmt

            # rust toolchain with wasm target
            rustToolchain

            # wasm bundling
            trunk
            wasm-bindgen-cli
            binaryen

            # serve the built dist locally
            static-web-server

            # task runner (see justfile)
            just
          ];

          env = wasmClangEnv;

          shellHook = ''
            export TRUNK_SKIP_VERSION_CHECK=true
          '';
        };

        # Keep legacy attribute for `nix develop` compat
        devShell = self.devShells.${system}.default;
      }
    );
}
