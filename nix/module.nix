{ self }:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.athenut-frontend;
in
{
  options.services.athenut-frontend = {
    enable = lib.mkEnableOption "athenut-frontend, a privacy-preserving search engine frontend";

    publicApiUrl = lib.mkOption {
      type = lib.types.str;
      default = "https://athenut.com";
      description = ''
        The base URL for the Athenut search API backend (also used as the
        default Cashu mint URL).
        Set to "" for relative paths (frontend and backend on same domain).

        Note: This is a build-time setting. Changing it will trigger a rebuild of the package.
      '';
      example = "https://v2.athenut.com";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 3000;
      description = "Port the frontend HTTP server listens on.";
    };

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Address the frontend HTTP server binds to.";
    };

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.system}.athenut-frontend.override {
        publicApiUrl = cfg.publicApiUrl;
      };
      defaultText = lib.literalExpression ''
        self.packages.''${pkgs.system}.athenut-frontend.override {
          publicApiUrl = config.services.athenut-frontend.publicApiUrl;
        }
      '';
      description = "The athenut-frontend package to use.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.athenut-frontend = {
      description = "Athenut Frontend - Privacy-preserving search engine";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      serviceConfig = {
        Type = "simple";
        # The package is a static site (wasm app built with trunk); serve it
        # with static-web-server, falling back to index.html so client-side
        # routes work on reload.
        ExecStart = lib.concatStringsSep " " [
          (lib.getExe pkgs.static-web-server)
          "--host ${cfg.host}"
          "--port ${toString cfg.port}"
          "--root ${cfg.package}"
          "--page-fallback ${cfg.package}/index.html"
        ];
        Restart = "on-failure";
        RestartSec = 5;

        # Security hardening
        DynamicUser = true;
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictSUIDSGID = true;
        RestrictNamespaces = true;
        LockPersonality = true;
        RestrictRealtime = true;
        SystemCallFilter = [ "@system-service" "~@privileged" ];
        MemoryDenyWriteExecute = true;
      };
    };
  };
}
