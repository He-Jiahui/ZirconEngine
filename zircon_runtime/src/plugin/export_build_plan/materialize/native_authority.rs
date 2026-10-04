use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::plugin::native::{NativePluginArtifactAuthority, NativePluginArtifactTarget};
use crate::plugin::native_plugin_loader::native_library_file_name_for_manifest;
use crate::plugin::PluginPackageManifest;

use super::super::{ExportBuildPlan, ExportMaterializeReport};
use super::copy::{validate_native_dynamic_package_file_entries, NativeDynamicPackageFileEntry};
use super::package_lookup::NativePackageInventory;

const MAX_ADMITTED_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;

pub(super) fn write_embedded_native_authority(
    bytes: &[u8],
    output_root: &Path,
    report: &mut ExportMaterializeReport,
) -> std::io::Result<()> {
    let path = output_root.join("src/zircon_native_authority.json");
    std::fs::create_dir_all(path.parent().expect("generated source has a parent"))?;
    std::fs::write(&path, bytes)?;
    report.generated_files.push(path);
    Ok(())
}

pub(super) fn embedded_native_authority_bytes(
    plan: &ExportBuildPlan,
    inventory: &NativePackageInventory,
) -> std::io::Result<Vec<u8>> {
    let mut expectations = Vec::new();
    for plugin_id in &plan.native_dynamic_packages {
        let package_dir = inventory.package_dir(plugin_id).ok_or_else(|| {
            std::io::Error::other(format!(
                "native authority cannot capture selected package {plugin_id}"
            ))
        })?;
        let captured = NativePluginArtifactAuthority::capture_trusted_build_package(
            plugin_id,
            &package_dir.join("plugin.toml"),
            "zircon-product-build",
            NativePluginArtifactTarget::new(plan.profile.target_mode, plan.profile.target_platform),
        )
        .map_err(std::io::Error::other)?;
        validate_captured_expectations(plugin_id, inventory, &captured)?;
        expectations.extend(captured);
    }
    for plugin_id in &plan.native_dynamic_packages {
        let entries = inventory.file_inventory(plugin_id).ok_or_else(|| {
            std::io::Error::other(format!(
                "native authority has no immutable file inventory for selected package {plugin_id}"
            ))
        })?;
        validate_native_dynamic_package_file_entries(&entries.entries)?;
    }
    serde_json::to_vec(&expectations).map_err(std::io::Error::other)
}

fn validate_captured_expectations(
    plugin_id: &str,
    inventory: &NativePackageInventory,
    expectations: &[crate::plugin::native::NativePluginArtifactExpectation],
) -> std::io::Result<()> {
    let entries = inventory.file_inventory(plugin_id).ok_or_else(|| {
        std::io::Error::other(format!(
            "native authority has no immutable file inventory for selected package {plugin_id}"
        ))
    })?;
    let manifest = entries
        .entries
        .iter()
        .find(|entry| entry.relative_path == "plugin.toml")
        .ok_or_else(|| {
            std::io::Error::other(format!(
                "native authority has no immutable manifest snapshot for selected package {plugin_id}"
            ))
        })?;
    let manifest_source = read_inventory_manifest(manifest)?;
    for expectation in expectations {
        if expectation.manifest_digest != manifest.source_digest {
            return Err(std::io::Error::other(format!(
                "native package {plugin_id} manifest changed after product-role admission"
            )));
        }
        let Some(file_name) =
            native_library_file_name_for_manifest(&manifest_source, &expectation.module_kinds)
        else {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("native package {plugin_id} has no library declared for captured modules"),
            ));
        };
        let relative_path = format!("native/{file_name}");
        if !entries.entries.iter().any(|entry| {
            entry.relative_path == relative_path
                && entry.source_digest == expectation.library_digest
        }) {
            return Err(std::io::Error::other(format!(
                "native package {plugin_id} library {relative_path} is missing or changed after product-role admission"
            )));
        }
        for dependency in &expectation.dependencies {
            let relative_path = format!("native/{}", dependency.file_name);
            let Some(entry) = entries
                .entries
                .iter()
                .find(|entry| entry.relative_path == relative_path)
            else {
                return Err(std::io::Error::other(format!(
                    "native package {plugin_id} dependency {} is absent from the immutable inventory",
                    dependency.file_name
                )));
            };
            if entry.source_digest != dependency.digest {
                return Err(std::io::Error::other(format!(
                    "native package {plugin_id} dependency {} changed after product-role admission",
                    dependency.file_name
                )));
            }
        }
    }
    Ok(())
}

fn read_inventory_manifest(
    entry: &NativeDynamicPackageFileEntry,
) -> std::io::Result<PluginPackageManifest> {
    if entry.source_digest.byte_length > MAX_ADMITTED_MANIFEST_BYTES {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "native package manifest exceeds the admission size limit",
        ));
    }
    let mut file = entry.source_file.try_clone()?;
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    file.take(MAX_ADMITTED_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)?;
    let actual_digest = crate::plugin::native::NativePluginArtifactDigest {
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        byte_length: bytes.len() as u64,
    };
    if actual_digest != entry.source_digest {
        return Err(std::io::Error::new(
            ErrorKind::PermissionDenied,
            "native package manifest changed after immutable inventory admission",
        ));
    }
    let source = std::str::from_utf8(&bytes).map_err(std::io::Error::other)?;
    toml::from_str(source).map_err(std::io::Error::other)
}

#[cfg(test)]
#[path = "tests/native_authority.rs"]
mod tests;
