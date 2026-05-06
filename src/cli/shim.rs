use crate::{module::nvmd_home, utils::help};
use anyhow::Result;
use std::collections::BTreeSet;

/// Manage executable shims placed in '$NVMD_HOME/bin'.
///
/// Shims allow globally installed tools (like npm packages)
/// to be routed through the currently active Node.js version.
#[derive(clap::Args)]
pub struct Shim {
    #[command(subcommand)]
    command: ShimSubcommand,
}

#[derive(clap::Subcommand)]
enum ShimSubcommand {
    /// Add a shim under $NVMD_HOME/bin
    Add(ShimAdd),
    /// Remove a shim from $NVMD_HOME/bin
    Remove(ShimRemove),
    /// List all shims
    List,
    /// List all shims (alias for 'list')
    Ls,
}

#[derive(clap::Args)]
struct ShimAdd {
    /// shim name
    name: String,
}

#[derive(clap::Args)]
struct ShimRemove {
    /// shim name
    name: String,
}

impl super::Command for Shim {
    fn run(self) -> Result<()> {
        match self.command {
            ShimSubcommand::Add(add) => {
                help::link_package(&add.name)?;
                eprintln!(
                    "{} {}",
                    console::style("✔").green(),
                    format!("Shim '{}' created", &add.name)
                );
            }
            ShimSubcommand::Remove(remove) => {
                help::unlink_package(&remove.name)?;
                eprintln!(
                    "{} {}",
                    console::style("✔").green(),
                    format!("Shim '{}' removed", &remove.name)
                );
            }
            ShimSubcommand::List | ShimSubcommand::Ls => {
                for shim in get_shims()? {
                    eprintln!("{shim}");
                }
            }
        };
        Ok(())
    }
}

fn get_shims() -> Result<BTreeSet<String>> {
    let mut shims = BTreeSet::new();
    let bin_dir = nvmd_home()?.bin_dir();
    for entry in std::fs::read_dir(&bin_dir)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        #[cfg(unix)]
        {
            if name == "nvmd" || name == ".DS_Store" {
                continue;
            }
            shims.insert(name.to_string());
        }

        #[cfg(windows)]
        {
            if let Some(name) = name.strip_suffix(".exe") {
                if name != "nvmd" {
                    shims.insert(name.to_string());
                }
            }
        }
    }
    Ok(shims)
}
