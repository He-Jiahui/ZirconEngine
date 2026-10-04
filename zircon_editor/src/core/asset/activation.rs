use serde::{Deserialize, Serialize};
use zircon_runtime::asset::{AssetUri, AssetUuid};

/// The gesture or command that captured an exact asset target for activation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetActivationSource {
    DoubleClick,
    Enter,
    OpenCommand,
    ContextMenu,
    Reference,
}

/// Captured asset identity and revisions; execution must compare them with current authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetActivationIntent {
    pub asset_uuid: AssetUuid,
    pub asset_locator: AssetUri,
    pub catalog_revision: u64,
    pub resource_revision: Option<u64>,
    pub source: AssetActivationSource,
}

/// A host-issued terminal result for one exact activation intent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetActivationReceipt {
    pub intent: AssetActivationIntent,
    pub result: AssetActivationResult,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetActivationResult {
    Opened { view_instance_id: String },
    Reused { view_instance_id: String },
    Unavailable { reason: String },
    Failed { message: String },
}

#[cfg(test)]
#[path = "activation/tests/cases.rs"]
mod tests;
