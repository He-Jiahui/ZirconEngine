use super::*;
mod preflight;
use crate::{
    asset::pack::ZrPackReader,
    plugin::{
        native::{
            verify_native_package_receipts, NativePackageModuleArtifact,
            NativePackageReceiptPolicy, NativePackageReceiptTrust,
        },
        PluginPackageManifest,
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BundlePlan {
    schema_version: u32,
    manifest_path: String,
    manifest_logical_name: String,
    receipts: Vec<String>,
    modules: Vec<NativePackageModuleArtifact>,
}

pub(super) fn prepare(
    request: InstallRequest,
    bytes: Vec<u8>,
    host: &PackageHostPolicy,
) -> Result<PreparedPackage> {
    request.validate()?;
    if request.target != host.target {
        return Err(PackageError::Trust);
    }
    if bytes.len() as u64 != request.artifact_size || digest(&bytes) != request.artifact_digest {
        return Err(PackageError::Trust);
    }
    preflight::admit(&bytes)?;
    let reader = ZrPackReader::from_bytes(bytes).map_err(|_| PackageError::Invalid)?;
    let members = &reader.manifest().assets;
    let mut total = 0u64;
    if members.is_empty() || members.len() > MAX_FILES {
        return Err(PackageError::Capacity);
    }
    if !distinct_members(members.iter().map(|member| member.path.as_str())) {
        return Err(PackageError::Invalid);
    }
    for member in members {
        total = total
            .checked_add(member.size)
            .filter(|sum| *sum <= MAX_PACKAGE_BYTES as u64)
            .ok_or(PackageError::Capacity)?;
    }
    let read_control = |name: &str| -> Result<Vec<u8>> {
        if !reader
            .manifest()
            .asset(name)
            .is_some_and(|item| item.size <= MAX_CONTROL_BYTES as u64)
        {
            return Err(PackageError::Capacity);
        }
        reader.read_asset(name).map_err(|_| PackageError::Invalid)
    };
    let plan: BundlePlan = serde_json::from_slice(&read_control("package-install.json")?)
        .map_err(|_| PackageError::Invalid)?;
    if plan.schema_version != 1
        || plan.receipts.is_empty()
        || plan.receipts.len() > 16
        || plan.modules.is_empty()
        || plan.modules.len() > 3
    {
        return Err(PackageError::Invalid);
    }
    let manifest_bytes = read_control(&plan.manifest_path)?;
    let manifest: PluginPackageManifest =
        toml::from_str(std::str::from_utf8(&manifest_bytes).map_err(|_| PackageError::Invalid)?)
            .map_err(|_| PackageError::Invalid)?;
    if !valid_plugin_id(&manifest.id) || !product_install_role_is_allowed(&manifest) {
        return Err(PackageError::Trust);
    }
    // This transaction admits a complete single-package closure, including all declared DLLs.
    if !manifest.dependencies.is_empty()
        || !manifest.optional_features.is_empty()
        || !manifest.feature_extensions.is_empty()
        || manifest.modules.len() != plan.modules.len()
        || manifest
            .modules
            .iter()
            .any(|module| !module.module_dependencies.is_empty())
    {
        return Err(PackageError::Invalid);
    }
    let receipts = plan
        .receipts
        .iter()
        .map(|path| read_control(path))
        .collect::<Result<Vec<_>>>()?;
    let receipt_refs = receipts.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let trust = NativePackageReceiptTrust::from_registry_json(
        &serde_json::to_vec(&host.trust_registry).map_err(|_| PackageError::Invalid)?,
        host.key_policies.clone(),
        host.trust_valid_until,
        MAX_CONTROL_BYTES,
    )
    .map_err(|_| PackageError::Trust)?;
    let policy = NativePackageReceiptPolicy {
        plugin_id: manifest.id.clone(),
        package_id: request.package_id.clone(),
        package_version: request.version.clone(),
        sdk_api_version: host.sdk_api_version.clone(),
        build_set_id: host.build_set_id.clone(),
        target: host.target.clone(),
        target_triple: host.target_triple.clone(),
        manifest_logical_name: plan.manifest_logical_name.clone(),
        manifest_relative_path: plan.manifest_path.clone(),
        modules: plan.modules.clone(),
        required_capabilities: Vec::new(),
        allowed_capabilities: host.allowed_capabilities.clone(),
        now: Utc::now(),
        max_receipt_age_seconds: host.max_receipt_age_seconds,
        max_receipt_bytes: MAX_CONTROL_BYTES,
        max_receipt_count: 16,
        max_manifest_bytes: MAX_CONTROL_BYTES,
    };
    let proof = verify_native_package_receipts(&receipt_refs, &manifest_bytes, &trust, &policy)
        .map_err(|_| PackageError::Trust)?;
    let mut files = BTreeMap::new();
    files.insert(plan.manifest_path, manifest_bytes);
    for (binding, expected) in plan.modules.iter().zip(proof.artifact_expectations()) {
        let bytes = reader
            .read_asset(&binding.relative_path)
            .map_err(|_| PackageError::Invalid)?;
        if bytes.len() as u64 != expected.library_digest.byte_length
            || digest(&bytes) != expected.library_digest.sha256
        {
            return Err(PackageError::Trust);
        }
        if files.insert(binding.relative_path.clone(), bytes).is_some() {
            return Err(PackageError::Invalid);
        }
        for (binding, expected) in binding.dependencies.iter().zip(&expected.dependencies) {
            let bytes = reader
                .read_asset(&binding.relative_path)
                .map_err(|_| PackageError::Invalid)?;
            if bytes.len() as u64 != expected.digest.byte_length
                || digest(&bytes) != expected.digest.sha256
            {
                return Err(PackageError::Trust);
            }
            if files.insert(binding.relative_path.clone(), bytes).is_some() {
                return Err(PackageError::Invalid);
            }
        }
    }
    for (name, bytes) in plan.receipts.into_iter().zip(receipts) {
        if files.insert(name, bytes).is_some() {
            return Err(PackageError::Invalid);
        }
    }
    if files
        .insert(
            "package-install.json".into(),
            read_control("package-install.json")?,
        )
        .is_some()
        || files.len() != members.len()
    {
        return Err(PackageError::Invalid);
    }
    Ok(PreparedPackage {
        request,
        plugin_id: manifest.id,
        files,
        valid_until: proof.valid_until(),
    })
}

fn product_install_role_is_allowed(manifest: &PluginPackageManifest) -> bool {
    manifest.package_role.is_product_catalog_eligible()
}

#[cfg(test)]
#[path = "tests/verify.rs"]
mod tests;
