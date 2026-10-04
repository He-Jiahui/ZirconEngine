use super::super::{
    RandomAlgorithmId, RandomAlgorithmIdError, RandomSequenceId, RandomSequenceIdError,
};

// 稳定算法 ID 直接进入持久化数据；未知数值必须拒绝，不能静默降级到当前实现。
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

// PCG 增量最低位保留给奇数约束，因此序列 ID 只占 63 位，序列化边界也必须拒绝高位值。
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
