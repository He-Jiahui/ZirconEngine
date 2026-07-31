use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

use crate::asset::{AssetKind, AssetUri, AssetUuid};

/// Metadata required for discovery and dependency queries without loading asset payloads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRegistryEntry {
    uuid: AssetUuid,
    path: AssetUri,
    type_marker: AssetKind,
    tags: BTreeSet<String>,
    dependencies: Vec<AssetUuid>,
    source_digest: String,
}

impl AssetRegistryEntry {
    pub fn new(
        uuid: AssetUuid,
        path: AssetUri,
        type_marker: AssetKind,
        source_digest: impl Into<String>,
    ) -> Self {
        Self {
            uuid,
            path,
            type_marker,
            tags: BTreeSet::new(),
            dependencies: Vec::new(),
            source_digest: source_digest.into(),
        }
    }

    pub fn with_tags(mut self, tags: BTreeSet<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<AssetUuid>) -> Self {
        self.dependencies = unique_dependencies(dependencies);
        self
    }

    pub fn uuid(&self) -> AssetUuid {
        self.uuid
    }

    pub fn path(&self) -> &AssetUri {
        &self.path
    }

    pub fn type_marker(&self) -> AssetKind {
        self.type_marker
    }

    pub fn tags(&self) -> &BTreeSet<String> {
        &self.tags
    }

    pub fn dependencies(&self) -> &[AssetUuid] {
        &self.dependencies
    }

    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }

    pub(super) fn set_dependencies(&mut self, dependencies: Vec<AssetUuid>) {
        self.dependencies = unique_dependencies(dependencies);
    }
}

fn unique_dependencies(dependencies: Vec<AssetUuid>) -> Vec<AssetUuid> {
    let mut unique = Vec::with_capacity(dependencies.len());
    let mut seen = HashSet::with_capacity(dependencies.len());
    for dependency in dependencies {
        if seen.insert(dependency) {
            unique.push(dependency);
        }
    }
    unique
}
