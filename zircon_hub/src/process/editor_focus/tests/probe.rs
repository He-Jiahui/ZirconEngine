use zircon_runtime_interface::project::session_lock::{
    ProjectSessionAdmissionLifecycleV1, ProjectSessionAdmissionRecordV1,
    ProjectSessionGenerationV1, ProjectSessionPrincipalV1,
};
use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

use super::{classify_live_session_record, ProjectEditorSessionProbe};

#[test]
fn live_activating_lease_is_pending_not_focusable() {
    let activating = fixture_claimed_record()
        .transition_to(ProjectSessionAdmissionLifecycleV1::PreflightApproved)
        .expect("fixture preflight approval")
        .transition_to(ProjectSessionAdmissionLifecycleV1::Activating)
        .expect("fixture activation");

    assert!(matches!(
        classify_live_session_record(activating),
        ProjectEditorSessionProbe::Pending(_)
    ));
}

#[test]
fn live_ready_lease_is_the_only_focusable_session() {
    let ready = fixture_claimed_record()
        .transition_to(ProjectSessionAdmissionLifecycleV1::PreflightApproved)
        .expect("fixture preflight approval")
        .transition_to(ProjectSessionAdmissionLifecycleV1::Activating)
        .expect("fixture activation")
        .commit_ready(ProjectSessionGenerationV1::new(1).expect("fixture generation"))
        .expect("fixture ready commit");

    assert!(matches!(
        classify_live_session_record(ready),
        ProjectEditorSessionProbe::Ready(_)
    ));
}

fn fixture_claimed_record() -> ProjectSessionAdmissionRecordV1 {
    let operation = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
        .allocate()
        .expect("fixture operation");
    ProjectSessionAdmissionRecordV1::claim(
        913,
        "913-1723718523000-1",
        ProjectSessionPrincipalV1::Hub,
        ZrRuntimeBuildSetId::parse(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .expect("fixture BuildSet"),
        operation,
        1_723_718_523_000,
    )
    .expect("fixture admission record")
}
