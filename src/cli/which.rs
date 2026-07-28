use crate::module::{NodeVersionResolver, Setting};
use anyhow::{Result, bail};

#[derive(clap::Args)]
pub struct Which {
    /// The version number of Node.js
    version: String,
}

impl super::Command for Which {
    fn run(self) -> Result<()> {
        let version = NodeVersionResolver::resolve(&self.version)?;
        let mut path = Setting::global()?.get_directory()?.join(&version);
        if cfg!(unix) {
            path.push("bin");
        }

        if path.exists() {
            eprintln!("{:?}", path);
        } else {
            bail!("Node@v{} has not been installed", &version);
        }

        Ok(())
    }
}
