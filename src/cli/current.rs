use crate::module::Context;
use anyhow::Result;

#[derive(clap::Args)]
pub struct Current {}

impl super::Command for Current {
    fn run(self) -> Result<()> {
        if let Some(version) = Context::global()?.get_version() {
            eprintln!("v{}", version);
        }
        Ok(())
    }
}
