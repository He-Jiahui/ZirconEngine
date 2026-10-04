use super::*;

#[test]
fn allocator_emits_the_last_valid_sequence_once_before_exhaustion() {
    let origin_instance = ProjectLaunchInstanceId::new();
    let generator = ProjectActivationOperationIdGenerator {
        origin_instance,
        next_sequence: AtomicU64::new(u64::MAX),
    };

    let final_operation = generator.allocate().expect("last sequence");

    assert_eq!(final_operation.origin_instance(), origin_instance);
    assert_eq!(final_operation.sequence().get(), u64::MAX);
    assert_eq!(generator.allocate(), None);
}
