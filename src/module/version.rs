use anyhow::{Context as _, Result, anyhow};
use std::fs;

use crate::module::Setting;

#[derive(Debug, PartialEq, Eq)]
enum NodeVersionRequest {
    Exact(semver::Version),
    Major(u64),
    MajorMinor(u64, u64),
}

pub struct NodeVersionResolver;

impl NodeVersionResolver {
    /// Parse exact node version.
    ///
    /// Examples:
    /// v14.21.3 -> 14.21.3
    /// 14.21.3  -> 14.21.3
    pub fn parse(input: &str) -> Result<semver::Version> {
        let version = Self::normalize(input);
        semver::Version::parse(version).with_context(|| {
            anyhow!(
                "Failed to parse Node version {} \nPlease ensure the correct version is specified.",
                input
            )
        })
    }

    /// Resolve user input to latest installed node version.
    ///
    /// Examples:
    /// 14       -> 14.21.3
    /// 14.18    -> 14.18.3
    /// 14.18.3  -> 14.18.3
    pub fn resolve(input: &str) -> Result<String> {
        let request = Self::parse_request(input)?;
        let versions_dir = Setting::global()?.get_directory()?;
        let versions = fs::read_dir(&versions_dir)
            .with_context(|| {
                format!(
                    "failed to read the directory \"{}\"",
                    versions_dir.display()
                )
            })?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let file_type = entry.file_type().ok()?;
                if !file_type.is_dir() {
                    return None;
                }

                let version = entry.file_name().into_string().ok()?;
                Self::parse(&version).ok()
            })
            .collect::<Vec<_>>();

        Self::latest_matching(&request, versions)
            .map(|version| version.to_string())
            .ok_or_else(|| anyhow!("Node@v{} has not been installed", input))
    }

    fn parse_request(input: &str) -> Result<NodeVersionRequest> {
        let version = Self::normalize(input);
        let parts = version.split('.').collect::<Vec<_>>();

        match parts.as_slice() {
          [major] if !major.is_empty() => major
              .parse::<u64>()
              .map(NodeVersionRequest::Major)
              .with_context(|| {
                  anyhow!(
                      "Failed to parse Node version {} \nPlease ensure the correct version is specified.",
                      input
                  )
              }),
          [major, minor] if !major.is_empty() && !minor.is_empty() => {
              let major = major.parse::<u64>().with_context(|| {
                  anyhow!(
                      "Failed to parse Node version {} \nPlease ensure the correct version is specified.",
                      input
                  )
              })?;
              let minor = minor.parse::<u64>().with_context(|| {
                  anyhow!(
                      "Failed to parse Node version {} \nPlease ensure the correct version is specified.",
                      input
                  )
              })?;
              Ok(NodeVersionRequest::MajorMinor(major, minor))
          }
          _ => Self::parse(version).map(NodeVersionRequest::Exact),
      }
    }

    fn latest_matching(
        request: &NodeVersionRequest,
        mut versions: Vec<semver::Version>,
    ) -> Option<semver::Version> {
        versions.retain(|version| match request {
            NodeVersionRequest::Major(major) => version.major == *major,
            NodeVersionRequest::MajorMinor(major, minor) => {
                version.major == *major && version.minor == *minor
            }
            NodeVersionRequest::Exact(exact) => version == exact,
        });
        versions.sort();
        versions.pop()
    }

    fn normalize(input: &str) -> &str {
        let input = input.trim();
        input.strip_prefix('v').unwrap_or(input)
    }
}

#[cfg(test)]
mod tests {
    use super::{NodeVersionRequest, NodeVersionResolver};

    #[test]
    fn parse_partial_and_exact_version_requests() {
        assert_eq!(
            NodeVersionResolver::parse_request("14").unwrap(),
            NodeVersionRequest::Major(14)
        );
        assert_eq!(
            NodeVersionResolver::parse_request("14.21").unwrap(),
            NodeVersionRequest::MajorMinor(14, 21)
        );
        assert_eq!(
            NodeVersionResolver::parse_request("v14.21.3").unwrap(),
            NodeVersionRequest::Exact(semver::Version::parse("14.21.3").unwrap())
        );
    }

    #[test]
    fn choose_latest_installed_matching_semver_version_request() {
        let versions = ["14.18.3", "14.21.3", "16.17.0", "18.19.1"]
            .into_iter()
            .map(|version| semver::Version::parse(version).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(
            super::NodeVersionResolver::latest_matching(
                &NodeVersionRequest::Major(14),
                versions.clone()
            )
            .unwrap()
            .to_string(),
            "14.21.3"
        );
        assert_eq!(
            super::NodeVersionResolver::latest_matching(
                &NodeVersionRequest::MajorMinor(14, 18),
                versions
            )
            .unwrap()
            .to_string(),
            "14.18.3"
        );
    }
}
