use crate::{
    module::{Context, PackageJson, Packages},
    utils::help::unlink_package,
};
use anyhow::Result;
use std::ffi::OsStr;

/// The list of tools passed to an uninstall command
pub struct UninstallArgs<'a> {
    /// Common arguments that apply to each tool (e.g. flags)
    pub common_args: Vec<&'a OsStr>,
    /// The individual tool arguments
    pub tools: Vec<&'a OsStr>,
}

impl UninstallArgs<'_> {
    /// Convert the tools into an executor for the uninstall command
    pub fn before_executor(&self) -> Result<()> {
        let npm_prefix = super::common::get_npm_prefix()?;
        let packages = self
            .tools
            .iter()
            .flat_map(|tool| PackageJson::new(&npm_prefix, tool).bin_names())
            .collect::<Vec<String>>();

        if !packages.is_empty() {
            super::NEED_REMOVE_PACKAGES.lock().unwrap().extend(packages);
        }

        Ok(())
    }

    /// Convert the tools into an executor for the uninstall command
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
}
