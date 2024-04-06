
# Battery notifier

Is a customizable daemon designed to report the battery status of your laptop when using window managers. It can be utilized as a [systemd](https://wiki.archlinux.org/title/systemd) service or managed through the `exec` dispatcher of your preferred window manager/compositor.

![Demo](./.github/assets/demo.gif)

This project follows the [Power supply class specification](https://docs.kernel.org/power/power_supply_class.html#attributes-properties-detailed) defined in the [Linux Kernel Documentation](https://docs.kernel.org/).

## Features

- **Lightweight**: Minimal impact on system resources (3.75 MiB of consumption on my computer).
- **Configurable notification levels**: Customize three different notification levels – *reminder*, *warning*, and *threat*.
- **Adjustable check interval:** Set the check interval to your liking, ensuring timely updates on your battery status.
- **Custom notification icon**: Choose your preferred icon.
- **Good configuration defaults**: Comes with well-considered default settings.

## Why?

Window managers lack dedicated programs to notify the battery status of your computer. This project aims to fill that gap by providing a fully customizable solution that operates as a daemon, offering both battery status reporting and built-in performance features.

## Configuration

Configuration files should be located in the `$XDG_CONFIG_FILE`. If undefined, the default location is `$HOME/.config`.

```toml
# battery-notifier/config.toml

interval_ms = 700 # 7s
icon_path = "/absolute/path/to/alternative/icon"

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

## Installation

<details open>
<summary><b>Ubuntu</b></summary>
<br>

```sh
$ git clone https://github.com/luisnquin/battery-notifier.git
$ cd battery-notifier
# Install necessary build dependencies.
$ apt update && apt install cmake g++ cargo -y
# Install the program binary. Default location is $HOME/.cargo/bin.
$ cargo install --path .
# Install systemd unit in your computer
$ mkdir -p $HOME/.config/systemd/user/
$ sed 's#ExecStart=battery-notifier#ExecStart=$HOME/.cargo/bin/battery-notifier#' systemd/battery-notifier.service > "$HOME/.config/systemd/user/battery-notifier.service"
$ systemctl --user enable battery-notifier.service
```

</details>

<details>
<summary><b>Home Manager</b/></summary>

If you use [Home Manager](https://github.com/nix-community/home-manager) to manage your user environment, integrating the battery notifier into your configuration is straightforward.

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    battery-notifier = {
      url = "github:luisnquin/battery-notifier";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
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
          services.battery-notifier = {
            enable = true;
            settings = {
              icon_path = ../assets/icons/battery-notifier.png; # Nix path
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

</details>

<details>
<summary><b>NixOS</b/></summary>

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    battery-notifier = {
      url = "github:luisnquin/battery-notifier";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    battery-notifier,
    nixpkgs,
    ...
  }: let
    system = "x86_64-linux";
    hostname = "nixos";

    pkgs = import nixpkgs {inherit system;};
  in {
    nixosConfigurations."${hostname}" = nixpkgs.lib.nixosSystem {
      inherit pkgs;

      modules = [
        battery-notifier.nixosModules.default
        {
          services.battery-notifier = {
            enable = true;
            settings = {
              icon_path = ../assets/icons/battery-notifier.png; # Nix path
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

</details>

## Nix options reference

### `services.battery-notifier` module reference

Applicable for the supplied home-manager and nixos configuration options.

#### `services.battery-notifier.enable`

**Type:** [Boolean](https://nixos.org/manual/nix/stable/language/values#type-boolean)

To enable the battery-notifier systemd service. It already has its own defaults so this can be enough for you.

#### `services.battery-notifier.settings`

**Type:** [Attribute set](https://nixos.org/manual/nix/stable/language/values#attribute-set)

User preferences of the program.

#### `services.battery-notifier.settings.interval_ms`

**Type:** [Number](https://nixos.org/manual/nix/stable/language/values#type-number)

The number of milliseconds the program will wait to check again your **BAT(0|1)** file.

#### `services.battery-notifier.settings.icon_path`

**Type:** [Nix path](https://nixos.org/manual/nix/stable/language/values#type-path) or [String](https://nixos.org/manual/nix/stable/language/values#type-string)

Absolute path to the icon to use in the notification message.

#### `services.battery-notifier.settings.<bound>`

**Type:** [Attribute set](https://nixos.org/manual/nix/stable/language/values#attribute-set)

Settings for the different alert levels whose respectively are:

- **reminder** [1st alert]
- **warn** [2nd alert]
- **threat** [final alert]

Ensure consistent threshold values across all levels, or the module assertion **will fail**.

#### `services.battery-notifier.settings.<bound>.threshold`

**Type:** [Number](https://nixos.org/manual/nix/stable/language/values#type-number)

Number between **0 and 100** (careful) that will determine the capacity of the computer and whether it has just entered or exited.

#### `services.battery-notifier.settings.<bound>.title`

**Type:** [String](https://nixos.org/manual/nix/stable/language/values#type-string)

Title of the notification message displayed when the battery enters a specific **bound**.

#### `services.battery-notifier.settings.<bound>.content`

Content of the notification message displayed when the battery enters a specific **bound**.

## CLI reference

```text
A customizable battery notifier for Linux kernels focused in BAT0 and BAT1

Usage: battery-notifier [OPTIONS]

Options:
  -d, --debug-file <DEBUG_FILE>    To simulate battery states (yaml)
  -c, --config-file <CONFIG_FILE>  The config file path (toml)
  -h, --help                       Print help
  -V, --version                    Print version
```

## Development

To develop and contribute to the project, use standard Cargo commands such as **build**, **run**, and **add**.

### Debugging

Almost always you'll need to check that some behaviors are working as expected or not.
For this you can create or modify a [debug file](./debug.yaml) and pass it via CLI arguments.

```sh
# Start the program using the debug file as a mock.
$ cargo run -- --debug-file=./debug.yaml
```

This command serves as a **manual test suite**, so after any changes, ensure to run the program using the original debug file.

## Troubleshooting

- **I'm not receiving audio alerts**: Check that the [soloud-rs](https://github.com/MoAlyousef/soloud-rs?tab=readme-ov-file#backends) package is being compiled with the
audio backend that you're using. **By default** soloud-rs is compiled to only use [miniaudio](https://miniaud.io/).

## Contributing

Feel free to create a new issue or pull request if you see something to improve.

Anyway, this project uses [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/), so please align with that.

## License

[MIT](./.github/LICENSE)
