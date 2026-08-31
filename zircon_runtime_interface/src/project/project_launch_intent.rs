use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};

use super::{
    validate_project_name, ProjectActivationOperationId, ProjectLaunchIntentError,
    ProjectTemplateId,
};

/// The only currently accepted wire revision for a requested project launch.
pub const PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1: u32 = 1;

/// Selects the project-derived capabilities that a later preflight composition may admit.
///
/// This is requested intent, not approval. The preflight receipt is the first authority allowed
/// to turn it into a concrete composition plan.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectLaunchProfile {
    #[default]
    Normal,
    Safe,
    Recovery,
}

/// The product surface that originated a launch operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectLaunchSource {
    Application,
    Cli,
    Hub,
    Welcome,
    Recent,
}

/// A preflight input, deliberately separate from a canonical `ProjectIdentity`.
///
/// A requested path is user-controlled input. Only data-only preflight can resolve it and attach
/// the persisted project GUID plus manifest digest required for canonical identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", deny_unknown_fields)]
pub enum ProjectLaunchTarget {
    OpenExisting {
        requested_path: PathBuf,
    },
    CreateProject {
        project_name: String,
        location: PathBuf,
        template: ProjectTemplateId,
    },
}

/// Versioned, idempotency-addressable project lifecycle input shared by Hub, App, and Editor.
///
/// It represents a request only. It contains neither `ProjectIdentity` nor an activation permit,
/// and cannot be used as evidence that project-derived code is safe to execute.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProjectLaunchIntent {
    schema_version: u32,
    operation_id: ProjectActivationOperationId,
    source: ProjectLaunchSource,
    profile: ProjectLaunchProfile,
    target: ProjectLaunchTarget,
}

impl ProjectLaunchIntent {
    pub fn open_existing(
        operation_id: ProjectActivationOperationId,
        source: ProjectLaunchSource,
        profile: ProjectLaunchProfile,
        requested_path: impl Into<PathBuf>,
    ) -> Result<Self, ProjectLaunchIntentError> {
        Self::new(
            operation_id,
            source,
            profile,
            ProjectLaunchTarget::OpenExisting {
                requested_path: requested_path.into(),
            },
        )
    }

    pub fn create_project(
        operation_id: ProjectActivationOperationId,
        source: ProjectLaunchSource,
        profile: ProjectLaunchProfile,
        project_name: impl Into<String>,
        location: impl Into<PathBuf>,
        template: ProjectTemplateId,
    ) -> Result<Self, ProjectLaunchIntentError> {
        Self::new(
            operation_id,
            source,
            profile,
            ProjectLaunchTarget::CreateProject {
                project_name: project_name.into(),
                location: location.into(),
                template,
            },
        )
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }

    pub const fn source(&self) -> ProjectLaunchSource {
        self.source
    }

    pub const fn profile(&self) -> ProjectLaunchProfile {
        self.profile
    }

    pub fn target(&self) -> &ProjectLaunchTarget {
        &self.target
    }

    /// Keeps the initiating operation and policy when an input has become an existing root.
    pub fn retarget_open_existing_project(
        &self,
        requested_path: impl Into<PathBuf>,
    ) -> Result<Self, ProjectLaunchIntentError> {
        Self::open_existing(self.operation_id, self.source, self.profile, requested_path)
    }

    fn new(
        operation_id: ProjectActivationOperationId,
        source: ProjectLaunchSource,
        profile: ProjectLaunchProfile,
        target: ProjectLaunchTarget,
    ) -> Result<Self, ProjectLaunchIntentError> {
        validate_target(&target)?;
        Ok(Self {
            schema_version: PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1,
            operation_id,
            source,
            profile,
            target,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectLaunchIntentWire {
    schema_version: u32,
    operation_id: ProjectActivationOperationId,
    source: ProjectLaunchSource,
    profile: ProjectLaunchProfile,
    target: ProjectLaunchTarget,
}

impl<'de> Deserialize<'de> for ProjectLaunchIntent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProjectLaunchIntentWire::deserialize(deserializer)?;
        if wire.schema_version != PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1 {
            return Err(serde::de::Error::custom(
                ProjectLaunchIntentError::UnsupportedSchemaVersion {
                    expected: PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1,
                    actual: wire.schema_version,
                },
            ));
        }
        Self::new(wire.operation_id, wire.source, wire.profile, wire.target)
            .map_err(serde::de::Error::custom)
    }
}

fn validate_target(target: &ProjectLaunchTarget) -> Result<(), ProjectLaunchIntentError> {
    match target {
        ProjectLaunchTarget::OpenExisting { requested_path } => {
            validate_path(requested_path, ProjectLaunchIntentError::EmptyOpenPath)
        }
        ProjectLaunchTarget::CreateProject {
            project_name,
            location,
            ..
        } => {
            validate_project_name(project_name)?;
            validate_path(location, ProjectLaunchIntentError::EmptyCreateLocation)
        }
    }
}

fn validate_path(
    path: &PathBuf,
    empty_error: ProjectLaunchIntentError,
) -> Result<(), ProjectLaunchIntentError> {
    if path.as_os_str().is_empty() {
        return Err(empty_error);
    }
    let Some(value) = path.to_str() else {
        return Err(ProjectLaunchIntentError::NonTextPath { path: path.clone() });
    };
    if value.trim().is_empty() {
        return Err(empty_error);
    }
    Ok(())
}
