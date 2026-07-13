use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetUri, AssetUuid};

/// Recoverable registry events surfaced to editor diagnostics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetRegistryDiagnostic {
    CorruptPersistenceRebuilt {
        path: PathBuf,
        reason: String,
    },
    DuplicateGuidReminted {
        original: AssetUuid,
        first_path: AssetUri,
        path: AssetUri,
        replacement: AssetUuid,
    },
    UnresolvedDependency {
        owner: AssetUuid,
        path: AssetUri,
    },
}
