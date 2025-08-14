use super::common;
use crate::{
    module::{Context, PackageJson, Packages},
    utils::help::link_package,
};
use anyhow::Result;
use regex::Regex;
use std::ffi::OsStr;

/// The arguments passed to a global install command
pub struct InstallArgs<'a> {
    /// Common arguments that apply to each tool (e.g. flags)
    pub common_args: Vec<&'a OsStr>,
    /// The individual tool arguments
    pub tools: Vec<&'a OsStr>,
}

impl InstallArgs<'_> {
    /// Convert these global install arguments into an executor for the command
    pub fn after_executor(&self) -> Result<()> {
        let version = Context::global()?.get_version().unwrap_or_default();
        let reg = Regex::new("@[0-9]|@latest|@\"|@npm:")?;
        let npm_prefix = common::get_npm_prefix()?;

        let mut pkg_names = vec![];
        self.tools
            .iter()
            .map(|tool| {
                let pkg = tool.to_string_lossy().to_string();
                match reg.find(&pkg) {
                    Some(mat) => pkg[0..(mat.start())].to_string(),
                    None => pkg,
                }
            })
            .for_each(|pkg| {
                let names = PackageJson::new(&npm_prefix, &pkg).bin_names();
                if !names.is_empty() {
                    pkg_names.extend(names.iter().cloned());
                }
            });

        let mut packages = Packages::new()?;
        packages.record_installed(&pkg_names, &version);
        packages.save()?;

        for name in &pkg_names {
            link_package(name)?;
        }

        Ok(())
    }
}
