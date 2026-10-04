use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::asset::{AssetImportContext, AssetImportError};

use super::super::{
    validate_gltf_texture_import_support, validate_required_gltf_material_extension_support,
};
use super::auxiliary_source::AuxiliarySourceResolver;
use super::gltf_meshopt::decode_meshopt_views;

mod budget;
mod buffers;
mod images;
mod sources;

#[cfg(test)]
#[path = "gltf_decode/tests/admission_tests.rs"]
mod admission_tests;

use budget::DecodedBudget;
use buffers::load_buffers;
use images::decode_images;
use sources::ExternalSources;

pub(crate) const MAX_GLTF_AUXILIARY_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const RUNTIME_SUPPORTED_REQUIRED_EXTENSIONS: &[&str] = &[
    "EXT_meshopt_compression",
    "EXT_texture_webp",
    "KHR_mesh_quantization",
    "KHR_materials_anisotropy",
    "KHR_materials_clearcoat",
    "KHR_materials_emissive_strength",
    "KHR_materials_ior",
    "KHR_materials_transmission",
    "KHR_materials_unlit",
    "KHR_materials_volume",
    "KHR_texture_transform",
];

#[derive(Debug)]
pub struct DecodedGltf {
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

/// Decode primary and companion bytes under the caller's required-extension policy.
pub fn decode_gltf_source_with_required_extensions(
    context: &AssetImportContext,
    supported_required_extensions: &[&str],
) -> Result<DecodedGltf, AssetImportError> {
    decode_gltf_source_with_policy(
        context,
        MAX_GLTF_AUXILIARY_BYTES,
        supported_required_extensions,
    )
}

pub(crate) fn decode_gltf_source(
    context: &AssetImportContext,
) -> Result<DecodedGltf, AssetImportError> {
    decode_gltf_source_with_buffer_limit(context, MAX_GLTF_AUXILIARY_BYTES)
}

fn decode_gltf_source_with_buffer_limit(
    context: &AssetImportContext,
    limit: u64,
) -> Result<DecodedGltf, AssetImportError> {
    decode_gltf_source_with_policy(context, limit, RUNTIME_SUPPORTED_REQUIRED_EXTENSIONS)
}

fn decode_gltf_source_with_policy(
    context: &AssetImportContext,
    limit: u64,
    supported_required_extensions: &[&str],
) -> Result<DecodedGltf, AssetImportError> {
    if context.source_bytes.len() as u64 > MAX_GLTF_AUXILIARY_BYTES {
        return Err(gltf_parse_error(
            "gltf source document exceeds its byte limit",
        ));
    }
    let gltf = gltf::Gltf::from_slice_without_validation(&context.source_bytes)
        .map_err(|error| gltf_parse_error(format!("parse gltf: {error}")))?;
    let blob = gltf.blob;
    let mut json = gltf.document.into_json();
    let required_extensions = json.extensions_required.clone();
    validate_required_extensions(&required_extensions, supported_required_extensions)?;
    json.extensions_required
        .retain(|extension| !supported_required_extensions.contains(&extension.as_str()));
    let document = gltf::Document::from_json(json)
        .map_err(|error| gltf_parse_error(format!("validate gltf: {error}")))?;
    validate_required_gltf_material_extension_support(&document, &required_extensions)?;
    validate_gltf_texture_import_support(&document)?;
    let mut sources = ExternalSources::new(context);
    let mut budget = DecodedBudget::new(limit);
    let mut buffers = load_buffers(&document, blob, &mut sources, &mut budget)?;
    decode_meshopt_views(&document, &mut buffers)?;
    let images = decode_images(&document, &buffers, &mut sources, &mut budget)?;
    Ok(DecodedGltf {
        document,
        buffers,
        images,
    })
}

fn validate_required_extensions(
    required: &[String],
    supported_required_extensions: &[&str],
) -> Result<(), AssetImportError> {
    if let Some(extension) = required
        .iter()
        .find(|extension| !supported_required_extensions.contains(&extension.as_str()))
    {
        return Err(gltf_parse_error(format!(
            "gltf requires unsupported extension `{extension}`"
        )));
    }
    Ok(())
}

fn gltf_buffer_source_name(source: gltf::buffer::Source<'_>) -> Cow<'_, str> {
    match source {
        gltf::buffer::Source::Bin => Cow::Borrowed("the GLB binary chunk"),
        gltf::buffer::Source::Uri(uri)
            if uri
                .get(..5)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:")) =>
        {
            Cow::Borrowed("an embedded data URI")
        }
        gltf::buffer::Source::Uri(uri) => Cow::Owned(format!("`{uri}`")),
    }
}

fn is_data_uri(uri: &str) -> bool {
    uri.get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
}

pub(super) fn gltf_parse_error(message: impl Into<String>) -> AssetImportError {
    AssetImportError::Parse(message.into())
}

pub(crate) fn snapshot_external_gltf_sources(
    asset_root: &Path,
    source_path: &Path,
    _source_uri: &crate::asset::AssetUri,
    source_bytes: &[u8],
    existing: &BTreeMap<PathBuf, Vec<u8>>,
    limit: u64,
) -> Result<(BTreeMap<PathBuf, Vec<u8>>, u64), AssetImportError> {
    let gltf = gltf::Gltf::from_slice_without_validation(source_bytes)
        .map_err(|error| gltf_parse_error(format!("parse gltf for source snapshot: {error}")))?;
    let base_dir = source_path.parent().unwrap_or_else(|| Path::new(""));
    let resolver = AuxiliarySourceResolver::for_asset_root(asset_root, base_dir)?;
    let uris = gltf
        .document
        .buffers()
        .filter_map(|buffer| match buffer.source() {
            gltf::buffer::Source::Uri(uri) if !is_data_uri(uri) => Some(uri),
            _ => None,
        })
        .chain(
            gltf.document
                .images()
                .filter_map(|image| match image.source() {
                    gltf::image::Source::Uri { uri, .. } if !is_data_uri(uri) => Some(uri),
                    _ => None,
                }),
        );
    let mut references = BTreeMap::new();
    let existing_keys = existing
        .keys()
        .map(crate::asset::project::ProjectPaths::lexical_identity)
        .collect::<Result<std::collections::BTreeSet<_>, _>>()
        .map_err(|error| gltf_parse_error(format!("gltf existing source identity: {error}")))?;
    for uri in uris {
        let lexical = resolver.resolve_gltf_uri_lexical(uri)?;
        let key = crate::asset::project::ProjectPaths::lexical_identity(&lexical)
            .map_err(|error| gltf_parse_error(format!("gltf source identity: {error}")))?;
        if existing_keys.contains(&key) || references.contains_key(&key) {
            continue;
        }
        if existing_keys.len().saturating_add(references.len())
            >= AuxiliarySourceResolver::MAX_SNAPSHOT_FILES
        {
            return Err(gltf_parse_error(format!(
                "gltf source snapshot exceeds the {}-file cumulative limit",
                AuxiliarySourceResolver::MAX_SNAPSHOT_FILES
            )));
        }
        references.insert(key, (lexical, uri));
    }
    let mut remaining = limit.min(MAX_GLTF_AUXILIARY_BYTES);
    let mut snapshots = BTreeMap::new();
    let mut latest_mtime_unix_ms = 0;
    for (_, (lexical, uri)) in references {
        let (_, bytes, mtime_unix_ms) = resolver.read_gltf_uri_snapshot(uri, remaining)?;
        remaining = remaining.checked_sub(bytes.len() as u64).ok_or_else(|| {
            gltf_parse_error(format!(
                "gltf auxiliary sources exceed the {}-byte cumulative limit",
                MAX_GLTF_AUXILIARY_BYTES
            ))
        })?;
        latest_mtime_unix_ms = latest_mtime_unix_ms.max(mtime_unix_ms);
        snapshots.insert(lexical, bytes);
    }
    Ok((snapshots, latest_mtime_unix_ms))
}

#[cfg(test)]
#[path = "tests/gltf_decode_plugins07_deferred_buffer_diagnostic_tests.rs"]
mod plugins07_deferred_buffer_diagnostic_tests;
