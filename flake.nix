{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
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
        pkgs = nixpkgs.legacyPackages.${system};

        # The athenut-frontend package built with buildNpmPackage
        mkAthenutFrontend = { publicApiUrl ? "" }:
          pkgs.buildNpmPackage {
            pname = "athenut-frontend";
            version = "0.0.1";

            src = pkgs.lib.cleanSource ./.;

            npmDepsHash = "sha256-ib7e13BlJ/yQ+Gpz5u4+tWVJwVdDWwlTIqLuSd3vRYQ=";

            nodejs = pkgs.nodejs_22;
            makeCacheWritable = true;

            # PUBLIC_API_URL is a build-time static env var for SvelteKit
            env.PUBLIC_API_URL = publicApiUrl;

            buildPhase = ''
              runHook preBuild
              npm run build
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall

              mkdir -p $out/lib/athenut-frontend
              cp -r build $out/lib/athenut-frontend/build
              cp package.json $out/lib/athenut-frontend/

              # Copy production node_modules needed at runtime
              cp -r node_modules $out/lib/athenut-frontend/node_modules

              runHook postInstall
            '';

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

        # scripts
        bunbuild = pkgs.writeShellScriptBin "bunbuild" ''
          bun run build
        '';

        bundev = pkgs.writeShellScriptBin "bundev" ''
          bun run dev
        '';

        bunstart = pkgs.writeShellScriptBin "bunstart" ''
          bun run start
        '';

        buntest = pkgs.writeShellScriptBin "buntest" ''
          bun run test
        '';
      in
      with pkgs; {
        packages = {
          athenut-frontend = mkAthenutFrontend { };
          default = self.packages.${system}.athenut-frontend;
        };

        devShell = mkShell {
          buildInputs = [
            # list whatever packages you need
            # search for packages at https://search.nixos.org/

            # formatting for .nix files
            nixpkgs-fmt

            # binaries
            nodejs_22
            bun
            playwright-driver.browsers # node package version and this must match

            # custom scripts defined above
            bunbuild
            bundev
            bunstart
            buntest

            nodePackages.svelte-language-server
          ];

          shellHook = ''
            export PLAYWRIGHT_BROWSERS_PATH=${pkgs.playwright-driver.browsers}

            bun install
          '';
        };
      }
    );
}
