use crate::{node::Node, utils::help::node_version_parse};
use anyhow::Result;

#[derive(clap::Args)]
pub struct Install {
    /// The version number of Node.js
    version: String,
}

impl super::Command for Install {
    fn run(self) -> Result<()> {
        let version = node_version_parse(&self.version)?;
        Node::new(version).ensure_fetched()?;

        Ok(())
    }
}
