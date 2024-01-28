use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    /// To simulate battery states (yaml).
    pub debug_file: Option<String>,
    /// The config file path (toml).
    #[arg(short, long)]
    pub config_file: Option<String>,
}
