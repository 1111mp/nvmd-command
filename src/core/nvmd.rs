use crate::cli::Subcommand;
use anyhow::Result;
use clap::Parser;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::{env, process::ExitStatus};

#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = "command tools for nvm-desktop",
    long_about = None,
    styles = CLAP_STYLING
)]
#[command(help_template = "\
{before-help}{name} ({version})
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
")]
struct Cli {
    #[command(subcommand)]
    command: Option<Subcommand>,
}

pub(super) fn command() -> Result<ExitStatus> {
    Cli::parse()
        .command
        .map_or(Ok(()), |subcommand| subcommand.run())
        .map(|_| ExitStatus::from_raw(0))
}

// See also `clap_cargo::style::CLAP_STYLING`
const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);
