
# Battery notifier

Is a customizable daemon designed to report the battery status of your laptop when using window managers. It can be utilized as a [systemd](https://wiki.archlinux.org/title/systemd) service or managed through the `exec` dispatcher of your preferred window manager/compositor.

## Why?

Many window managers lack dedicated battery status programs. This project aims to fill that gap by providing a fully customizable solution that operates as a daemon, offering both battery status reporting and built-in performance features.

## Configuration

Configuration files should be located in the `$XDG_CONFIG_FILE`. If undefined, the default location is `$HOME/.config`.

```toml
# battery-notifier/config.toml

interval_ms = 700 # 7s

[reminder]
threshold = 30
title = "Battery somewhat low"
content = "Battery capacity is at ${{capacity}}%.\nConsider plugging in your laptop to avoid running out of power."

[warn]
threshold = 15
title = "Battery low"
content = "Battery capacity is critically low at ${{capacity}}%.\nPlease plug in your laptop."

[threat]
threshold = 5
title = "Battery in critical state"
content = "Battery capacity is extremely low at ${{capacity}}%.\nConnect your laptop to a power source urgently to prevent data loss and unexpected shutdown."
```

Adjust the values to suit your preferences.

## Installation via Home Manager

If you use [Home Manager](https://github.com/nix-community/home-manager) to manage your user environment, integrating the battery notifier into your configuration is straightforward.

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    home-manager.url = "github:nix-community/home-manager";
    battery-notifier.url = "github:luisnquin/battery-notifier";
  };

  outputs = inputs @ {
    self,
    home-manager,
    battery-notifier,
    nixpkgs,
    ...
  }: let
    system = "x86_64-linux";
    username = "xyz";

    pkgs = import nixpkgs {inherit system;};
  in {
    homeConfigurations.${username} = home-manager.lib.homeManagerConfiguration {
      inherit pkgs;

      modules = [
        battery-notifier.homeManagerModule.default
        {
          programs.battery-notifier = {
            enable = true;
            settings = {
              interval_ms = 700;
              reminder = {threshold = 30;};
              threat = {threshold = 5;};
              warn = {threshold = 15;};
            };
          };
        }
      ];
    };
  };
}
```


## Development

To develop and contribute to the project, use standard Cargo commands such as **build**, **run**, and **add**.

### Debugging

Almost always you'll need to check that some behaviors are working as expected or not.
For this you can create or modify a [debug file](./debug.yaml) and pass it via CLI arguments.

```sh
# Start the program using the debug file as a mock.
$ cargo run --debug-file=./debug.yaml
```

Feel free to create a new pull request if you see something to improve.


## License

[MIT](./.github/LICENSE)