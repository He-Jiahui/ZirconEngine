use std::path::Path;

use crate::asset::project::AssetMetaDocument;
use crate::asset::{AssetImportError, AssetImporterDescriptor, AssetKind, AssetUri, AssetUuid};

use super::hash_bytes::hash_bytes;

pub(super) fn load_or_create_meta(
    meta_path: &Path,
    uri: &AssetUri,
    kind: AssetKind,
) -> Result<AssetMetaDocument, AssetImportError> {
    if meta_path.exists() {
        let mut meta = AssetMetaDocument::load(meta_path)?;
        meta.url = uri.clone();
        meta.asset_kind = kind;
        return Ok(meta);
    }

    Ok(mint_meta(uri, kind))
}

fn mint_meta(uri: &AssetUri, kind: AssetKind) -> AssetMetaDocument {
    AssetMetaDocument::new(AssetUuid::new(), uri.clone(), kind)
}

/// The single current owner for minting a new v7 sidecar identity.
/// Migration uses this constructor but stages the returned bytes in its own transaction.
pub(crate) fn mint_meta_for_migration(
    source_bytes: &[u8],
    uri: &AssetUri,
    descriptor: &AssetImporterDescriptor,
) -> Result<Vec<u8>, AssetImportError> {
    let mut meta = mint_meta(uri, descriptor.output_kind);
    meta.importer_id = descriptor.id.clone();
    meta.importer_version = descriptor.importer_version;
    meta.source_digest = hash_bytes(source_bytes);
    toml::to_string_pretty(&meta)
        .map(String::into_bytes)
        .map_err(|error| AssetImportError::Parse(error.to_string()))
}
