use std::path::PathBuf;

use serde::Serialize;
use zircon_runtime::asset::migration::{
    migrate_project_assets, AssetMigrationIssueKind, AssetMigrationMode, AssetMigrationOptions,
    AssetMigrationReport,
};

use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry};

const MIGRATE_ASSETS_COMMANDLET: &str = "migrate-assets";
const MIGRATE_ASSETS_OPERATION: &str = "asset.migration.migrate_assets";
const ASSET_MIGRATION_CAPABILITY: &str = "asset.migration";

/// Fully parsed invocation of a headless editor commandlet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandletRequest {
    command: String,
    project_root: PathBuf,
    mode: AssetMigrationMode,
}

impl CommandletRequest {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn project_root(&self) -> &std::path::Path {
        &self.project_root
    }

    pub fn mode(&self) -> AssetMigrationMode {
        self.mode
    }
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommandletReport {
    pub command: Option<String>,
    pub status: CommandletStatus,
    pub exit_code: CommandletExitCode,
    pub migration: Option<CommandletMigrationReport>,
    pub error: Option<String>,
}

impl CommandletReport {
    pub fn exit_code(&self) -> CommandletExitCode {
        self.exit_code
    }

    pub fn status(&self) -> CommandletStatus {
        self.status
    }

    pub fn migration(&self) -> Option<&CommandletMigrationReport> {
        self.migration.as_ref()
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
            error: Some(error.into()),
        }
    }

    fn missing_capabilities(command: impl Into<String>, capabilities: Vec<String>) -> Self {
        Self {
            command: Some(command.into()),
            status: CommandletStatus::MissingCapabilities,
            exit_code: CommandletExitCode::MissingCapability,
            migration: None,
            error: Some(format!(
                "commandlet requires unavailable capabilities: {}",
                capabilities.join(", ")
            )),
        }
    }

    fn succeeded(command: impl Into<String>, migration: CommandletMigrationReport) -> Self {
        Self {
            command: Some(command.into()),
            status: CommandletStatus::Succeeded,
            exit_code: CommandletExitCode::Success,
            migration: Some(migration),
            error: None,
        }
    }
}

/// Parse `--run migrate-assets --project <root> --dry-run|--apply` without creating an
/// application-local commandlet registry. Calls without `--run` remain available to the normal
/// editor startup parser.
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
    if command != MIGRATE_ASSETS_COMMANDLET {
        return Err(CommandletReport::invalid_arguments(
            Some(command),
            "unknown editor commandlet",
        ));
    }
    let Some(project_root) = project_root else {
        return Err(CommandletReport::invalid_arguments(
            Some(command),
            "migrate-assets requires --project",
        ));
    };
    let mode = match (dry_run, apply) {
        (true, false) => AssetMigrationMode::DryRun,
        (false, true) => AssetMigrationMode::Apply,
        (true, true) => {
            return Err(CommandletReport::invalid_arguments(
                Some(command),
                "--dry-run and --apply are mutually exclusive",
            ));
        }
        (false, false) => {
            return Err(CommandletReport::invalid_arguments(
                Some(command),
                "migrate-assets requires exactly one of --dry-run or --apply",
            ));
        }
    };
    Ok(Some(CommandletRequest {
        command,
        project_root,
        mode,
    }))
}

/// Run a commandlet with the capabilities provided by the headless editor profile.
pub fn run_commandlet(request: CommandletRequest) -> CommandletReport {
    run_commandlet_with_capabilities(request, [ASSET_MIGRATION_CAPABILITY])
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
    let registry = EditorCommandRegistry::default_workbench();
    let Some(descriptor) = registry.command(MIGRATE_ASSETS_OPERATION) else {
        return CommandletReport::failed(
            request.command,
            "migrate-assets is absent from the canonical editor command registry",
            None,
        );
    };
    if !descriptor.callable_from_remote() {
        return CommandletReport::failed(
            request.command,
            "migrate-assets is not callable from remote control",
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

    match migrate_project_assets(AssetMigrationOptions::new(
        request.project_root,
        request.mode,
    )) {
        Ok(report) if report.succeeded() => {
            CommandletReport::succeeded(MIGRATE_ASSETS_COMMANDLET, migration_report(&report))
        }
        Ok(report) => CommandletReport::failed(
            MIGRATE_ASSETS_COMMANDLET,
            "asset migration reported one or more issues",
            Some(migration_report(&report)),
        ),
        Err(error) => CommandletReport::failed(MIGRATE_ASSETS_COMMANDLET, error.to_string(), None),
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
