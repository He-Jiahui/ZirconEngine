use super::*;

#[test]
fn aggregate_wire_length_rejects_limit_excess_and_saturating_overflow() {
    assert_eq!(checked_wire_length("fixture", 4, [3, 2], 9), Ok(9));
    assert_eq!(
        checked_wire_length("fixture", 4, [3, 3], 9),
        Err(ProtocolError::CollectionTooLarge {
            context: "fixture",
            actual: 10,
            maximum: 9,
        })
    );
    assert!(matches!(
        checked_wire_length("fixture", usize::MAX, [1], u64::MAX),
        Err(ProtocolError::CollectionTooLarge { actual, .. }) if actual == usize::MAX
    ));
}
