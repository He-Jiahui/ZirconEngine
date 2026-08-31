use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use zircon_runtime::asset::migration::{
    migrate_project_assets, AssetMigrationIssueKind, AssetMigrationMode, AssetMigrationOptions,
    AssetMigrationReport,
};

use crate::core::commands::{
    CommandEvalCtx, EditorCommandAction, EditorCommandDescriptor, EditorCommandRegistry,
};
use crate::core::plugin::{EditorPluginCatalogProjection, EditorPluginManager};

const ASSET_MIGRATION_CAPABILITY: &str = "asset.migration";
const PLUGIN_CATALOG_READ_CAPABILITY: &str = "plugin.catalog.read";

/// Fully parsed invocation of a headless editor commandlet.
///
/// The descriptor is the immutable registry token resolved during parsing. Execution consumes
/// this same token so one invocation never rebuilds the registry or resolves its route again.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandletRequest {
    command: String,
    descriptor: EditorCommandDescriptor,
    project_root: Option<PathBuf>,
    mode: Option<AssetMigrationMode>,
    automation_path: Option<PathBuf>,
}

impl CommandletRequest {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn project_root(&self) -> Option<&std::path::Path> {
        self.project_root.as_deref()
    }

    pub fn mode(&self) -> Option<AssetMigrationMode> {
        self.mode
    }

    pub fn automation_path(&self) -> Option<&std::path::Path> {
        self.automation_path.as_deref()
    }

    fn descriptor(&self) -> &EditorCommandDescriptor {
        &self.descriptor
    }
}

/// Typed input for the retained-host authoring commandlet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoringAutomationCommandletRequest {
    project_root: PathBuf,
    automation_path: PathBuf,
}

impl AuthoringAutomationCommandletRequest {
    pub fn project_root(&self) -> &std::path::Path {
        &self.project_root
    }

    pub fn automation_path(&self) -> &std::path::Path {
        &self.automation_path
    }
}

/// Process-owned work required by commandlets that intentionally use a retained editor host.
pub trait CommandletHost {
    type AuthoringAutomationReport: Serialize;
    type Error: Display;

    fn run_authoring_automation(
        &self,
        request: &AuthoringAutomationCommandletRequest,
    ) -> Result<Self::AuthoringAutomationReport, Self::Error>;
}

/// Four stable process outcomes shared by every editor commandlet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandletExitCode {
    Success,
    Failed,
    InvalidArguments,
    MissingCapability,
}

impl CommandletExitCode {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Failed => 1,
            Self::InvalidArguments => 2,
            Self::MissingCapability => 3,
        }
    }
}

impl Serialize for CommandletExitCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.as_u8())
    }
}

/// Semantic status returned in the stable commandlet JSON envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandletStatus {
    Succeeded,
    Failed,
    InvalidArguments,
    MissingCapabilities,
}

/// Stable JSON-safe summary of a migration report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommandletMigrationReport {
    pub mode: String,
    pub scanned_files: usize,
    pub changed_files: Vec<CommandletMigrationChange>,
    pub issues: Vec<CommandletMigrationIssue>,
    pub applied: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommandletMigrationChange {
    pub path: String,
    pub reference_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommandletMigrationIssue {
    pub kind: &'static str,
    pub path: Option<String>,
    pub message: String,
}

/// Result envelope printed by the executable for both success and failure paths.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandletReport<T = ()> {
    pub command: Option<String>,
    pub status: CommandletStatus,
    pub exit_code: CommandletExitCode,
    pub migration: Option<CommandletMigrationReport>,
    pub plugins: Option<Arc<EditorPluginCatalogProjection>>,
    pub automation: Option<T>,
    pub error: Option<String>,
}

impl<T> CommandletReport<T> {
    pub fn exit_code(&self) -> CommandletExitCode {
        self.exit_code
    }

    pub fn status(&self) -> CommandletStatus {
        self.status
    }

    pub fn migration(&self) -> Option<&CommandletMigrationReport> {
        self.migration.as_ref()
    }

    pub fn plugins(&self) -> Option<&EditorPluginCatalogProjection> {
        self.plugins.as_deref()
    }

    pub fn plugin_catalog_projection(&self) -> Option<&Arc<EditorPluginCatalogProjection>> {
        self.plugins.as_ref()
    }

    pub fn automation(&self) -> Option<&T> {
        self.automation.as_ref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn invalid_arguments(command: Option<String>, error: impl Into<String>) -> Self {
        Self {
            command,
            status: CommandletStatus::InvalidArguments,
            exit_code: CommandletExitCode::InvalidArguments,
            migration: None,
            plugins: None,
            automation: None,
            error: Some(error.into()),
        }
    }

    fn failed(
        command: impl Into<String>,
        error: impl Into<String>,
        migration: Option<CommandletMigrationReport>,
    ) -> Self {
        Self {
            command: Some(command.into()),
            status: CommandletStatus::Failed,
            exit_code: CommandletExitCode::Failed,
            migration,
            plugins: None,
            automation: None,
            error: Some(error.into()),
        }
    }

    fn missing_capabilities(command: impl Into<String>, capabilities: Vec<String>) -> Self {
        Self {
            command: Some(command.into()),
            status: CommandletStatus::MissingCapabilities,
            exit_code: CommandletExitCode::MissingCapability,
            migration: None,
            plugins: None,
            automation: None,
            error: Some(format_missing_capabilities(&capabilities)),
        }
    }

    fn succeeded_migration(
        command: impl Into<String>,
        migration: CommandletMigrationReport,
    ) -> Self {
        Self {
            command: Some(command.into()),
            status: CommandletStatus::Succeeded,
            exit_code: CommandletExitCode::Success,
            migration: Some(migration),
            plugins: None,
            automation: None,
            error: None,
        }
    }

    fn succeeded_plugin_list(
        command: impl Into<String>,
        plugins: Arc<EditorPluginCatalogProjection>,
    ) -> Self {
        Self {
            command: Some(command.into()),
            status: CommandletStatus::Succeeded,
            exit_code: CommandletExitCode::Success,
            migration: None,
            plugins: Some(plugins),
            automation: None,
            error: None,
        }
    }

    fn succeeded_authoring_automation(command: impl Into<String>, automation: T) -> Self {
        Self {
            command: Some(command.into()),
            status: CommandletStatus::Succeeded,
            exit_code: CommandletExitCode::Success,
            migration: None,
            plugins: None,
            automation: Some(automation),
            error: None,
        }
    }
}

fn format_missing_capabilities(capabilities: &[String]) -> String {
    const PREFIX: &str = "commandlet requires unavailable capabilities: ";
    const SEPARATOR: &str = ", ";

    let capability_bytes = capabilities.iter().map(String::len).sum::<usize>();
    let mut message = String::with_capacity(
        PREFIX.len() + capability_bytes + SEPARATOR.len() * capabilities.len().saturating_sub(1),
    );
    message.push_str(PREFIX);
    if let Some((first, remaining)) = capabilities.split_first() {
        message.push_str(first);
        for capability in remaining {
            message.push_str(SEPARATOR);
            message.push_str(capability);
        }
    }
    message
}

impl<T> Serialize for CommandletReport<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct SerializableCommandletReport<'a, T> {
            command: &'a Option<String>,
            status: CommandletStatus,
            exit_code: CommandletExitCode,
            migration: &'a Option<CommandletMigrationReport>,
            plugins: Option<&'a EditorPluginCatalogProjection>,
            automation: &'a Option<T>,
            error: &'a Option<String>,
        }

        SerializableCommandletReport {
            command: &self.command,
            status: self.status,
            exit_code: self.exit_code,
            migration: &self.migration,
            plugins: self.plugins.as_deref(),
            automation: &self.automation,
            error: &self.error,
        }
        .serialize(serializer)
    }
}

/// Parse a `--run <commandlet>` invocation without creating an application-local registry.
/// Calls without `--run` remain available to the normal editor startup parser.
pub fn parse_commandlet_args<I, S>(args: I) -> Result<Option<CommandletRequest>, CommandletReport>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if !args.iter().any(|argument| argument == "--run") {
        return Ok(None);
    }

    let mut command = None;
    let mut project_root = None;
    let mut automation_path = None;
    let mut dry_run = false;
    let mut apply = false;
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        match argument.as_str() {
            "--run" => {
                if command.is_some() {
                    return Err(CommandletReport::invalid_arguments(
                        command,
                        "--run was provided more than once",
                    ));
                }
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(CommandletReport::invalid_arguments(
                        None,
                        "--run requires a commandlet name",
                    ));
                };
                command = Some(value.clone());
            }
            "--project" => {
                if project_root.is_some() {
                    return Err(CommandletReport::invalid_arguments(
                        command,
                        "--project was provided more than once",
                    ));
                }
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(CommandletReport::invalid_arguments(
                        command,
                        "--project requires a project root",
                    ));
                };
                project_root = Some(PathBuf::from(value));
            }
            "--automation" => {
                if automation_path.is_some() {
                    return Err(CommandletReport::invalid_arguments(
                        command,
                        "--automation was provided more than once",
                    ));
                }
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(CommandletReport::invalid_arguments(
                        command,
                        "--automation requires a JSON file path",
                    ));
                };
                automation_path = Some(PathBuf::from(value));
            }
            "--dry-run" => {
                if dry_run {
                    return Err(CommandletReport::invalid_arguments(
                        command,
                        "--dry-run was provided more than once",
                    ));
                }
                dry_run = true;
            }
            "--apply" => {
                if apply {
                    return Err(CommandletReport::invalid_arguments(
                        command,
                        "--apply was provided more than once",
                    ));
                }
                apply = true;
            }
            other => {
                return Err(CommandletReport::invalid_arguments(
                    command,
                    format!("unknown commandlet argument `{other}`"),
                ));
            }
        }
        index += 1;
    }

    let Some(command) = command else {
        return Err(CommandletReport::invalid_arguments(
            None,
            "--run requires a commandlet name",
        ));
    };
    let registry = EditorCommandRegistry::default_workbench();
    let Some(descriptor) = registry.command_for_headless_commandlet_name(&command) else {
        return Err(CommandletReport::invalid_arguments(
            Some(command),
            "unknown editor commandlet",
        ));
    };
    let is_authoring_automation = matches!(
        descriptor.action(),
        EditorCommandAction::HeadlessAuthoringAutomation
    );
    if is_authoring_automation {
        if project_root.is_none() {
            return Err(CommandletReport::invalid_arguments(
                Some(command),
                "authoring-automation requires --project",
            ));
        }
        if automation_path.is_none() {
            return Err(CommandletReport::invalid_arguments(
                Some(command),
                "authoring-automation requires --automation",
            ));
        }
    } else if automation_path.is_some() {
        return Err(CommandletReport::invalid_arguments(
            Some(command),
            "--automation is only accepted by authoring-automation",
        ));
    }
    let mode = match (dry_run, apply) {
        (true, false) => Some(AssetMigrationMode::DryRun),
        (false, true) => Some(AssetMigrationMode::Apply),
        (true, true) => {
            return Err(CommandletReport::invalid_arguments(
                Some(command),
                "--dry-run and --apply are mutually exclusive",
            ));
        }
        (false, false) => None,
    };
    if is_authoring_automation && mode.is_some() {
        return Err(CommandletReport::invalid_arguments(
            Some(command),
            "authoring-automation does not accept migration mode arguments",
        ));
    }
    Ok(Some(CommandletRequest {
        command,
        descriptor: descriptor.clone(),
        project_root,
        mode,
        automation_path,
    }))
}

/// Run a commandlet with the capabilities provided by the headless editor profile.
pub fn run_commandlet(request: CommandletRequest) -> CommandletReport {
    run_commandlet_with_capabilities_and_host(
        request,
        [ASSET_MIGRATION_CAPABILITY, PLUGIN_CATALOG_READ_CAPABILITY],
        &NoopCommandletHost,
    )
}

/// Run a commandlet that needs the editor executable's retained-host composition.
pub fn run_commandlet_with_host<H>(
    request: CommandletRequest,
    host: &H,
) -> CommandletReport<H::AuthoringAutomationReport>
where
    H: CommandletHost,
{
    run_commandlet_with_capabilities_and_host(
        request,
        [ASSET_MIGRATION_CAPABILITY, PLUGIN_CATALOG_READ_CAPABILITY],
        host,
    )
}

/// Run a commandlet under an explicit capability projection. This is the headless capability
/// gate used by the CLI and keeps missing feature support distinct from task failure.
pub fn run_commandlet_with_capabilities<I, S>(
    request: CommandletRequest,
    capabilities: I,
) -> CommandletReport
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    run_commandlet_with_capabilities_and_host(request, capabilities, &NoopCommandletHost)
}

fn run_commandlet_with_capabilities_and_host<I, S, H>(
    request: CommandletRequest,
    capabilities: I,
    host: &H,
) -> CommandletReport<H::AuthoringAutomationReport>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
    H: CommandletHost,
{
    let descriptor = request.descriptor();
    if !descriptor.callable_from_remote() {
        return CommandletReport::failed(
            request.command,
            "editor commandlet is not callable from remote control",
            None,
        );
    }
    let context = CommandEvalCtx::headless(capabilities);
    if let Err(error) = EditorCommandRegistry::ensure_enabled(descriptor, &context) {
        let missing = descriptor.missing_required_capabilities(&context);
        if !missing.is_empty() {
            return CommandletReport::missing_capabilities(request.command, missing);
        }
        return CommandletReport::failed(request.command, error.to_string(), None);
    }

    let action = descriptor.action().clone();
    match action {
        EditorCommandAction::HeadlessAssetMigration => run_asset_migration_commandlet(request),
        EditorCommandAction::HeadlessPluginList => run_plugin_list_commandlet(request),
        EditorCommandAction::HeadlessAuthoringAutomation => {
            run_authoring_automation_commandlet(request, host)
        }
        _ => CommandletReport::failed(
            request.command,
            "canonical command descriptor has no headless commandlet action",
            None,
        ),
    }
}

fn run_asset_migration_commandlet<T>(request: CommandletRequest) -> CommandletReport<T> {
    let CommandletRequest {
        command,
        project_root,
        mode,
        ..
    } = request;
    let Some(project_root) = project_root else {
        return CommandletReport::invalid_arguments(
            Some(command),
            "migrate-assets requires --project",
        );
    };
    let Some(mode) = mode else {
        return CommandletReport::invalid_arguments(
            Some(command),
            "migrate-assets requires exactly one of --dry-run or --apply",
        );
    };
    match migrate_project_assets(AssetMigrationOptions::new(project_root, mode)) {
        Ok(report) if report.succeeded() => {
            CommandletReport::succeeded_migration(command, migration_report(&report))
        }
        Ok(report) => CommandletReport::failed(
            command,
            "asset migration reported one or more issues",
            Some(migration_report(&report)),
        ),
        Err(error) => CommandletReport::failed(command, error.to_string(), None),
    }
}

fn run_plugin_list_commandlet<T>(request: CommandletRequest) -> CommandletReport<T> {
    let CommandletRequest {
        command,
        project_root,
        mode,
        automation_path,
        ..
    } = request;
    if project_root.is_some() || mode.is_some() || automation_path.is_some() {
        return CommandletReport::invalid_arguments(
            Some(command),
            "plugin-list does not accept project or migration mode arguments",
        );
    }
    let snapshot = match EditorPluginManager::builtin_shared() {
        Ok(manager) => manager.catalog_snapshot(),
        Err(error) => {
            return CommandletReport::failed(
                command,
                format!("plugin catalog initialization failed: {error}"),
                None,
            );
        }
    };
    CommandletReport::succeeded_plugin_list(command, Arc::clone(snapshot.projection()))
}

fn run_authoring_automation_commandlet<H>(
    request: CommandletRequest,
    host: &H,
) -> CommandletReport<H::AuthoringAutomationReport>
where
    H: CommandletHost,
{
    let CommandletRequest {
        command,
        project_root,
        mode,
        automation_path,
        ..
    } = request;
    let (Some(project_root), Some(automation_path)) = (project_root, automation_path) else {
        return CommandletReport::invalid_arguments(
            Some(command),
            "authoring-automation requires --project and --automation",
        );
    };
    if mode.is_some() {
        return CommandletReport::invalid_arguments(
            Some(command),
            "authoring-automation does not accept migration mode arguments",
        );
    }
    let request = AuthoringAutomationCommandletRequest {
        project_root,
        automation_path,
    };
    match host.run_authoring_automation(&request) {
        Ok(report) => CommandletReport::succeeded_authoring_automation(command, report),
        Err(error) => CommandletReport::failed(
            command,
            format!("authoring automation host failed: {error}"),
            None,
        ),
    }
}

struct NoopCommandletHost;

impl CommandletHost for NoopCommandletHost {
    type AuthoringAutomationReport = ();
    type Error = &'static str;

    fn run_authoring_automation(
        &self,
        _: &AuthoringAutomationCommandletRequest,
    ) -> Result<Self::AuthoringAutomationReport, Self::Error> {
        Err("authoring-automation requires an editor process host")
    }
}

fn migration_report(report: &AssetMigrationReport) -> CommandletMigrationReport {
    CommandletMigrationReport {
        mode: migration_mode_name(report.mode()).to_string(),
        scanned_files: report.scanned_files(),
        changed_files: report
            .changed_files()
            .iter()
            .map(|change| CommandletMigrationChange {
                path: change.path().display().to_string(),
                reference_count: change.reference_count(),
            })
            .collect(),
        issues: report
            .issues()
            .iter()
            .map(|issue| CommandletMigrationIssue {
                kind: issue_kind_name(issue.kind()),
                path: issue.path().map(|path| path.display().to_string()),
                message: issue.message().to_string(),
            })
            .collect(),
        applied: report.applied(),
    }
}

fn migration_mode_name(mode: AssetMigrationMode) -> &'static str {
    match mode {
        AssetMigrationMode::DryRun => "dry_run",
        AssetMigrationMode::Apply => "apply",
    }
}

fn issue_kind_name(kind: AssetMigrationIssueKind) -> &'static str {
    match kind {
        AssetMigrationIssueKind::PendingRecovery => "pending_recovery",
        AssetMigrationIssueKind::DanglingReference => "dangling_reference",
        AssetMigrationIssueKind::MissingGuid => "missing_guid",
        AssetMigrationIssueKind::MissingPath => "missing_path",
        AssetMigrationIssueKind::RegistryConflict => "registry_conflict",
        AssetMigrationIssueKind::AmbiguousPath => "ambiguous_path",
        AssetMigrationIssueKind::UnsupportedScheme => "unsupported_scheme",
        AssetMigrationIssueKind::InvalidDocument => "invalid_document",
        AssetMigrationIssueKind::UnsafePath => "unsafe_path",
        AssetMigrationIssueKind::PathIo => "path_io",
    }
}

#[cfg(test)]
#[path = "runner/capability_error_tests.rs"]
mod capability_error_tests;
