#![forbid(unsafe_code)]

mod client_hud_projection;
mod client_projection;
mod client_window_projection;
mod presentation;
mod transaction;

pub use client_hud_projection::{
    ClientPresentationProjection, ClientProjectionCodecError, ClientProjectionError, HudAction,
    HudActionId, HudCast, HudMeter, HudProjection, HudProjectionError, HudQuestObjective,
    HudQuestState, HudResource, HudResourceKind, HudTrackedQuest, HudUnit, HudUnitRole,
    CLIENT_PRESENTATION_SCHEMA_VERSION, MAX_CLIENT_PRESENTATION_BYTES,
};
pub use client_projection::{
    ActorAnimationInput, ActorAppearance, ActorPresentation, ActorTransform,
    BulkPresentationProjection, PresentationProjectionError, PresentationVec3,
};
pub use client_window_projection::{
    ClientWindowProjection, EquippedBagProjection, InventoryItemProjection,
    InventoryProjectionError, InventoryWindowProjection, QuestLogEntryProjection,
    QuestLogObjectiveProjection, QuestLogProjectionError, QuestLogWindowProjection,
    WindowProjectionError, INVENTORY_BAG_SOCKET_COUNT,
};
pub use presentation::{
    PresentationBlendMode, PresentationCadence, PresentationCadenceError, PresentationSample,
    PresentationSnapshot, PresentationTimeline, PresentationTimelineError,
    PresentationTimelinePush,
};
pub use transaction::{
    BudgetKind, CommittedSnapshot, RuntimeRole, RuntimeStatus, TickBudgets, TickUsage,
    VmReloadStage, VmTickError, VmTickResult, WocOfflineBootstrapError, WocProjectVm,
    WocReloadError, WocReloadableVm, WocTickFault, WocTickFaultKind, WocTransactionalRuntime,
};

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WocHostRole {
    Client,
    Server,
    Bot,
    Headless,
}

impl WocHostRole {
    fn runtime_target_mode(self) -> &'static str {
        match self {
            Self::Client => "client_runtime",
            Self::Server | Self::Bot | Self::Headless => "server_runtime",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WocProjectIdentity {
    pub project_name: String,
    pub source_commit: String,
    pub contract_schema_fingerprint: String,
    pub command_catalog_sha256: String,
    pub command_payload_schema_sha256: String,
    pub world_state_format: String,
    pub world_state_schema_version: u16,
    pub script_package: String,
    pub backend: String,
    pub zr_vm_project: String,
    pub zr_vm_entry_module: String,
    pub zr_vm_execution_mode: String,
    pub role: WocHostRole,
    pub simulation_hz: u32,
    pub presentation_hz: u32,
}

#[derive(Debug, Error)]
pub enum WocProjectIdentityError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse TOML {path}: {source}")]
    Toml {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("failed to parse JSON {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("invalid WOC project identity: {0}")]
    Invalid(String),
}

pub fn inspect_project(
    project_root: impl AsRef<Path>,
    role: WocHostRole,
) -> Result<WocProjectIdentity, WocProjectIdentityError> {
    let root = project_root.as_ref();
    let project_path = root.join("zircon-project.toml");
    let project: toml::Value = read_toml(&project_path)?;
    let project_name = required_toml_string(&project, "name")?;
    let startup_packages = project
        .get("scripts")
        .and_then(|scripts| scripts.get("startup_packages"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| {
            WocProjectIdentityError::Invalid("missing scripts.startup_packages".into())
        })?;
    if !startup_packages
        .iter()
        .any(|package| package.as_str() == Some("woc_game"))
    {
        return Err(WocProjectIdentityError::Invalid(
            "woc_game is not a startup package".into(),
        ));
    }

    let package_path = root.join("scripts/woc_game/plugin.toml");
    let package: toml::Value = read_toml(&package_path)?;
    let package_name = required_toml_string(&package, "name")?;
    let backend = required_toml_string(&package, "backend")?;
    if backend != "zr_vm:project" {
        return Err(WocProjectIdentityError::Invalid(format!(
            "script backend is {backend}, expected zr_vm:project"
        )));
    }
    let package_entry = required_toml_string(&package, "entry")?;
    if package_entry != "main" {
        return Err(WocProjectIdentityError::Invalid(format!(
            "script package entry must be main, got {package_entry}"
        )));
    }
    let zr_vm = package
        .get("zr_vm")
        .ok_or_else(|| WocProjectIdentityError::Invalid("missing zr_vm project binding".into()))?;
    let zr_vm_project = required_toml_string(zr_vm, "project")?;
    if zr_vm_project != "woc_game.zrp" {
        return Err(WocProjectIdentityError::Invalid(format!(
            "zr_vm.project must be woc_game.zrp, got {zr_vm_project}"
        )));
    }
    let zr_vm_entry_module = required_toml_string(zr_vm, "entry_module")?;
    if zr_vm_entry_module != package_entry {
        return Err(WocProjectIdentityError::Invalid(format!(
            "zr_vm.entry_module {zr_vm_entry_module} does not match package entry {package_entry}"
        )));
    }
    let zr_vm_execution_mode = required_toml_string(zr_vm, "execution_mode")?;
    if zr_vm_execution_mode != "interp" {
        return Err(WocProjectIdentityError::Invalid(format!(
            "zr_vm.execution_mode must be interp, got {zr_vm_execution_mode}"
        )));
    }

    let zr_vm_project_path = root.join("scripts/woc_game").join(&zr_vm_project);
    let zr_vm_project_manifest = read_json(&zr_vm_project_path)?;
    let zr_vm_project_name = required_json_string(&zr_vm_project_manifest, "name")?;
    if zr_vm_project_name != package_name {
        return Err(WocProjectIdentityError::Invalid(format!(
            "ZrVM project name {zr_vm_project_name} does not match package {package_name}"
        )));
    }
    let zr_vm_project_source = required_json_string(&zr_vm_project_manifest, "source")?;
    if zr_vm_project_source != "src" {
        return Err(WocProjectIdentityError::Invalid(format!(
            "ZrVM project source must be src, got {zr_vm_project_source}"
        )));
    }
    let zr_vm_project_binary = required_json_string(&zr_vm_project_manifest, "binary")?;
    if zr_vm_project_binary != "bin" {
        return Err(WocProjectIdentityError::Invalid(format!(
            "ZrVM project binary must be bin, got {zr_vm_project_binary}"
        )));
    }
    let zr_vm_project_entry = required_json_string(&zr_vm_project_manifest, "entry")?;
    if zr_vm_project_entry != zr_vm_entry_module {
        return Err(WocProjectIdentityError::Invalid(format!(
            "ZrVM project entry {zr_vm_project_entry} does not match zr_vm.entry_module {zr_vm_entry_module}"
        )));
    }

    let target_mode = role.runtime_target_mode();
    require_plugin_for_target_mode(&project, "zr_vm_language", target_mode)?;
    require_plugin_for_target_mode(&project, "woc_runtime", target_mode)?;

    let source_path = root.join("reference/current-head/source_manifest.json");
    let source = read_json(&source_path)?;
    let source_commit = source
        .get("source_commit")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| WocProjectIdentityError::Invalid("missing source_commit".into()))?;
    if source_commit != woc_protocol::REFERENCE_COMMIT {
        return Err(WocProjectIdentityError::Invalid(format!(
            "source commit is {source_commit}, expected {}",
            woc_protocol::REFERENCE_COMMIT
        )));
    }

    Ok(WocProjectIdentity {
        project_name,
        source_commit: source_commit.to_string(),
        contract_schema_fingerprint: woc_protocol::SCHEMA_FINGERPRINT_HEX.to_string(),
        command_catalog_sha256: woc_protocol::COMMAND_CATALOG_SHA256.to_string(),
        command_payload_schema_sha256: woc_protocol::COMMAND_PAYLOAD_SCHEMA_SHA256.to_string(),
        world_state_format: woc_protocol::WORLD_STATE_FORMAT.to_string(),
        world_state_schema_version: woc_protocol::WORLD_STATE_SCHEMA_VERSION,
        script_package: package_name,
        backend,
        zr_vm_project,
        zr_vm_entry_module,
        zr_vm_execution_mode,
        role,
        simulation_hz: woc_protocol::SIMULATION_HZ,
        presentation_hz: woc_protocol::PRESENTATION_HZ,
    })
}

pub fn identity_report_json(
    project_root: impl AsRef<Path>,
    role: WocHostRole,
) -> Result<String, WocProjectIdentityError> {
    let identity = inspect_project(project_root, role)?;
    serde_json::to_string_pretty(&identity).map_err(|source| WocProjectIdentityError::Json {
        path: PathBuf::from("<identity-report>"),
        source,
    })
}

fn read_toml(path: &Path) -> Result<toml::Value, WocProjectIdentityError> {
    let source = fs::read_to_string(path).map_err(|source| WocProjectIdentityError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&source).map_err(|source| WocProjectIdentityError::Toml {
        path: path.to_path_buf(),
        source,
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value, WocProjectIdentityError> {
    let source = fs::read_to_string(path).map_err(|source| WocProjectIdentityError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&source).map_err(|source| WocProjectIdentityError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn required_toml_string(
    document: &toml::Value,
    field: &str,
) -> Result<String, WocProjectIdentityError> {
    document
        .get(field)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| WocProjectIdentityError::Invalid(format!("missing string field {field}")))
}

fn required_json_string(
    document: &serde_json::Value,
    field: &str,
) -> Result<String, WocProjectIdentityError> {
    document
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| WocProjectIdentityError::Invalid(format!("missing string field {field}")))
}

fn require_plugin_for_target_mode(
    project: &toml::Value,
    plugin_id: &str,
    target_mode: &str,
) -> Result<(), WocProjectIdentityError> {
    let selections = project
        .get("plugins")
        .and_then(|plugins| plugins.get("selections"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| WocProjectIdentityError::Invalid("missing plugins.selections".into()))?;
    let selection = selections
        .iter()
        .find(|selection| selection.get("id").and_then(toml::Value::as_str) == Some(plugin_id))
        .ok_or_else(|| {
            WocProjectIdentityError::Invalid(format!(
                "missing required plugin selection {plugin_id}"
            ))
        })?;
    let enabled = selection
        .get("enabled")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let target_modes = selection
        .get("target_modes")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| {
            WocProjectIdentityError::Invalid(format!(
                "{plugin_id} is not enabled for {target_mode}"
            ))
        })?;
    let enabled_for_target = enabled
        && target_modes
            .iter()
            .any(|mode| mode.as_str() == Some(target_mode));
    if !enabled_for_target {
        return Err(WocProjectIdentityError::Invalid(format!(
            "{plugin_id} is not enabled for {target_mode}"
        )));
    }
    if selection.get("required").and_then(toml::Value::as_bool) != Some(true) {
        return Err(WocProjectIdentityError::Invalid(format!(
            "{plugin_id} is not required for {target_mode}"
        )));
    }
    Ok(())
}
