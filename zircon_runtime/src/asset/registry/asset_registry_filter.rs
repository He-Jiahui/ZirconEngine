use std::collections::BTreeSet;

use crate::asset::AssetKind;

/// Offline registry filter mirroring class, tag, path-prefix, and package constraints.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetRegistryFilter {
    pub(super) type_marker: Option<AssetKind>,
    pub(super) required_tags: BTreeSet<String>,
    pub(super) path_prefix: Option<String>,
    pub(super) package_id: Option<String>,
}

impl AssetRegistryFilter {
    pub fn with_type_marker(mut self, type_marker: AssetKind) -> Self {
        self.type_marker = Some(type_marker);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.required_tags.insert(tag.into());
        self
    }

    pub fn with_path_prefix(mut self, path_prefix: impl Into<String>) -> Self {
        self.path_prefix = Some(path_prefix.into());
        self
    }

    pub fn with_package(mut self, package_id: impl Into<String>) -> Self {
        self.package_id = Some(package_id.into());
        self
    }
}
