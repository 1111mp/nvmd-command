use anyhow::Result;
use once_cell::sync::Lazy;
use std::{ffi::OsStr, sync::Mutex};

mod common;
mod install;
mod link;
mod uninstall;
mod unlink;

static NEED_REMOVE_PACKAGES: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static SKIP_NEXT: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

/// Aliases that npm supports for the 'install' command
const NPM_INSTALL_ALIASES: [&str; 12] = [
    "i", "in", "ins", "inst", "insta", "instal", "install", "isnt", "isnta", "isntal", "isntall",
    "add",
];
/// Aliases that npm supports for the 'uninstall' command
const NPM_UNINSTALL_ALIASES: [&str; 5] = ["un", "uninstall", "remove", "rm", "r"];
/// Aliases that npm supports for the `update` command
const NPM_UPDATE_ALIASES: [&str; 4] = ["update", "udpate", "upgrade", "up"];
/// Aliases that npm supports for the 'link' command
const NPM_LINK_ALIASES: [&str; 2] = ["link", "ln"];

pub enum CommandArg<'a> {
    Global(GlobalCommand<'a>),
    Intercepted(InterceptedCommand<'a>),
    Standard,
}

impl<'a> CommandArg<'a> {
    /// Parse the given set of arguments to see if they correspond to an intercepted npm command
    pub fn from_npm<S>(args: &'a [S]) -> Self
    where
        S: AsRef<OsStr>,
    {
        let mut positionals = args.iter().filter(is_positional).map(AsRef::as_ref);

        match positionals.next() {
            Some(cmd) if NPM_INSTALL_ALIASES.iter().any(|a| a == &cmd) => {
                if has_global(args) {
                    let tools: Vec<_> = positionals.collect();

                    if tools.is_empty() {
                        CommandArg::Standard
                    } else {
                        let mut common_args = vec![cmd];
                        common_args.extend(args.iter().filter(is_flag).map(AsRef::as_ref));

                        CommandArg::Global(GlobalCommand::Install(install::InstallArgs {
                            common_args,
                            tools,
                        }))
                    }
                } else {
                    CommandArg::Standard
                }
            }
            Some(cmd) if NPM_UNINSTALL_ALIASES.iter().any(|a| a == &cmd) => {
                if has_global(args) {
                    let tools: Vec<_> = positionals.collect();
                    if tools.is_empty() {
                        CommandArg::Standard
                    } else {
                        let mut common_args = vec![cmd];
                        common_args.extend(args.iter().filter(is_flag).map(AsRef::as_ref));

                        CommandArg::Global(GlobalCommand::Uninstall(uninstall::UninstallArgs {
                            common_args,
                            tools,
                        }))
                    }
                } else {
                    CommandArg::Standard
                }
            }
            Some(cmd) if cmd == "unlink" => {
                if has_global(args) {
                    let mut common_args = vec![cmd];
                    common_args.extend(args.iter().filter(is_flag).map(AsRef::as_ref));
                    let tools: Vec<_> = positionals.collect();

                    CommandArg::Intercepted(InterceptedCommand::Unlink(unlink::UnlinkArgs {
                        common_args,
                        tools,
                    }))
                } else {
                    CommandArg::Standard
                }
            }
            Some(cmd) if NPM_LINK_ALIASES.iter().any(|a| a == &cmd) => {
                // Much like install, the common args for a link are the command combined with any flags
                let mut common_args = vec![cmd];
                common_args.extend(args.iter().filter(is_flag).map(AsRef::as_ref));
                let tools: Vec<_> = positionals.collect();

                CommandArg::Intercepted(InterceptedCommand::Link(link::LinkArgs {
                    common_args,
                    tools,
                }))
            }
            Some(cmd) if NPM_UPDATE_ALIASES.iter().any(|a| a == &cmd) => {
                if has_global(args) {
                    // Once again, the common args are the command combined with any flags
                    let mut common_args = vec![cmd];
                    common_args.extend(args.iter().filter(is_flag).map(AsRef::as_ref));
                    let tools: Vec<_> = positionals.collect();

                    CommandArg::Global(GlobalCommand::Upgrade(install::InstallArgs {
                        common_args,
                        tools,
                    }))
                } else {
                    CommandArg::Standard
                }
            }
            _ => CommandArg::Standard,
        }
    }
}

pub enum GlobalCommand<'a> {
    Install(install::InstallArgs<'a>),
    Uninstall(uninstall::UninstallArgs<'a>),
    Upgrade(install::InstallArgs<'a>),
}

impl GlobalCommand<'_> {
    pub fn before_executor(&self) -> Result<()> {
        match self {
            GlobalCommand::Install(_cmd) => Ok(()),
            GlobalCommand::Uninstall(cmd) => cmd.before_executor(),
            GlobalCommand::Upgrade(_cmd) => Ok(()),
        }
    }

    pub fn after_executor(&self) -> Result<()> {
        match self {
            GlobalCommand::Install(cmd) => cmd.after_executor(),
            GlobalCommand::Uninstall(cmd) => cmd.after_executor(),
            GlobalCommand::Upgrade(cmd) => cmd.after_executor(),
        }
    }
}

/// An intercepted local command
pub enum InterceptedCommand<'a> {
    Link(link::LinkArgs<'a>),
    Unlink(unlink::UnlinkArgs<'a>),
}

fn has_global<A>(args: &[A]) -> bool
where
    A: AsRef<OsStr>,
{
    args.iter()
        .fold(false, |global, arg| match arg.as_ref().to_str() {
            Some("-g") | Some("--global") => true,
            _ => global,
        })
}

fn is_flag<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    match arg.as_ref().to_str() {
        Some(a) => a.starts_with('-'),
        None => false,
    }
}

/// https://docs.npmjs.com/cli/v7/commands/npm-install#workspace
/// https://docs.npmjs.com/cli/v7/commands/npm-link#workspace
/// We should filter out the arguments passed via '--workspace <name>'
/// It is a feature introduced in 'npm v7'
fn is_positional<A>(arg: &A) -> bool
where
    A: AsRef<OsStr>,
{
    let mut skip_next = SKIP_NEXT.lock().unwrap();

    if *skip_next {
        *skip_next = false;
        return false;
    }

    if let Some(arg_str) = arg.as_ref().to_str() {
        if arg_str == "--workspace" {
            *skip_next = true;
            return false;
        }
    }

    !is_flag(arg)
}
