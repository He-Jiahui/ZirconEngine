use std::ffi::OsStr;
use std::fs;
use std::io::{self, ErrorKind, Read};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{
    PluginModuleKind, PluginPackageKind, PluginPackageManifest, PluginPackageRole,
};

use super::{
    ExportLinkedRuntimeCrate, ExportRuntimeCrateRegistrationKind, LibraryEmbedCompileHostTarget,
    LibraryEmbedLinkedRuntimeCrate,
};

const MAX_SOURCE_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;

/// The manifest and crate path independently admitted for one generated linked feature.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportLinkedFeatureSourceReceipt {
    plugin_root: PathBuf,
    provider_package_id: String,
    owner_plugin_id: String,
    feature_id: String,
    crate_name: String,
    crate_path: String,
    target_mode: RuntimeTargetMode,
    package_role: PluginPackageRole,
    manifest_sha256: String,
    crate_manifest_sha256: String,
}

impl ExportLinkedFeatureSourceReceipt {
    pub const fn package_role(&self) -> PluginPackageRole {
        self.package_role
    }

    pub(crate) fn crate_name(&self) -> &str {
        &self.crate_name
    }

    pub(crate) fn generated_provider_call(&self) -> String {
        let mut call = format!(
            "ExportRuntimePluginFeatureRegistrationProvider::new({}::plugin_feature_registration)",
            self.crate_name
        );
        if self.provider_package_id != self.owner_plugin_id {
            call.push_str(&format!(
                ".with_provider_package_id({:?})",
                self.provider_package_id
            ));
        }
        call.push_str(&format!(
            ".with_admitted_source_identity({:?}, {:?}, {:?}, {:?}, zircon_runtime::plugin::PluginPackageRole::{:?})",
            self.feature_id,
            self.owner_plugin_id,
            self.provider_package_id,
            self.crate_name,
            self.package_role
        ));
        call
    }

    pub(crate) fn generated_cargo_path(&self) -> String {
        self.plugin_root
            .join(&self.crate_path)
            .to_string_lossy()
            .replace('\\', "/")
    }

    pub(crate) fn matches_library_embed_link(
        &self,
        linked_crate: &LibraryEmbedLinkedRuntimeCrate,
    ) -> bool {
        linked_crate.registration_kind == LibraryEmbedCompileHostTarget::RuntimeFeature
            && linked_crate.crate_name == self.crate_name
            && linked_crate.path == self.crate_path
            && linked_crate.provider_package_id.as_deref()
                == (self.provider_package_id != self.owner_plugin_id)
                    .then_some(self.provider_package_id.as_str())
    }

    pub(crate) fn verify_current(&self) -> io::Result<()> {
        if !self.package_role.is_product_catalog_eligible() {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "linked feature {} provider package {} with role {:?} is not eligible for product export",
                    self.feature_id, self.provider_package_id, self.package_role
                ),
            ));
        }
        let current = admit_source(
            &self.plugin_root,
            &self.provider_package_id,
            &self.owner_plugin_id,
            &self.feature_id,
            &self.crate_name,
            &self.crate_path,
            self.target_mode,
        )
        .map_err(|error| {
            io::Error::new(
                ErrorKind::PermissionDenied,
                format!("linked feature source changed after planning: {error}"),
            )
        })?;
        if current != *self {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "linked feature source changed after planning: {} from {}",
                    self.feature_id, self.provider_package_id
                ),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub(crate) enum LinkedFeatureSourceAdmissionError {
    #[error("linked feature {feature_id} requires a plugin source root")]
    MissingSourceRoot { feature_id: String },
    #[error("linked feature {feature_id} source admission failed: {detail}")]
    InvalidSource { feature_id: String, detail: String },
}

pub(super) fn admit_linked_feature_sources(
    linked_crates: &mut [ExportLinkedRuntimeCrate],
    plugin_root: Option<&Path>,
    target_mode: RuntimeTargetMode,
) -> (Vec<ExportLinkedFeatureSourceReceipt>, Vec<String>) {
    let mut receipts = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    for linked_crate in linked_crates.iter_mut().filter(|linked_crate| {
        linked_crate.registration_kind == ExportRuntimeCrateRegistrationKind::RuntimeFeature
    }) {
        let feature_id = linked_crate
            .feature_id
            .as_deref()
            .expect("runtime feature link must retain its feature identity");
        let owner_plugin_id = linked_crate
            .owner_plugin_id
            .as_deref()
            .expect("runtime feature link must retain its owner identity");
        let provider_package_id = linked_crate
            .provider_package_id
            .as_deref()
            .unwrap_or(owner_plugin_id);
        let result = plugin_root
            .ok_or_else(|| LinkedFeatureSourceAdmissionError::MissingSourceRoot {
                feature_id: feature_id.to_owned(),
            })
            .and_then(|plugin_root| {
                admit_source(
                    plugin_root,
                    provider_package_id,
                    owner_plugin_id,
                    feature_id,
                    &linked_crate.crate_name,
                    &linked_crate.path,
                    target_mode,
                )
            });
        match result {
            Ok(receipt) => {
                linked_crate.provider_package_role = Some(receipt.package_role());
                linked_crate.admitted_source_path =
                    Some(receipt.plugin_root.join(&receipt.crate_path));
                if !receipt.package_role().is_product_catalog_eligible() {
                    fatal_diagnostics.push(format!(
                        "linked feature {feature_id} provider package {provider_package_id} with role {:?} is not eligible for product export",
                        receipt.package_role()
                    ));
                }
                receipts.push(receipt);
            }
            Err(error) => fatal_diagnostics.push(error.to_string()),
        }
    }
    (receipts, fatal_diagnostics)
}

fn admit_source(
    plugin_root: &Path,
    provider_package_id: &str,
    owner_plugin_id: &str,
    feature_id: &str,
    crate_name: &str,
    crate_path: &str,
    target_mode: RuntimeTargetMode,
) -> Result<ExportLinkedFeatureSourceReceipt, LinkedFeatureSourceAdmissionError> {
    let invalid = |detail: String| LinkedFeatureSourceAdmissionError::InvalidSource {
        feature_id: feature_id.to_owned(),
        detail,
    };
    let plugin_root = std::path::absolute(plugin_root)
        .map_err(|error| invalid(format!("cannot resolve plugin source root: {error}")))?;
    if plugin_root.to_str().is_none() {
        return Err(invalid(format!(
            "{} cannot be encoded in generated Cargo metadata",
            plugin_root.display()
        )));
    }
    require_real_directory(&plugin_root).map_err(&invalid)?;
    require_single_component(provider_package_id).map_err(|detail| {
        invalid(format!(
            "provider package id {provider_package_id:?}: {detail}"
        ))
    })?;

    let package_root = plugin_root.join(provider_package_id);
    require_real_directory(&package_root).map_err(&invalid)?;
    let manifest_path = package_root.join("plugin.toml");
    let manifest_bytes = read_real_manifest(&manifest_path).map_err(&invalid)?;
    let manifest_source = std::str::from_utf8(&manifest_bytes)
        .map_err(|error| invalid(format!("{} is not UTF-8: {error}", manifest_path.display())))?;
    if provider_package_id != owner_plugin_id
        && toml::from_str::<toml::Value>(manifest_source)
            .map_err(|error| invalid(format!("{} is invalid: {error}", manifest_path.display())))?
            .get("package_role")
            .is_none()
    {
        return Err(invalid(format!(
            "{} must declare an explicit package_role for external linked feature admission",
            manifest_path.display()
        )));
    }
    let manifest: PluginPackageManifest = toml::from_str(manifest_source)
        .map_err(|error| invalid(format!("{} is invalid: {error}", manifest_path.display())))?;
    if manifest.id != provider_package_id {
        return Err(invalid(format!(
            "{} declares package id {} instead of {provider_package_id}",
            manifest_path.display(),
            manifest.id
        )));
    }
    if provider_package_id != owner_plugin_id
        && manifest.package_kind != PluginPackageKind::FeatureExtension
    {
        return Err(invalid(format!(
            "{} external linked feature provider must declare package_kind = feature_extension",
            manifest_path.display()
        )));
    }
    if !manifest.supported_targets.is_empty() && !manifest.supported_targets.contains(&target_mode)
    {
        return Err(invalid(format!(
            "{} does not support target {target_mode:?}",
            manifest_path.display()
        )));
    }

    let matches = manifest
        .optional_features
        .iter()
        .chain(&manifest.feature_extensions)
        .filter(|feature| feature.id == feature_id)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(invalid(format!(
            "{} must declare feature {feature_id} exactly once",
            manifest_path.display()
        )));
    }
    let feature = matches[0];
    if feature.owner_plugin_id != owner_plugin_id
        || feature
            .provider_package_id
            .as_deref()
            .unwrap_or(&manifest.id)
            != provider_package_id
    {
        return Err(invalid(format!(
            "{} feature {feature_id} owner/provider does not match {owner_plugin_id}/{provider_package_id}",
            manifest_path.display()
        )));
    }
    let runtime_modules = feature
        .modules
        .iter()
        .filter(|module| {
            module.kind == PluginModuleKind::Runtime
                && module.crate_name == crate_name
                && (module.target_modes.is_empty() || module.target_modes.contains(&target_mode))
        })
        .count();
    if runtime_modules != 1 {
        return Err(invalid(format!(
            "{} feature {feature_id} must declare runtime crate {crate_name} for {target_mode:?} exactly once",
            manifest_path.display()
        )));
    }

    let relative_crate_path = Path::new(crate_path);
    let mut components = relative_crate_path.components();
    if components.next() != Some(Component::Normal(OsStr::new(provider_package_id)))
        || !components.all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(invalid(format!(
            "linked crate path {crate_path:?} must remain inside provider package {provider_package_id}"
        )));
    }
    let mut current = plugin_root.clone();
    for component in relative_crate_path.components() {
        current.push(component.as_os_str());
        require_real_directory(&current).map_err(&invalid)?;
    }
    let crate_manifest_path = current.join("Cargo.toml");
    let crate_manifest_bytes = read_real_manifest(&crate_manifest_path).map_err(&invalid)?;
    let cargo_source = std::str::from_utf8(&crate_manifest_bytes).map_err(|error| {
        invalid(format!(
            "{} is not UTF-8: {error}",
            crate_manifest_path.display()
        ))
    })?;
    let cargo: toml::Value = toml::from_str(cargo_source).map_err(|error| {
        invalid(format!(
            "{} is invalid: {error}",
            crate_manifest_path.display()
        ))
    })?;
    if cargo
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        != Some(crate_name)
    {
        return Err(invalid(format!(
            "{} package.name must equal {crate_name}",
            crate_manifest_path.display()
        )));
    }

    Ok(ExportLinkedFeatureSourceReceipt {
        plugin_root,
        provider_package_id: provider_package_id.to_owned(),
        owner_plugin_id: owner_plugin_id.to_owned(),
        feature_id: feature_id.to_owned(),
        crate_name: crate_name.to_owned(),
        crate_path: crate_path.to_owned(),
        target_mode,
        package_role: manifest.package_role,
        manifest_sha256: format!("{:x}", Sha256::digest(&manifest_bytes)),
        crate_manifest_sha256: format!("{:x}", Sha256::digest(&crate_manifest_bytes)),
    })
}

fn require_single_component(value: &str) -> Result<(), String> {
    let mut components = Path::new(value).components();
    if matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none() {
        Ok(())
    } else {
        Err("must be one normal path component".to_owned())
    }
}

fn require_real_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{} is not a real directory: {error}", path.display()))?;
    if !metadata.is_dir() || is_redirecting_metadata(&metadata) {
        return Err(format!("{} is not a real directory", path.display()));
    }
    Ok(())
}

fn read_real_manifest(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{} is missing or unreadable: {error}", path.display()))?;
    if !metadata.is_file() || is_redirecting_metadata(&metadata) {
        return Err(format!("{} is not a real file", path.display()));
    }
    if metadata.len() > MAX_SOURCE_MANIFEST_BYTES {
        return Err(format!(
            "{} exceeds source manifest size limit",
            path.display()
        ));
    }
    let mut file = fs::File::open(path)
        .map_err(|error| format!("{} is unreadable: {error}", path.display()))?;
    let opened = file
        .metadata()
        .map_err(|error| format!("{} metadata is unreadable: {error}", path.display()))?;
    if !opened.is_file() || !same_file_identity(&metadata, &opened) {
        return Err(format!(
            "{} changed while opening source manifest",
            path.display()
        ));
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(MAX_SOURCE_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{} is unreadable: {error}", path.display()))?;
    if bytes.len() as u64 > MAX_SOURCE_MANIFEST_BYTES {
        return Err(format!(
            "{} exceeds source manifest size limit",
            path.display()
        ));
    }
    let opened_after = file
        .metadata()
        .map_err(|error| format!("{} metadata is unreadable: {error}", path.display()))?;
    let current = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "{} changed while reading source manifest: {error}",
            path.display()
        )
    })?;
    if !current.is_file()
        || is_redirecting_metadata(&current)
        || !same_file_identity(&opened, &opened_after)
        || !same_file_identity(&opened, &current)
        || opened.len() != opened_after.len()
        || opened.modified().ok() != opened_after.modified().ok()
    {
        return Err(format!(
            "{} changed while reading source manifest",
            path.display()
        ));
    }
    // Confirm the path still names the captured contents after the handle read.
    let mut current_file = fs::File::open(path).map_err(|error| {
        format!(
            "{} changed while reading source manifest: {error}",
            path.display()
        )
    })?;
    let current_opened = current_file
        .metadata()
        .map_err(|error| format!("{} metadata is unreadable: {error}", path.display()))?;
    let mut current_bytes = Vec::new();
    (&mut current_file)
        .take(MAX_SOURCE_MANIFEST_BYTES + 1)
        .read_to_end(&mut current_bytes)
        .map_err(|error| format!("{} is unreadable: {error}", path.display()))?;
    let current_after = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "{} changed while reading source manifest: {error}",
            path.display()
        )
    })?;
    if current_bytes != bytes
        || !current_after.is_file()
        || is_redirecting_metadata(&current_after)
        || !same_file_identity(&opened, &current_opened)
        || !same_file_identity(&opened, &current_after)
    {
        return Err(format!(
            "{} changed while reading source manifest",
            path.display()
        ));
    }
    Ok(bytes)
}

fn same_file_identity(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        return left.creation_time() == right.creation_time()
            && left.last_write_time() == right.last_write_time()
            && left.file_size() == right.file_size();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        return left.dev() == right.dev() && left.ino() == right.ino();
    }
    #[cfg(not(any(windows, unix)))]
    {
        left.len() == right.len()
            && left.created().ok() == right.created().ok()
            && left.modified().ok() == right.modified().ok()
    }
}

fn is_redirecting_metadata(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    false
}
