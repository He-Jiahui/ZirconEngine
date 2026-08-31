use super::super::{
    assembly::{random_state_from_sequence, random_state_with_progress},
    RandomAlgorithmId, RandomSeedReceipt, RandomSequenceId, RandomServiceState, RandomState,
};

#[test]
fn random_state_serialization_preserves_the_stable_algorithm_id_and_validates_on_restore() {
    let state =
        RandomState::new(RandomAlgorithmId::Pcg32XshRrV1, 11, 5, 7).expect("valid PCG state");
    let encoded = serde_json::to_string(&state).expect("state should serialize");

    assert!(encoded.contains("\"algorithm\":1"));
    assert_eq!(
        serde_json::from_str::<RandomState>(&encoded).expect("valid state should deserialize"),
        state
    );
    assert!(serde_json::from_str::<RandomState>(
        "{\"algorithm\":1,\"state\":11,\"increment\":4,\"draw_index\":7}"
    )
    .is_err());
    assert!(serde_json::from_str::<RandomState>(
        "{\"algorithm\":2,\"state\":11,\"increment\":5,\"draw_index\":7}"
    )
    .is_err());
}

#[test]
fn typed_sequence_assembly_and_progress_preserve_the_increment_invariant() {
    let sequence = RandomSequenceId::new(RandomSequenceId::MAX_VALUE)
        .expect("the maximum PCG sequence id should be valid");
    let state = random_state_from_sequence(RandomAlgorithmId::Pcg32XshRrV1, 11, sequence, 7);
    let progressed = random_state_with_progress(state, 19, 8);

    assert_eq!(state.increment(), u64::MAX);
    assert_eq!(state.sequence_id(), sequence);
    assert_eq!(progressed.algorithm(), state.algorithm());
    assert_eq!(progressed.increment(), state.increment());
    assert_eq!(progressed.sequence_id(), sequence);
    assert_eq!(progressed.generator_state(), 19);
    assert_eq!(progressed.draw_index(), 8);
}

#[test]
fn service_state_serialization_preserves_algorithm_and_seed_generation() {
    let snapshot = RandomServiceState::new(RandomAlgorithmId::Pcg32XshRrV1, 0x6688, 7);
    let encoded = serde_json::to_string(&snapshot).expect("service state should serialize");
    let restored: RandomServiceState =
        serde_json::from_str(&encoded).expect("valid service state should deserialize");

    assert_eq!(restored, snapshot);
    assert_eq!(restored.algorithm(), RandomAlgorithmId::Pcg32XshRrV1);
    assert_eq!(restored.master_seed(), 0x6688);
    assert_eq!(restored.master_seed_generation(), 7);
}

#[test]
fn seed_receipt_requires_a_single_successor_generation() {
    assert_eq!(
        RandomSeedReceipt::try_new(0x11, 0x22, 7, 9),
        Err(
            super::super::RandomSeedReceiptError::NonSuccessorGeneration {
                previous_generation: 7,
                generation: 9,
            }
        )
    );
    let receipt = RandomSeedReceipt::try_new(0x11, 0x22, 7, 8).expect("successor generation");
    assert_eq!(receipt.generation(), 8);

    let encoded = serde_json::to_string(&receipt).expect("seed receipt should serialize");
    assert_eq!(
        serde_json::from_str::<RandomSeedReceipt>(&encoded)
            .expect("valid seed receipt should deserialize"),
        receipt
    );
    assert!(serde_json::from_str::<RandomSeedReceipt>(
        "{\"previous_seed\":17,\"seed\":34,\"previous_generation\":7,\"generation\":9}"
    )
    .is_err());
}
