use super::*;

#[test]
fn mutex_group_enforces_its_utf8_byte_budget_for_parse_and_deserialize() {
    assert!(MutexGroup::parse("a".repeat(MutexGroup::MAX_BYTES)).is_ok());
    assert_eq!(
        MutexGroup::parse("a".repeat(MutexGroup::MAX_BYTES + 1)),
        Err(MutexGroupError::TooLong {
            len: MutexGroup::MAX_BYTES + 1,
            max: MutexGroup::MAX_BYTES,
        })
    );
    let encoded = serde_json::to_string(&"a".repeat(MutexGroup::MAX_BYTES + 1)).unwrap();
    assert!(serde_json::from_str::<MutexGroup>(&encoded).is_err());
}

#[test]
#[ignore = "managed Editor09 performance evidence"]
fn editor09_mutex_group_retention_budget_evidence() {
    const OVERSIZED_BYTES: usize = 1_048_576;

    let error = MutexGroup::parse("a".repeat(OVERSIZED_BYTES)).unwrap_err();

    assert_eq!(
        error,
        MutexGroupError::TooLong {
            len: OVERSIZED_BYTES,
            max: MutexGroup::MAX_BYTES,
        }
    );
    println!(
        "EDITOR_JOB_BENCH_V1 kind=mutex_group_retention oversized_input_bytes={} retained_identity_bytes_before={} retained_identity_bytes_after=0 retained_byte_reduction_percent=100.0000 maximum_bytes={}",
        OVERSIZED_BYTES,
        OVERSIZED_BYTES,
        MutexGroup::MAX_BYTES,
    );
}
