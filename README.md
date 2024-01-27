
# Battery notifier

Is a customizable daemon designed to report the battery status of your laptop when using window managers. It can be utilized as a [systemd](https://wiki.archlinux.org/title/systemd) service or managed through the `exec` dispatcher of your preferred window manager/compositor.

## Why?

Many window managers lack dedicated battery status programs. This project aims to fill that gap by providing a fully customizable solution that operates as a daemon, offering both battery status reporting and built-in performance features.

## Configuration

Configuration files should be located in the `$XDG_CONFIG_FILE`. If undefined, the default location is `$HOME/.config`.

```toml
# battery-notifier/config.toml

reminder_threshold = 30
threat_threshold = 15
warn_threshold = 5
interval_ms = 700 # 7s
```

Adjust the values to suit your preferences.

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