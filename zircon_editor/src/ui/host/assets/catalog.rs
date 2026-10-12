use std::path::PathBuf;

use zircon_runtime::asset::project::{AssetMetaDocument, PreviewState};
use zircon_runtime::asset::{AssetId, AssetKind, AssetReference, AssetUri, AssetUuid};

#[derive(Clone, Debug, PartialEq)]
pub struct AssetCatalogRecord {
    pub asset_uuid: AssetUuid,
    pub asset_id: AssetId,
    pub locator: AssetUri,
    pub kind: AssetKind,
    pub display_name: String,
    pub file_name: String,
    pub extension: String,
    pub meta_path: PathBuf,
    pub meta: AssetMetaDocument,
    pub source_mtime_unix_ms: u64,
    pub source_hash: String,
    pub preview_state: PreviewState,
    pub preview_artifact_path: PathBuf,
    pub dirty: bool,
    pub diagnostics: Vec<String>,
    pub direct_references: Vec<AssetReference>,
}
