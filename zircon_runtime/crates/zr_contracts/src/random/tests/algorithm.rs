use super::super::{
    RandomAlgorithmId, RandomAlgorithmIdError, RandomSequenceId, RandomSequenceIdError,
};

#[test]
fn algorithm_ids_have_a_fail_closed_stable_persistence_mapping() {
    assert_eq!(RandomAlgorithmId::Pcg32XshRrV1.stable_id(), 1);
    assert_eq!(
        RandomAlgorithmId::from_stable_id(1),
        Ok(RandomAlgorithmId::Pcg32XshRrV1)
    );
    assert_eq!(
        RandomAlgorithmId::from_stable_id(2),
        Err(RandomAlgorithmIdError::UnsupportedStableId { value: 2 })
    );
}

#[test]
fn pcg32_sequence_ids_reject_values_outside_the_63_bit_stream_space() {
    let maximum = RandomSequenceId::new(RandomSequenceId::MAX_VALUE)
        .expect("the maximum 63-bit sequence id should be valid");

    assert_eq!(maximum.value(), RandomSequenceId::MAX_VALUE);
    assert_eq!(
        RandomSequenceId::new(RandomSequenceId::MAX_VALUE + 1),
        Err(RandomSequenceIdError::OutOfRange {
            value: RandomSequenceId::MAX_VALUE + 1,
        })
    );

    let encoded = serde_json::to_string(&maximum).expect("sequence id should serialize");
    assert_eq!(
        serde_json::from_str::<RandomSequenceId>(&encoded)
            .expect("valid sequence id should deserialize"),
        maximum
    );
    assert!(serde_json::from_str::<RandomSequenceId>("9223372036854775808").is_err());
}
