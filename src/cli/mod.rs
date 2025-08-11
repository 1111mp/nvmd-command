use anyhow::Result;

mod current;
mod install;
mod list;
mod uninstall;
mod r#use;
mod which;

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Get the currently used version
    Current(current::Current),

    /// Install the specified version of Node.js
    Install(install::Install),

    /// List the all installed versions of Node.js
    List(list::List),

    /// List the all installed versions of Node.js
    Ls(list::List),

    /// Uninstall the specified version of Node.js
    Uninstall(uninstall::Uninstall),

    /// Use the installed version of Node.js (default is global)
    Use(r#use::Use),

    /// Get the path to the executable to where Node.js was installed
    Which(which::Which),
}

impl Subcommand {
    pub fn run(self) -> Result<()> {
        match self {
            Subcommand::Current(current) => current.run(),
            Subcommand::Install(install) => install.run(),
            Subcommand::List(list) | Subcommand::Ls(list) => list.run(),
            Subcommand::Uninstall(uninstall) => uninstall.run(),
            Subcommand::Use(r#use) => r#use.run(),
            Subcommand::Which(which) => which.run(),
        }
    }
}

/// Nvmd command
pub trait Command: Sized {
    fn run(self) -> Result<()>;
}
