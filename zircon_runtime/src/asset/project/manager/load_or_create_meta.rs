use std::path::Path;

use crate::asset::project::AssetMetaDocument;
use crate::asset::{AssetImportError, AssetKind, AssetUri, AssetUuid};

pub(super) fn load_or_create_meta(
    meta_path: &Path,
    uri: &AssetUri,
    kind: AssetKind,
) -> Result<AssetMetaDocument, AssetImportError> {
    if meta_path.exists() {
        let mut meta = AssetMetaDocument::load(meta_path)?;
        meta.primary_locator = uri.clone();
        meta.kind = kind;
        return Ok(meta);
    }

    Ok(AssetMetaDocument::new(AssetUuid::new(), uri.clone(), kind))
}
