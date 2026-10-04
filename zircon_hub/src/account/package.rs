mod process;
mod project_lock;
pub(super) use project_lock::capture_project_lock;
#[cfg(test)]
#[path = "package/tests/cases.rs"]
mod tests;
#[cfg(windows)]
mod windows;

use super::{
    operations::{
        validate_id, validate_revision, OperationIdentity, OperationPayload, OperationStatus,
    },
    AccountBroker, AccountError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub(super) const MAX_PACKAGE_BYTES: usize = 16 * 1024 * 1024;
const LEGACY_PACKAGE_INSTALL_SCHEMA_V1: u8 = 1;
const PACKAGE_INSTALL_SCHEMA_V2: u8 = 2;
const POLICY_INDEX_SCHEMA_VERSION: u32 = 1;
const POLICY_INDEX_KIND: &str = "zircon_native_plugin_policy_index";
const POLICY_INDEX_ENV: &str = "ZIRCON_NATIVE_PLUGIN_POLICY_INDEX";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PackageRuntimeMode {
    EditorHost,
    ClientRuntime,
}

impl PackageRuntimeMode {
    fn wire_name(self) -> &'static str {
        match self {
            Self::EditorHost => "editor_host",
            Self::ClientRuntime => "client_runtime",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "u8")]
pub(crate) struct PackageActionSchemaV2;

impl TryFrom<u8> for PackageActionSchemaV2 {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == PACKAGE_INSTALL_SCHEMA_V2 {
            Ok(Self)
        } else {
            Err("unsupported package action schema")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PackageTargetPlatform {
    Windows,
    Linux,
    Macos,
}

impl PackageTargetPlatform {
    fn current_host() -> Result<Self, AccountError> {
        match std::env::consts::OS {
            "windows" => Ok(Self::Windows),
            "linux" => Ok(Self::Linux),
            "macos" => Ok(Self::Macos),
            _ => Err(AccountError::PackageUnavailable),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageInstallTarget {
    runtime_mode: PackageRuntimeMode,
    platform: PackageTargetPlatform,
}

impl PackageInstallTarget {
    fn for_runtime_mode(runtime_mode: PackageRuntimeMode) -> Result<Self, AccountError> {
        Ok(Self {
            runtime_mode,
            platform: PackageTargetPlatform::current_host()?,
        })
    }

    fn validate_for_current_host(&self) -> Result<(), AccountError> {
        if self.platform != PackageTargetPlatform::current_host()? {
            return Err(AccountError::PackageUnavailable);
        }
        Ok(())
    }
}

fn validate_target_echo(
    response: &Value,
    expected: PackageInstallTarget,
) -> Result<(), AccountError> {
    let echoed: PackageInstallTarget = serde_json::from_value(
        response
            .get("target")
            .cloned()
            .ok_or(AccountError::PackageUnavailable)?,
    )
    .map_err(|_| AccountError::PackageUnavailable)?;
    if echoed != expected {
        return Err(AccountError::PackageUnavailable);
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativePluginPolicyIndex {
    schema_version: u32,
    kind: String,
    entries: Vec<NativePluginPolicyIndexEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativePluginPolicyIndexEntry {
    target: PackageInstallTarget,
    policy_path: PathBuf,
    policy_sha256: String,
}

fn selected_policy_index_entry(
    index: &NativePluginPolicyIndex,
    target: PackageInstallTarget,
) -> Result<&NativePluginPolicyIndexEntry, AccountError> {
    if index.schema_version != POLICY_INDEX_SCHEMA_VERSION
        || index.kind != POLICY_INDEX_KIND
        || index.entries.is_empty()
        || index.entries.len() > 24
    {
        return Err(AccountError::PackageTrust);
    }
    let mut targets = std::collections::HashSet::new();
    let mut policy_paths = std::collections::HashSet::new();
    let mut selected = None;
    for entry in &index.entries {
        if !entry.policy_path.is_absolute()
            || entry.policy_path.components().any(|part| {
                matches!(
                    part,
                    std::path::Component::CurDir | std::path::Component::ParentDir
                )
            })
            || !is_digest(&entry.policy_sha256)
            || !targets.insert((entry.target.runtime_mode, entry.target.platform))
            || !policy_paths.insert(entry.policy_path.to_string_lossy().to_ascii_lowercase())
        {
            return Err(AccountError::PackageTrust);
        }
        if entry.target == target {
            selected = Some(entry);
        }
    }
    selected.ok_or(AccountError::PackageTargetUnconfigured)
}

fn selected_policy_root(
    entry: &NativePluginPolicyIndexEntry,
    target: PackageInstallTarget,
    bytes: &[u8],
) -> Result<PathBuf, AccountError> {
    if digest(bytes) != entry.policy_sha256 {
        return Err(AccountError::PackageTrust);
    }
    let policy: Value = serde_json::from_slice(bytes).map_err(|_| AccountError::PackageTrust)?;
    let policy_target: PackageInstallTarget = serde_json::from_value(
        policy
            .get("target")
            .cloned()
            .ok_or(AccountError::PackageTrust)?,
    )
    .map_err(|_| AccountError::PackageTrust)?;
    if policy_target != target {
        return Err(AccountError::PackageTrust);
    }
    let root = PathBuf::from(
        policy
            .get("root")
            .and_then(Value::as_str)
            .ok_or(AccountError::PackageTrust)?,
    );
    if !root.is_absolute() {
        return Err(AccountError::PackageTrust);
    }
    Ok(root)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct InstallRequest {
    pub operation_id: String,
    pub identity_digest: String,
    pub package_id: String,
    pub version: String,
    pub release_revision: String,
    pub artifact_digest: String,
    pub artifact_size: u64,
    pub expected_inventory_revision: String,
    #[serde(
        default = "legacy_package_install_schema_v1",
        skip_serializing_if = "is_legacy_package_install_schema_v1"
    )]
    pub schema_version: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<PackageInstallTarget>,
}

fn legacy_package_install_schema_v1() -> u8 {
    LEGACY_PACKAGE_INSTALL_SCHEMA_V1
}

fn is_legacy_package_install_schema_v1(version: &u8) -> bool {
    *version == LEGACY_PACKAGE_INSTALL_SCHEMA_V1
}

impl InstallRequest {
    pub(super) fn target_mode(&self) -> Option<PackageRuntimeMode> {
        self.target.as_ref().map(|target| target.runtime_mode)
    }

    pub(super) fn validate(&self) -> Result<(), AccountError> {
        validate_id(&self.operation_id)?;
        validate_id(&self.package_id)?;
        validate_revision(&self.release_revision, false)?;
        if !is_digest(&self.identity_digest)
            || !is_digest(&self.artifact_digest)
            || !canonical_revision(&self.expected_inventory_revision)
            || self.version.is_empty()
            || self.version.len() > 64
            || self.version.chars().any(char::is_control)
        {
            return Err(AccountError::ServiceFailure);
        }
        if self.artifact_size == 0 || self.artifact_size > MAX_PACKAGE_BYTES as u64 {
            return Err(AccountError::PackageCapacity);
        }
        match (self.schema_version, self.target.as_ref()) {
            (LEGACY_PACKAGE_INSTALL_SCHEMA_V1, None) => {}
            (PACKAGE_INSTALL_SCHEMA_V2, Some(target)) => target.validate_for_current_host()?,
            _ => return Err(AccountError::ServiceFailure),
        }
        Ok(())
    }

    fn validate_for_new_install(&self) -> Result<PackageInstallTarget, AccountError> {
        self.validate()?;
        if self.schema_version != PACKAGE_INSTALL_SCHEMA_V2 {
            return Err(AccountError::PackageTargetRequired);
        }
        self.target.ok_or(AccountError::PackageTargetRequired)
    }

    fn fingerprint(&self) -> Result<String, AccountError> {
        Ok(digest(
            &serde_json::to_vec(self).map_err(|_| AccountError::ServiceFailure)?,
        ))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ClientConfig {
    #[serde(rename = "schemaVersion")]
    _schema_version: PackageActionSchemaV2,
    executable: PathBuf,
    executable_sha256: String,
    runtime_library: Option<RuntimeLibrary>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimeLibrary {
    path: PathBuf,
    sha256: String,
}

struct PackageClient {
    config: ClientConfig,
    owner: String,
    policy_index_path: PathBuf,
    policy_index_digest: String,
    root: PathBuf,
}

#[cfg(test)]
thread_local! {
    static PACKAGE_DISPATCH_COUNTS: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}

#[cfg(test)]
fn record_package_client_load() {
    PACKAGE_DISPATCH_COUNTS.with(|counts| {
        let (loads, queries) = counts.get();
        counts.set((loads + 1, queries));
    });
}

#[cfg(test)]
fn record_package_helper_query() {
    PACKAGE_DISPATCH_COUNTS.with(|counts| {
        let (loads, queries) = counts.get();
        counts.set((loads, queries + 1));
    });
}

#[cfg(test)]
fn reset_package_dispatch_counts() {
    PACKAGE_DISPATCH_COUNTS.with(|counts| counts.set((0, 0)));
}

#[cfg(test)]
fn package_dispatch_counts() -> (usize, usize) {
    PACKAGE_DISPATCH_COUNTS.with(|counts| counts.get())
}

impl PackageClient {
    async fn load(target: PackageInstallTarget) -> Result<Self, AccountError> {
        #[cfg(test)]
        record_package_client_load();

        tokio::task::spawn_blocking(move || {
            let path = std::env::var_os("ZIRCON_HUB_PACKAGE_CONFIG")
                .map(PathBuf::from)
                .ok_or(AccountError::PackageUnavailable)?;
            let bytes = crate::file_io::read_bounded_regular(&path, 65536)
                .map_err(|_| AccountError::PackageUnavailable)?;
            let config: ClientConfig =
                serde_json::from_slice(&bytes).map_err(|_| AccountError::PackageUnavailable)?;
            if !config.executable.is_absolute()
                || !is_digest(&config.executable_sha256)
                || config.runtime_library.as_ref().is_some_and(|library| {
                    !library.path.is_absolute() || !is_digest(&library.sha256)
                })
            {
                return Err(AccountError::PackageUnavailable);
            }
            let policy_index_path = std::env::var_os(POLICY_INDEX_ENV)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .ok_or(AccountError::PackagePolicyUnconfigured)?;
            if !policy_index_path.is_absolute() {
                return Err(AccountError::PackageTrust);
            }
            let policy_index = crate::file_io::read_bounded_regular(&policy_index_path, 65536)
                .map_err(|_| AccountError::PackageTrust)?;
            let policy_index_digest = digest(&policy_index);
            let policy_index: NativePluginPolicyIndex =
                serde_json::from_slice(&policy_index).map_err(|_| AccountError::PackageTrust)?;
            let selected_policy = selected_policy_index_entry(&policy_index, target)?;
            let policy = crate::file_io::read_bounded_regular(&selected_policy.policy_path, 65536)
                .map_err(|_| AccountError::PackageTrust)?;
            let root = selected_policy_root(selected_policy, target, &policy)?;
            let root = std::fs::canonicalize(root).map_err(|_| AccountError::PackageUnavailable)?;
            #[cfg(windows)]
            let (root_pin, root_identity) = windows::root_identity(&root)?;
            #[cfg(not(windows))]
            let (root_pin, root_identity) = ((), String::new());
            let owner = digest(
                &serde_json::to_vec(&(root_identity, target))
                    .map_err(|_| AccountError::PackageUnavailable)?,
            );
            drop(root_pin);
            Ok(Self {
                config,
                owner,
                policy_index_path,
                policy_index_digest,
                root,
            })
        })
        .await
        .map_err(|_| AccountError::PackageUnavailable)?
    }
}

impl AccountBroker {
    pub async fn package_inventory(
        &self,
        generation: &str,
        organization: String,
        target_mode: PackageRuntimeMode,
    ) -> Result<Value, AccountError> {
        self.catalog_entitlements(generation, organization.clone(), None)
            .await?;
        let identity = self.operation_identity(generation).await?;
        let target = PackageInstallTarget::for_runtime_mode(target_mode)?;
        let client = PackageClient::load(target).await?;
        let response = process::query(&client, serde_json::json!({"action":"inventory","schemaVersion":PACKAGE_INSTALL_SCHEMA_V2,"target":target,"identityDigest":identity_digest(&identity, &organization)?})).await?;
        self.operation_identity(generation).await?;
        validate_target_echo(&response, target)?;
        project_inventory(
            response
                .get("inventory")
                .ok_or(AccountError::PackageUnavailable)?,
            target_mode,
        )
    }

    pub async fn install_package(
        &self,
        generation: &str,
        organization: String,
        operation_id: String,
        package_id: String,
        revision: String,
        expected_inventory_revision: String,
        target_mode: PackageRuntimeMode,
    ) -> Result<Value, AccountError> {
        let permit = self.mutation.try_lock().map_err(|_| AccountError::Busy)?;
        validate_id(&operation_id)?;
        if !canonical_revision(&expected_inventory_revision) {
            return Err(AccountError::ServiceFailure);
        }
        let identity = self.operation_identity(generation).await?;
        let release = self
            .catalog_release(generation, &organization, &package_id, &revision)
            .await?;
        if release.kind != "plugin" {
            return Err(AccountError::PackageTrust);
        }
        let target = PackageInstallTarget::for_runtime_mode(target_mode)?;
        let client = PackageClient::load(target).await?;
        let request = InstallRequest {
            operation_id: operation_id.clone(),
            identity_digest: identity_digest(&identity, &organization)?,
            package_id,
            version: release.version,
            release_revision: revision,
            artifact_digest: release.artifact_digest,
            artifact_size: release.artifact_size,
            expected_inventory_revision,
            schema_version: PACKAGE_INSTALL_SCHEMA_V2,
            target: Some(target),
        };
        let payload = OperationPayload::InstallPackage {
            organization,
            owner: client.owner.clone(),
            request,
        };
        self.install_with_permit(generation, operation_id, payload, false, &permit)
            .await
    }

    pub(super) async fn install_with_permit(
        &self,
        generation: &str,
        operation_id: String,
        payload: OperationPayload,
        retry: bool,
        _permit: &tokio::sync::MutexGuard<'_, ()>,
    ) -> Result<Value, AccountError> {
        let identity = self.operation_identity(generation).await?;
        let OperationPayload::InstallPackage {
            organization,
            owner,
            request,
        } = &payload
        else {
            return Err(AccountError::ServiceFailure);
        };
        payload.validate(&operation_id)?;
        let target = request.validate_for_new_install()?;
        if request.identity_digest != identity_digest(&identity, organization)? {
            return Err(AccountError::ServiceFailure);
        }
        let client = PackageClient::load(target).await?;
        if client.owner != *owner {
            return Err(AccountError::PackageUnavailable);
        }
        let journal = self.operation_journal()?;
        let first = tokio::task::spawn_blocking({
            let journal = journal.clone();
            let identity = identity.clone();
            let operation_id = operation_id.clone();
            let payload = payload.clone();
            let generation = generation.to_owned();
            move || {
                if retry {
                    journal.admit_retry(&identity, &operation_id, &generation, payload)
                } else {
                    journal.admit(&identity, &operation_id, &generation, payload)
                }
            }
        })
        .await
        .map_err(|_| AccountError::OperationStore)??;
        let mut commit_possible = false;
        let result = self
            .perform_install(
                generation,
                organization,
                request,
                &client,
                &mut commit_possible,
            )
            .await;
        let status = match &result {
            Ok(_) => OperationStatus::Committed,
            Err(_) if first && !commit_possible => OperationStatus::Failed,
            _ => OperationStatus::Unknown,
        };
        tokio::task::spawn_blocking(move || journal.finish(&identity, &operation_id, status))
            .await
            .map_err(|_| AccountError::OutcomeUnknown)??;
        if !first && result.is_err() {
            return Err(AccountError::OutcomeUnknown);
        }
        self.operation_identity(generation)
            .await
            .map_err(|_| AccountError::OutcomeUnknown)?;
        result
    }

    async fn perform_install(
        &self,
        generation: &str,
        organization: &str,
        request: &InstallRequest,
        client: &PackageClient,
        commit_possible: &mut bool,
    ) -> Result<Value, AccountError> {
        let mut cancellation = self.cancel.subscribe();
        let attempt = *cancellation.borrow();
        let release = self
            .catalog_release(
                generation,
                organization,
                &request.package_id,
                &request.release_revision,
            )
            .await?;
        if release.version != request.version
            || release.artifact_digest != request.artifact_digest
            || release.artifact_size != request.artifact_size
            || release.kind != "plugin"
        {
            return Err(AccountError::PackageTrust);
        }
        let bytes = tokio::select! {
            biased;
            _ = cancellation.changed() => return Err(AccountError::Cancelled),
            result = self.package_bytes(generation, organization, &release) => result?,
        };
        self.operation_identity(generation).await?;
        let mut session = process::Session::spawn(client).await?;
        let result = async {
            let prepared = tokio::select! {
                biased;
                _ = cancellation.changed() => return Err(AccountError::Cancelled),
                result = session.prepare(request, bytes) => result?,
            };
            if prepared.get("status").and_then(Value::as_str) == Some("committed") {
                *commit_possible = true;
                return project_receipt(
                    request,
                    prepared
                        .get("receipt")
                        .ok_or(AccountError::OutcomeUnknown)?,
                )
                .ok_or(AccountError::OutcomeUnknown);
            }
            let fresh = self
                .catalog_release(
                    generation,
                    organization,
                    &request.package_id,
                    &request.release_revision,
                )
                .await?;
            if fresh.artifact_digest != request.artifact_digest
                || fresh.artifact_size != request.artifact_size
            {
                return Err(AccountError::PackageTrust);
            }
            let response = {
                // Holding account state orders the commit authorization before a later generation.
                let state = self.state.lock().await;
                if state.view.generation != generation
                    || state.authenticated.is_none()
                    || *cancellation.borrow() != attempt
                {
                    return Err(AccountError::Cancelled);
                }
                *commit_possible = true;
                session.authorize_commit().await?;
                drop(state);
                session.committed(commit_possible).await?
            };
            project_receipt(
                request,
                response
                    .get("receipt")
                    .ok_or(AccountError::OutcomeUnknown)?,
            )
            .ok_or(AccountError::OutcomeUnknown)
        }
        .await;
        session.close().await;
        result
    }

    pub(super) async fn reconcile_install(
        &self,
        generation: &str,
        payload: &OperationPayload,
    ) -> Result<Option<Value>, AccountError> {
        let OperationPayload::InstallPackage {
            organization,
            owner,
            request,
        } = payload
        else {
            return Err(AccountError::ServiceFailure);
        };
        let identity = self.operation_identity(generation).await?;
        if identity_digest(&identity, organization)? != request.identity_digest {
            return Err(AccountError::ServiceFailure);
        }
        let target = request.validate_for_new_install()?;
        // Recovery reads local truth even after an entitlement is revoked; no new install occurs.
        let client = PackageClient::load(target).await?;
        if client.owner != *owner {
            return Err(AccountError::PackageUnavailable);
        }
        let response = process::query(&client, serde_json::json!({"action":"receipt","schemaVersion":PACKAGE_INSTALL_SCHEMA_V2,"target":target,"identityDigest":request.identity_digest,"operationId":request.operation_id})).await?;
        self.operation_identity(generation).await?;
        validate_target_echo(&response, target)?;
        let receipt = response
            .get("receipt")
            .ok_or(AccountError::PackageUnavailable)?;
        if receipt.is_null() {
            return Ok(None);
        }
        project_receipt(request, receipt)
            .map(Some)
            .ok_or(AccountError::OutcomeUnknown)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InstalledPackage {
    operation_id: String,
    package_id: String,
    version: String,
    release_revision: String,
    artifact_digest: String,
    slot: String,
    files: std::collections::BTreeMap<String, String>,
}

fn installed_view(item: &InstalledPackage) -> Result<Value, AccountError> {
    validate_id(&item.operation_id)?;
    validate_id(&item.package_id)?;
    validate_revision(&item.release_revision, false)?;
    if item.version.is_empty()
        || item.version.len() > 64
        || !is_digest(&item.artifact_digest)
        || item.slot.is_empty()
        || item.files.is_empty()
        || item.files.len() > 1024
    {
        return Err(AccountError::PackageUnavailable);
    }
    Ok(
        serde_json::json!({"operationId":item.operation_id,"packageId":item.package_id,"version":item.version,"releaseRevision":item.release_revision,"artifactDigest":item.artifact_digest}),
    )
}

fn project_inventory(
    value: &Value,
    target_mode: PackageRuntimeMode,
) -> Result<Value, AccountError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Inventory {
        schema_version: u32,
        revision: String,
        packages: Vec<InstalledPackage>,
    }
    let inventory: Inventory =
        serde_json::from_value(value.clone()).map_err(|_| AccountError::PackageUnavailable)?;
    if inventory.schema_version != 1
        || !canonical_revision(&inventory.revision)
        || inventory.packages.len() > 128
    {
        return Err(AccountError::PackageUnavailable);
    }
    let mut ids = std::collections::HashSet::new();
    let mut packages = Vec::new();
    for item in &inventory.packages {
        if !ids.insert(&item.package_id) {
            return Err(AccountError::PackageUnavailable);
        }
        packages.push(installed_view(item)?);
    }
    Ok(
        serde_json::json!({"schemaVersion":2,"targetMode":target_mode.wire_name(),"revision":inventory.revision,"packages":packages}),
    )
}

pub(super) fn project_receipt(request: &InstallRequest, value: &Value) -> Option<Value> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Receipt {
        schema_version: u32,
        operation_id: String,
        request_digest: String,
        inventory_revision: String,
        package: InstalledPackage,
        #[serde(default)]
        plugin_id: Option<String>,
        #[serde(default)]
        target: Option<PackageInstallTarget>,
    }
    let receipt: Receipt = serde_json::from_value(value.clone()).ok()?;
    let expected_target = request.target.as_ref()?;
    if receipt.schema_version != 2
        || receipt.operation_id != request.operation_id
        || receipt.request_digest != request.fingerprint().ok()?
        || !canonical_revision(&receipt.inventory_revision)
        || receipt.inventory_revision == "0"
        || receipt.package.operation_id != request.operation_id
        || receipt.package.package_id != request.package_id
        || receipt.package.version != request.version
        || receipt.package.release_revision != request.release_revision
        || receipt.package.artifact_digest != request.artifact_digest
        || receipt.plugin_id.as_deref().unwrap_or("").is_empty()
        || receipt
            .target
            .as_ref()
            .is_some_and(|target| target != expected_target)
        || (receipt.schema_version == 2 && receipt.target.as_ref() != Some(expected_target))
    {
        return None;
    }
    let target_mode = request.target.as_ref()?.runtime_mode.wire_name();
    Some(
        serde_json::json!({"schemaVersion":2,"targetMode":target_mode,"operationId":receipt.operation_id,"inventoryRevision":receipt.inventory_revision,"package":installed_view(&receipt.package).ok()?}),
    )
}

fn identity_digest(
    identity: &OperationIdentity,
    organization: &str,
) -> Result<String, AccountError> {
    validate_id(organization)?;
    Ok(digest(
        &serde_json::to_vec(&(identity, organization)).map_err(|_| AccountError::ServiceFailure)?,
    ))
}

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn canonical_revision(value: &str) -> bool {
    value
        .parse::<u64>()
        .is_ok_and(|number| number.to_string() == value)
}
