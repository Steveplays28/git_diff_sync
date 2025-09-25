use std::sync::OnceLock;

use clap::{Parser, Subcommand, crate_version};

pub static ARGUMENTS: OnceLock<Arguments> = OnceLock::new();

#[derive(Debug, Parser)]
#[command(version = crate_version!(), about, long_about = None, arg_required_else_help = true)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Pushes the current diff to the configured Git Diff Sync server.
    Push,
    /// Pulls the most recent diff from the configured Git Diff Sync server.
    Pull,
}

pub fn parse() {
    let _ = ARGUMENTS.set(Arguments::parse());
}
