use zircon_runtime_interface::project::session_lock::ProjectSessionPrincipalV1;
use zircon_runtime_interface::project::{ProjectActivationOperationId, ProjectLaunchIntent};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

/// Immutable data authenticated before an editor process claims a project writer lease.
///
/// It deliberately contains no project manager, manifest, plugin, or host object. Those effects
/// begin only after this request has been persisted as an admission record.
#[derive(Clone, Debug)]
pub struct SessionAdmissionRequest {
    operation_id: ProjectActivationOperationId,
    principal: ProjectSessionPrincipalV1,
    build_set_id: ZrRuntimeBuildSetId,
}

impl SessionAdmissionRequest {
    pub fn new(
        operation_id: ProjectActivationOperationId,
        principal: ProjectSessionPrincipalV1,
        build_set_id: ZrRuntimeBuildSetId,
    ) -> Self {
        Self {
            operation_id,
            principal,
            build_set_id,
        }
    }

    pub fn from_launch_intent(
        intent: &ProjectLaunchIntent,
        build_set_id: ZrRuntimeBuildSetId,
    ) -> Self {
        Self::new(
            intent.operation_id(),
            ProjectSessionPrincipalV1::from_launch_source(intent.source()),
            build_set_id,
        )
    }

    pub const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }

    pub const fn principal(&self) -> ProjectSessionPrincipalV1 {
        self.principal
    }

    pub fn build_set_id(&self) -> &ZrRuntimeBuildSetId {
        &self.build_set_id
    }
}
