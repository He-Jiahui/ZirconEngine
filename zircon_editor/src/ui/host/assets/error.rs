use thiserror::Error;
use zircon_runtime::asset::importer::AssetImportError;

use crate::core::asset::EditorAssetIndexError;

#[derive(Debug, Error)]
pub enum EditorAssetSyncError {
    #[error(transparent)]
    Runtime(#[from] AssetImportError),
    #[error(transparent)]
    Index(#[from] EditorAssetIndexError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
