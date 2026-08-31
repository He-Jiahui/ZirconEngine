use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::core::resource::io::transaction::{JournalDocument, RecoveryPolicy};

use super::IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_FILE_NAME;

pub(super) struct IblSourceCubemapBundleRecoveryPolicy {
    journal_directory: PathBuf,
    source_root: PathBuf,
    asset_derived_root: PathBuf,
    manifest_root: PathBuf,
}

impl IblSourceCubemapBundleRecoveryPolicy {
    pub(super) fn new(
        journal_directory: PathBuf,
        source_root: PathBuf,
        asset_derived_root: PathBuf,
        manifest_root: PathBuf,
    ) -> Self {
        Self {
            journal_directory,
            source_root,
            asset_derived_root,
            manifest_root,
        }
    }
}

impl RecoveryPolicy for IblSourceCubemapBundleRecoveryPolicy {
    fn validate_document(
        &self,
        journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String> {
        if journal_path.parent() != Some(self.journal_directory.as_path()) {
            return Err(format!(
                "IBL source bundle journal is outside its configured directory: {}",
                journal_path.display()
            ));
        }
        if document.retired_paths().next().is_some() {
            return Err("IBL source bundle transactions cannot retire live files".to_owned());
        }
        validate_ibl_bundle_target(&self.source_root, document.target(), IblBundleTarget::Source)
            .or_else(|source_error| {
                validate_ibl_bundle_target(
                    &self.asset_derived_root,
                    document.target(),
                    IblBundleTarget::AssetDerived,
                )
                .or_else(|asset_derived_error| {
                    validate_ibl_bundle_target(
                        &self.manifest_root,
                        document.target(),
                        IblBundleTarget::Manifest,
                    )
                    .map_err(|manifest_error| {
                        format!(
                            "IBL source bundle target {} is invalid: source ({source_error}); asset-derived ({asset_derived_error}); manifest ({manifest_error})",
                            document.target().display()
                        )
                    })
                })
            })
    }
}

#[derive(Clone, Copy)]
pub(super) enum IblBundleTarget {
    Source,
    AssetDerived,
    Manifest,
}

pub(super) fn validate_ibl_bundle_target(
    root: &Path,
    target: &Path,
    kind: IblBundleTarget,
) -> Result<(), String> {
    validate_ibl_bundle_directory(root)?;
    let relative = target.strip_prefix(root).map_err(|_| {
        format!(
            "target {} does not reside under {}",
            target.display(),
            root.display()
        )
    })?;
    let components = relative.components().collect::<Vec<_>>();
    let [Component::Normal(identity), Component::Normal(file_name)] = components.as_slice() else {
        return Err("target must have exactly an identity directory and artifact file".to_owned());
    };
    let identity = identity
        .to_str()
        .ok_or_else(|| "target identity is not valid Unicode".to_owned())?;
    if identity.len() != blake3::OUT_LEN * 2
        || !identity
            .bytes()
            .all(|value| value.is_ascii_digit() || matches!(value, b'a'..=b'f'))
    {
        return Err("target identity is not a lowercase BLAKE3 digest".to_owned());
    }
    let file_name = file_name
        .to_str()
        .ok_or_else(|| "target filename is not valid Unicode".to_owned())?;
    match kind {
        IblBundleTarget::Source if file_name == "source.zcube" => {}
        IblBundleTarget::AssetDerived if valid_asset_derived_filename(file_name) => {}
        IblBundleTarget::Manifest if file_name == IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_FILE_NAME => {}
        IblBundleTarget::Source => {
            return Err("source target filename is not source.zcube".to_owned());
        }
        IblBundleTarget::AssetDerived => {
            return Err("asset-derived target filename has invalid face/mip layout".to_owned());
        }
        IblBundleTarget::Manifest => {
            return Err("manifest target filename is not bundle.zriblmeta".to_owned());
        }
    }
    validate_ibl_bundle_directory(&root.join(identity))
}

fn valid_asset_derived_filename(file_name: &str) -> bool {
    let Some(layout) = file_name
        .strip_prefix("face_")
        .and_then(|value| value.strip_suffix(".zribl"))
        .and_then(|value| value.split_once("_mips_"))
    else {
        return false;
    };
    let (face_size, mip_count) = layout;
    face_size.len() >= 4
        && mip_count.len() >= 2
        && face_size.parse::<u32>().is_ok_and(|value| value > 0)
        && mip_count.parse::<u32>().is_ok_and(|value| value > 0)
}

fn validate_ibl_bundle_directory(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        // The journal exists before staging creates target parents; recovery
        // must be able to reject or remove an intent in that interval.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("inspect directory {}: {error}", path.display())),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{} is not a real directory", path.display()));
    }
    Ok(())
}
