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
      default = "";
      description = ''
        The base URL for the Athenut search API backend.
        Leave empty for relative paths (frontend and backend on same domain).
        Set to a full URL like "https://api.example.com" for a separate backend.

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

      environment = {
        PORT = toString cfg.port;
        HOST = cfg.host;
        NODE_ENV = "production";
      };

      serviceConfig = {
        Type = "simple";
        ExecStart = "${lib.getExe pkgs.nodejs_22} ${cfg.package}/lib/athenut-frontend/build/index.js";
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
        MemoryDenyWriteExecute = false; # Node.js JIT needs this
      };
    };
  };
}
