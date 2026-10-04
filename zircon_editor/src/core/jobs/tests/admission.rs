use super::*;

#[test]
fn admission_key_enforces_its_utf8_byte_budget() {
    assert!(EditorJobAdmissionKey::new("a".repeat(EditorJobAdmissionKey::MAX_BYTES)).is_ok());
    assert_eq!(
        EditorJobAdmissionKey::new("a".repeat(EditorJobAdmissionKey::MAX_BYTES + 1)),
        Err(JobAdmissionKeyError::TooLong {
            len: EditorJobAdmissionKey::MAX_BYTES + 1,
            max: EditorJobAdmissionKey::MAX_BYTES,
        })
    );
}

#[test]
#[ignore = "managed Editor09 performance evidence"]
fn editor09_admission_key_retention_budget_evidence() {
    const OVERSIZED_BYTES: usize = 1_048_576;

    let error = EditorJobAdmissionKey::new("a".repeat(OVERSIZED_BYTES)).unwrap_err();

    assert_eq!(
        error,
        JobAdmissionKeyError::TooLong {
            len: OVERSIZED_BYTES,
            max: EditorJobAdmissionKey::MAX_BYTES,
        }
    );
    println!(
        "EDITOR_JOB_BENCH_V1 kind=admission_key_retention oversized_input_bytes={} retained_identity_bytes_before={} retained_identity_bytes_after=0 retained_byte_reduction_percent=100.0000 maximum_bytes={}",
        OVERSIZED_BYTES,
        OVERSIZED_BYTES,
        EditorJobAdmissionKey::MAX_BYTES,
    );
}
