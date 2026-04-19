use serde::{Deserialize, Serialize};

use crate::core::resource::{ResourceDiagnostic, ResourceId, ResourceKind, ResourceLocator, ResourceState};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRecord {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub primary_locator: ResourceLocator,
    pub artifact_locator: Option<ResourceLocator>,
    pub revision: u64,
    pub state: ResourceState,
    pub dependency_ids: Vec<ResourceId>,
    pub diagnostics: Vec<ResourceDiagnostic>,
    pub source_hash: String,
    pub importer_version: u32,
    pub config_hash: String,
}

impl ResourceRecord {
    pub fn new(id: ResourceId, kind: ResourceKind, primary_locator: ResourceLocator) -> Self {
        Self {
            id,
            kind,
            primary_locator,
            artifact_locator: None,
            revision: 0,
            state: ResourceState::Pending,
            dependency_ids: Vec::new(),
            diagnostics: Vec::new(),
            source_hash: String::new(),
            importer_version: 0,
            config_hash: String::new(),
        }
    }

    pub fn id(&self) -> ResourceId {
        self.id
    }

    pub fn primary_locator(&self) -> &ResourceLocator {
        &self.primary_locator
    }

    pub fn artifact_locator(&self) -> Option<&ResourceLocator> {
        self.artifact_locator.as_ref()
    }

    pub fn with_artifact_locator(mut self, artifact_locator: ResourceLocator) -> Self {
        self.artifact_locator = Some(artifact_locator);
        self
    }

    pub fn with_source_hash(mut self, source_hash: impl Into<String>) -> Self {
        self.source_hash = source_hash.into();
        self
    }

    pub fn with_importer_version(mut self, importer_version: u32) -> Self {
        self.importer_version = importer_version;
        self
    }

    pub fn with_config_hash(mut self, config_hash: impl Into<String>) -> Self {
        self.config_hash = config_hash.into();
        self
    }
}
