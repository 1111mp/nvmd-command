use super::common;
use crate::{
    module::{Context, PackageJson, Packages},
    utils::help::unlink_package,
};
use anyhow::Result;
use std::ffi::OsStr;

/// The arguments passed to an `npm unlink` command
pub struct UnlinkArgs<'a> {
    /// Common arguments that apply to each tool (e.g. flags)
    pub common_args: Vec<&'a OsStr>,
    /// The individual tool arguments
    pub tools: Vec<&'a OsStr>,
}

impl UnlinkArgs<'_> {
    pub fn before_executor(&self) -> Result<()> {
        let packages = self.get_packages()?;
        if !packages.is_empty() {
            super::NEED_REMOVE_PACKAGES.lock().unwrap().extend(packages);
        }

        Ok(())
    }

    pub fn after_executor(&self) -> Result<()> {
        let version = Context::global()?.get_version().unwrap_or_default();
        let packages = super::NEED_REMOVE_PACKAGES.lock().unwrap();
        let mut pkgs = Packages::new()?;
        let need_remove_packages = pkgs.record_uninstalled(&packages, &version);
        pkgs.save()?;

        for package in &need_remove_packages {
            unlink_package(package)?;
        }

        Ok(())
    }

    fn get_packages(&self) -> Result<Vec<String>> {
        if self.tools.is_empty() {
            return Ok(PackageJson::from_current_dir()?.bin_names());
        }

        let npm_prefix = common::get_npm_prefix()?;
        Ok(self
            .tools
            .iter()
            .flat_map(|tool| PackageJson::new(&npm_prefix, tool).bin_names())
            .collect())
    }
}
