use crate::{
    module::Setting,
    utils::{help::node_version_parse, notice::Notice},
};
use anyhow::{bail, Result};
use console::style;
use fs_extra::dir;

#[derive(clap::Args)]
pub struct Uninstall {
    /// The version number of Node.js
    version: String,
}

impl super::Command for Uninstall {
    fn run(self) -> Result<()> {
        let version = node_version_parse(&self.version)?;
        let path = Setting::global()?
            .get_directory()?
            .join(&version.to_string());

        if !path.exists() {
            bail!("Node@v{} has not been installed", &version);
        }

        eprintln!("Removing Node@v{} at: {:?}", &version, &path);
        dir::remove(path)?;
        eprintln!(
            "{}",
            style(format!(
                "Node@v{} has been successfully uninstalled",
                version
            ))
            .green()
        );

        let _ = Notice::from_version().send();

        Ok(())
    }
}
