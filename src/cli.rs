use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "NetDout download daemon and CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run HTTP API daemon for browser extension integration
    Daemon,
    /// Queue a single download from CLI
    Download { url: String, output: String },
    /// Check state of a given download id
    Status { id: String },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
