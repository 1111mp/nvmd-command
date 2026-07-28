use crate::{module::NodeVersionResolver, node::Node, utils::notice::Notice};
use anyhow::Result;

#[derive(clap::Args)]
pub struct Install {
    /// The version number of Node.js
    version: String,
}

impl super::Command for Install {
    fn run(self) -> Result<()> {
        let version = NodeVersionResolver::parse(&self.version)?;
        Node::new(version).ensure_fetched()?;

        let _ = Notice::from_version().send();

        Ok(())
    }
}
