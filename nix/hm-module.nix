self: {
  config,
  pkgs,
  lib,
  ...
}:
with lib; let
  inherit (pkgs.stdenv.hostPlatform) system;
  tomlFormat = pkgs.formats.toml {};
  flake-pkgs = self.packages.${system};
in {
  options.programs.battery-notifier = let
    settingsModule = types.submodule {
      options = {
        interval_ms = mkOption {
          type = types.int;
          default = 700;
        };

        reminder_threshold = mkOption {
          type = types.int;
          default = 30;
        };

        warn_threshold = mkOption {
          type = types.int;
          default = 15;
        };

        threat_threshold = mkOption {
          type = types.int;
          default = 5;
        };
      };
    };
  in {
    enable = mkEnableOption "battery-notifier";

    settings = mkOption {
      default = null;
      type = types.nullOr settingsModule;
    };
  };

  config = let
    cfg = config.programs.battery-notifier;
  in
    mkIf cfg.enable {
      assertions = mkIf (cfg.settings != null) [
        {
          assertion = builtins.length (lib.attrsets.attrValues (lib.attrsets.filterAttrs (k: v: lib.strings.hasSuffix k "threshold" && v >= 0 && v <= 100) cfg.settings)) == 0;
          message = "threshold values must be greater equal than 0 and less equal than 100";
        }
        {
          assertion = cfg.settings.reminder_threshold > cfg.settings.warn_threshold;
          message = "'reminder' threshold must be greater than 'warn' threshold";
        }
        {
          assertion = cfg.settings.warn_threshold > cfg.settings.threat_threshold;
          message = "'warn' threshold must be greater than 'threat' threshold";
        }
        {
          assertion = cfg.settings.sleep_ms > 0;
          message = "sleep time must be greater than zero";
        }
      ];

      xdg.configFile = mkIf (cfg.settings != null) {
        "battery-notifier/config.toml".source = tomlFormat.generate "battery-notifier-config" cfg.settings;
      };

      systemd.user.services = {
        battery-notifier = {
          Unit = {
            Description = "A very useful battery notifier for window managers";
          };

          Service = {
            Type = "simple";
            ExecStart = "${flake-pkgs.battery-notifier}/bin/battery-notifier";
            Restart = "on-failure";
          };

          Install = {
            WantedBy = ["default.target"];
          };
        };
      };
    };
}
