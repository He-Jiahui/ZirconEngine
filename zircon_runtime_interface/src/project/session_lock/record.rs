use crate::project::{ProjectActivationOperationId, ProjectLaunchSource};
use crate::runtime_build_set::ZrRuntimeBuildSetId;

use super::ProjectSessionAdmissionRecordError;

/// Origin of a local desktop request. This is provenance, not an authentication claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectSessionPrincipalV1 {
    Application,
    Cli,
    Hub,
    Welcome,
    Recent,
}

impl ProjectSessionPrincipalV1 {
    pub const fn from_launch_source(source: ProjectLaunchSource) -> Self {
        match source {
            ProjectLaunchSource::Application => Self::Application,
            ProjectLaunchSource::Cli => Self::Cli,
            ProjectLaunchSource::Hub => Self::Hub,
            ProjectLaunchSource::Welcome => Self::Welcome,
            ProjectLaunchSource::Recent => Self::Recent,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Cli => "cli",
            Self::Hub => "hub",
            Self::Welcome => "welcome",
            Self::Recent => "recent",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ProjectSessionAdmissionRecordError> {
        match value {
            "application" => Ok(Self::Application),
            "cli" => Ok(Self::Cli),
            "hub" => Ok(Self::Hub),
            "welcome" => Ok(Self::Welcome),
            "recent" => Ok(Self::Recent),
            _ => Err(ProjectSessionAdmissionRecordError::new(
                "invalid or missing principal",
            )),
        }
    }
}

/// Persistent lifecycle state of an editor admission lease.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectSessionAdmissionLifecycleV1 {
    Claimed,
    PreflightApproved,
    Activating,
    Ready,
    Closing,
    RecoveryRequired,
}

impl ProjectSessionAdmissionLifecycleV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claimed => "claimed",
            Self::PreflightApproved => "preflight_approved",
            Self::Activating => "activating",
            Self::Ready => "ready",
            Self::Closing => "closing",
            Self::RecoveryRequired => "recovery_required",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ProjectSessionAdmissionRecordError> {
        match value {
            "claimed" => Ok(Self::Claimed),
            "preflight_approved" => Ok(Self::PreflightApproved),
            "activating" => Ok(Self::Activating),
            "ready" => Ok(Self::Ready),
            "closing" => Ok(Self::Closing),
            "recovery_required" => Ok(Self::RecoveryRequired),
            _ => Err(ProjectSessionAdmissionRecordError::new(
                "invalid or missing admission lifecycle",
            )),
        }
    }

    const fn allows_transition_to(self, next: Self) -> bool {
        match (self, next) {
            (Self::Claimed, Self::PreflightApproved | Self::Closing | Self::RecoveryRequired)
            | (
                Self::PreflightApproved,
                Self::Activating | Self::Closing | Self::RecoveryRequired,
            )
            | (Self::Activating, Self::Closing | Self::RecoveryRequired)
            | (Self::Ready, Self::Closing | Self::RecoveryRequired)
            | (Self::Closing, Self::RecoveryRequired) => true,
            _ => false,
        }
    }
}

/// A non-zero generation committed only after activation becomes ready.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectSessionGenerationV1(u64);

impl ProjectSessionGenerationV1 {
    pub const fn new(value: u64) -> Option<Self> {
        match value {
            0 => None,
            _ => Some(Self(value)),
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Persistent, cross-process state of the editor instance holding a project admission lease.
///
/// The record never substitutes for the operating-system lease. Consumers must require both the
/// OS lease and `Ready` lifecycle before treating a session as focusable or interactive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSessionAdmissionRecordV1 {
    process_id: u32,
    instance_id: String,
    principal: ProjectSessionPrincipalV1,
    build_set_id: ZrRuntimeBuildSetId,
    operation_id: ProjectActivationOperationId,
    lifecycle: ProjectSessionAdmissionLifecycleV1,
    checked_epoch: u64,
    session_generation: Option<ProjectSessionGenerationV1>,
    heartbeat_unix_millis: u64,
}

impl ProjectSessionAdmissionRecordV1 {
    pub fn claim(
        process_id: u32,
        instance_id: impl Into<String>,
        principal: ProjectSessionPrincipalV1,
        build_set_id: ZrRuntimeBuildSetId,
        operation_id: ProjectActivationOperationId,
        heartbeat_unix_millis: u64,
    ) -> Result<Self, ProjectSessionAdmissionRecordError> {
        Self::from_persisted(
            process_id,
            instance_id,
            principal,
            build_set_id,
            operation_id,
            ProjectSessionAdmissionLifecycleV1::Claimed,
            1,
            None,
            heartbeat_unix_millis,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_persisted(
        process_id: u32,
        instance_id: impl Into<String>,
        principal: ProjectSessionPrincipalV1,
        build_set_id: ZrRuntimeBuildSetId,
        operation_id: ProjectActivationOperationId,
        lifecycle: ProjectSessionAdmissionLifecycleV1,
        checked_epoch: u64,
        session_generation: Option<ProjectSessionGenerationV1>,
        heartbeat_unix_millis: u64,
    ) -> Result<Self, ProjectSessionAdmissionRecordError> {
        let instance_id = instance_id.into();
        validate_instance_id(&instance_id)?;
        if checked_epoch == 0 {
            return Err(ProjectSessionAdmissionRecordError::new(
                "checked_epoch must be non-zero",
            ));
        }
        if matches!(lifecycle, ProjectSessionAdmissionLifecycleV1::Ready)
            && session_generation.is_none()
        {
            return Err(ProjectSessionAdmissionRecordError::new(
                "ready admission record requires a committed session_generation",
            ));
        }
        if !matches!(
            lifecycle,
            ProjectSessionAdmissionLifecycleV1::Ready
                | ProjectSessionAdmissionLifecycleV1::Closing
                | ProjectSessionAdmissionLifecycleV1::RecoveryRequired
        ) && session_generation.is_some()
        {
            return Err(ProjectSessionAdmissionRecordError::new(
                "uncommitted admission lifecycle cannot carry a session_generation",
            ));
        }
        Ok(Self {
            process_id,
            instance_id,
            principal,
            build_set_id,
            operation_id,
            lifecycle,
            checked_epoch,
            session_generation,
            heartbeat_unix_millis,
        })
    }

    pub const fn process_id(&self) -> u32 {
        self.process_id
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub const fn principal(&self) -> ProjectSessionPrincipalV1 {
        self.principal
    }

    pub fn build_set_id(&self) -> &ZrRuntimeBuildSetId {
        &self.build_set_id
    }

    pub const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }

    pub const fn lifecycle(&self) -> ProjectSessionAdmissionLifecycleV1 {
        self.lifecycle
    }

    pub const fn checked_epoch(&self) -> u64 {
        self.checked_epoch
    }

    pub const fn session_generation(&self) -> Option<ProjectSessionGenerationV1> {
        self.session_generation
    }

    pub const fn heartbeat_unix_millis(&self) -> u64 {
        self.heartbeat_unix_millis
    }

    pub fn transition_to(
        &self,
        lifecycle: ProjectSessionAdmissionLifecycleV1,
    ) -> Result<Self, ProjectSessionAdmissionRecordError> {
        if matches!(lifecycle, ProjectSessionAdmissionLifecycleV1::Ready) {
            return Err(ProjectSessionAdmissionRecordError::new(
                "ready admission lifecycle requires an explicit generation commit",
            ));
        }
        if !self.lifecycle.allows_transition_to(lifecycle) {
            return Err(ProjectSessionAdmissionRecordError::new(format!(
                "invalid admission lifecycle transition {} -> {}",
                self.lifecycle.as_str(),
                lifecycle.as_str()
            )));
        }
        self.with_lifecycle(lifecycle, self.session_generation)
    }

    pub fn commit_ready(
        &self,
        generation: ProjectSessionGenerationV1,
    ) -> Result<Self, ProjectSessionAdmissionRecordError> {
        if self.lifecycle != ProjectSessionAdmissionLifecycleV1::Activating {
            return Err(ProjectSessionAdmissionRecordError::new(
                "only an activating admission record can commit ready",
            ));
        }
        self.with_lifecycle(ProjectSessionAdmissionLifecycleV1::Ready, Some(generation))
    }

    pub fn with_heartbeat_unix_millis(&self, heartbeat_unix_millis: u64) -> Self {
        Self {
            process_id: self.process_id,
            instance_id: self.instance_id.clone(),
            principal: self.principal,
            build_set_id: self.build_set_id.clone(),
            operation_id: self.operation_id,
            lifecycle: self.lifecycle,
            checked_epoch: self.checked_epoch,
            session_generation: self.session_generation,
            heartbeat_unix_millis,
        }
    }

    fn with_lifecycle(
        &self,
        lifecycle: ProjectSessionAdmissionLifecycleV1,
        session_generation: Option<ProjectSessionGenerationV1>,
    ) -> Result<Self, ProjectSessionAdmissionRecordError> {
        let checked_epoch = self
            .checked_epoch
            .checked_add(1)
            .ok_or_else(|| ProjectSessionAdmissionRecordError::new("checked_epoch overflow"))?;
        Self::from_persisted(
            self.process_id,
            self.instance_id.clone(),
            self.principal,
            self.build_set_id.clone(),
            self.operation_id,
            lifecycle,
            checked_epoch,
            session_generation,
            self.heartbeat_unix_millis,
        )
    }
}

pub(super) fn validate_instance_id(
    instance_id: &str,
) -> Result<(), ProjectSessionAdmissionRecordError> {
    if instance_id.is_empty()
        || !instance_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'-')
    {
        return Err(ProjectSessionAdmissionRecordError::new(
            "invalid or missing instance_id",
        ));
    }
    Ok(())
}
