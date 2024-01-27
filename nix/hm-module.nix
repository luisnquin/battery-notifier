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
    boundModule = types.submodule {
      options = {
        threshold = mkOption {
          type = types.int;
        };

        title = mkOption {
          type = types.str;
          default = "";
        };

        content = mkOption {
          type = types.str;
          default = "";
        };
      };
    };

    settingsModule = types.submodule {
      options = {
        interval_ms = mkOption {
          type = types.int;
          default = 700;
        };

        reminder = mkOption {
          type = boundModule;
          default = {
            threshold = 30;
          };
        };

        warn = mkOption {
          type = boundModule;
          default = {
            threshold = 15;
          };
        };

        threat = mkOption {
          type = boundModule;
          default = {
            threshold = 5;
          };
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
          assertion = let
            greatEq0LowEq100 = v: v >= 0 && v <= 100;
            inherit (cfg.settings) reminder warn threat;
          in
            greatEq0LowEq100 reminder.threshold && greatEq0LowEq100 warn.threshold && greatEq0LowEq100 threat.threshold;
          message = "threshold values must be greater equal than 0 and less equal than 100";
        }
        {
          assertion = cfg.settings.interval_ms > 0;
          message = "'interval_ms' must be greater than zero";
        }
        {
          assertion = cfg.settings.reminder.threshold > cfg.settings.warn.threshold;
          message = "'reminder' threshold must be greater than 'warn' threshold";
        }
        {
          assertion = cfg.settings.warn.threshold > cfg.settings.threat.threshold;
          message = "'warn' threshold must be greater than 'threat' threshold";
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
