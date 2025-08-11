use crate::{module::Setting, utils::help::node_version_parse};
use anyhow::{bail, Result};

#[derive(clap::Args)]
pub struct Which {
    /// The version number of Node.js
    version: String,
}

impl super::Command for Which {
    fn run(self) -> Result<()> {
        let version = node_version_parse(&self.version)?;
        let mut path = Setting::global()?
            .get_directory()?
            .join(&version.to_string());
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
