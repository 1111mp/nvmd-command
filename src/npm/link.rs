use super::common;
use crate::{
    module::{Context, PackageJson, Packages},
    utils::help::link_package,
};
use anyhow::Result;
use std::{ffi::OsStr, vec};

/// The arguments passed to an `npm link` command
pub struct LinkArgs<'a> {
    /// The common arguments that apply to each tool
    pub common_args: Vec<&'a OsStr>,
    /// The list of tools to link (if any)
    pub tools: Vec<&'a OsStr>,
}

impl LinkArgs<'_> {
    /// Convert these global link arguments into an executor for the command
    pub fn after_executor(&self) -> Result<()> {
        let version = Context::global()?.get_version().unwrap_or_default();
        let packages = self.get_packages()?;

        if !packages.is_empty() {
            let mut pkgs = Packages::new()?;
            pkgs.record_installed(&packages, &version);
            pkgs.save()?;

            for pkg in &packages {
                link_package(pkg)?;
            }
        }

        Ok(())
    }

    /// The link command supports execution from the current package directory without parameters.
    /// It also supports execution using the relative directory of the package as a parameter.
    /// Finally, the package name is passed in directly, similar to the "install" command.
    /// https://docs.npmjs.com/cli/v9/commands/npm-link#description
    fn get_packages(&self) -> Result<Vec<String>> {
        // 'npm link'
        if self.tools.is_empty() {
            return Ok(PackageJson::from_current_dir()?.bin_names());
        }

        // 'npm link package' or 'npm link ../package'
        // We only need to deal with 'npm link ../package'
        let names = self
            .tools
            .iter()
            .filter(common::is_relative_path)
            .map(|&path| path)
            .collect::<Vec<&OsStr>>();

        if names.is_empty() {
            return Ok(vec![]);
        }

        Ok(names
            .into_iter()
            .map(|pkg| PackageJson::from_current_dir_with_pkg(pkg).map(|p| p.bin_names()))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
    }
}
