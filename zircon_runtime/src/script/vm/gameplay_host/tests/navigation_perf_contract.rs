use serde::Deserialize;

use super::NavMeshAgentDescriptor;

#[test]
fn borrowed_navigation_agent_deserialization_preserves_descriptor() {
    let expected = NavMeshAgentDescriptor::default();
    let value = serde_json::to_value(&expected).unwrap();

    let actual = NavMeshAgentDescriptor::deserialize(&value).unwrap();

    assert_eq!(serde_json::to_value(actual).unwrap(), value);
}
