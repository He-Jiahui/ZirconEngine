use uuid::Uuid;

use crate::project::{
    ProjectActivationOperationId, ProjectActivationOperationSequence, ProjectLaunchInstanceId,
};
use crate::runtime_build_set::ZrRuntimeBuildSetId;

use super::{
    decode_project_session_admission_record, encode_project_session_admission_record,
    project_session_lock_path, ProjectSessionAdmissionLifecycleV1, ProjectSessionAdmissionRecordV1,
    ProjectSessionGenerationV1, ProjectSessionPrincipalV1,
};

#[test]
fn admission_record_round_trips_through_the_shared_strict_format() {
    let record = fixture_record();

    let encoded = encode_project_session_admission_record(&record);

    assert!(encoded.starts_with("version=2\n"));
    assert!(encoded.contains("principal=hub\n"));
    assert!(encoded.contains(
        "build_set_id=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n"
    ));
    assert!(encoded.contains("lifecycle=claimed\n"));
    assert!(encoded.contains("checked_epoch=1\n"));
    assert!(encoded.contains("session_generation=0\n"));
    assert_eq!(
        decode_project_session_admission_record(&encoded).expect("decode shared record"),
        record
    );
}

#[test]
fn admission_record_only_commits_a_generation_at_ready() {
    let claimed = fixture_record();
    let approved = claimed
        .transition_to(ProjectSessionAdmissionLifecycleV1::PreflightApproved)
        .expect("claimed session must record preflight approval");
    let activating = approved
        .transition_to(ProjectSessionAdmissionLifecycleV1::Activating)
        .expect("approved session must record activation");
    let ready = activating
        .commit_ready(ProjectSessionGenerationV1::new(7).expect("non-zero generation"))
        .expect("activating session must commit ready");

    assert_eq!(
        claimed.lifecycle(),
        ProjectSessionAdmissionLifecycleV1::Claimed
    );
    assert_eq!(claimed.session_generation(), None);
    assert_eq!(ready.lifecycle(), ProjectSessionAdmissionLifecycleV1::Ready);
    assert_eq!(ready.checked_epoch(), 4);
    assert_eq!(
        ready
            .session_generation()
            .map(ProjectSessionGenerationV1::get),
        Some(7)
    );
    assert!(ready
        .transition_to(ProjectSessionAdmissionLifecycleV1::Activating)
        .is_err());
}

#[test]
fn committed_session_close_is_persistently_non_ready_and_can_require_recovery() {
    let claimed = fixture_record();
    let approved = claimed
        .transition_to(ProjectSessionAdmissionLifecycleV1::PreflightApproved)
        .expect("claimed session must record preflight approval");
    let activating = approved
        .transition_to(ProjectSessionAdmissionLifecycleV1::Activating)
        .expect("approved session must record activation");
    let ready = activating
        .commit_ready(ProjectSessionGenerationV1::new(7).expect("non-zero generation"))
        .expect("activating session must commit ready");

    let closing = ready
        .transition_to(ProjectSessionAdmissionLifecycleV1::Closing)
        .expect("ready session must enter closing before teardown");
    let recovery = closing
        .transition_to(ProjectSessionAdmissionLifecycleV1::RecoveryRequired)
        .expect("failed close teardown must retain a recovery-required session");

    assert_eq!(
        closing.lifecycle(),
        ProjectSessionAdmissionLifecycleV1::Closing
    );
    assert_eq!(closing.checked_epoch(), ready.checked_epoch() + 1);
    assert_eq!(
        recovery.lifecycle(),
        ProjectSessionAdmissionLifecycleV1::RecoveryRequired
    );
    assert_eq!(recovery.checked_epoch(), closing.checked_epoch() + 1);
    assert_eq!(recovery.session_generation(), ready.session_generation());
}

#[test]
fn admission_record_rejects_old_unknown_or_unsafe_fields() {
    for source in [
        "version=1\nprocess_id=913\ninstance_id=913-1\nheartbeat_unix_millis=2\nlegacy_pid=1\n",
        "version=2\nprocess_id=913\ninstance_id=../unsafe\nprincipal=hub\nbuild_set_id=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\noperation_origin_instance=0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52\noperation_sequence=1\noperation_nonce=0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52\nlifecycle=claimed\nchecked_epoch=1\nsession_generation=0\nheartbeat_unix_millis=2\n",
        "version=2\nprocess_id=913\ninstance_id=913-1\nprincipal=hub\nbuild_set_id=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\noperation_origin_instance=0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52\noperation_sequence=1\noperation_nonce=0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52\nlifecycle=ready\nchecked_epoch=1\nsession_generation=0\nheartbeat_unix_millis=2\n",
    ] {
        assert!(
            decode_project_session_admission_record(source).is_err(),
            "must reject `{source}`"
        );
    }
}

#[test]
fn session_lock_path_stays_below_the_project_owned_zircon_directory() {
    assert_eq!(
        project_session_lock_path("E:/Projects/My Game"),
        std::path::PathBuf::from("E:/Projects/My Game/.zircon/session.lock")
    );
}

fn fixture_record() -> ProjectSessionAdmissionRecordV1 {
    let origin = ProjectLaunchInstanceId::try_from_uuid(
        Uuid::parse_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52").expect("fixture UUID"),
    )
    .expect("non-nil origin instance");
    let operation = ProjectActivationOperationId::try_from_parts(
        origin,
        ProjectActivationOperationSequence::new(1).expect("non-zero sequence"),
        Uuid::parse_str("e2ed5a8a-1df3-4e4e-b1ef-3058a5ed20af").expect("fixture UUID"),
    )
    .expect("non-nil operation nonce");
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
    .expect("valid admission record")
}
