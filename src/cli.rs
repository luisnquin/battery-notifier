use anstyle::{AnsiColor, Color, Style};
use clap::{builder::Styles, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(styles=get_styles())]
/// A customizable battery notifier for Linux kernels focused in BAT0 and BAT1
pub struct Args {
    #[arg(short, long)]
    /// To simulate battery states (yaml).
    pub debug_file: Option<String>,
    /// The config file path (toml).
    #[arg(short, long)]
    pub config_file: Option<String>,
}

fn get_styles() -> Styles {
    Styles::styled()
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::BrightCyan))),
        )
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::BrightCyan))),
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Black))))
}
