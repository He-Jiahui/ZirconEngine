use std::io;
use std::path::Path;

use crate::asset::project::AssetMetaDocument;
use crate::asset::{AssetImportError, AssetImporterDescriptor, AssetKind, AssetUri, AssetUuid};

use super::hash_bytes::hash_bytes;

pub(super) fn load_or_create_meta(
    meta_path: &Path,
    uri: &AssetUri,
    kind: AssetKind,
) -> Result<(AssetMetaDocument, Option<AssetMetaDocument>), AssetImportError> {
    match AssetMetaDocument::load(meta_path) {
        Ok(mut meta) => {
            let original = meta.clone();
            refresh_loaded_meta_identity(&mut meta, uri, kind);
            Ok((meta, Some(original)))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok((mint_meta(uri, kind), None)),
        Err(error) => Err(error.into()),
    }
}

/// The caller holds the shared meta authority until its file transaction finishes.
pub(super) fn verify_meta_precondition(
    path: &Path,
    expected: Option<&AssetMetaDocument>,
) -> Result<(), AssetImportError> {
    let current = match AssetMetaDocument::load(path) {
        Ok(document) => Some(document),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    if current.as_ref() != expected {
        return Err(AssetImportError::Parse(format!(
            "project metadata changed while targeted generation was prepared: {}",
            path.display()
        )));
    }
    Ok(())
}

fn refresh_loaded_meta_identity(meta: &mut AssetMetaDocument, uri: &AssetUri, kind: AssetKind) {
    if &meta.url != uri {
        meta.url = uri.clone();
    }
    meta.asset_kind = kind;
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

#[cfg(test)]
#[path = "load_or_create_meta/tests/matching_identity_tests.rs"]
mod matching_identity_tests;
