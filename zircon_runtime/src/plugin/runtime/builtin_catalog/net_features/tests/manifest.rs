use super::join_string_parts;

#[test]
fn exact_net_identifier_join_preserves_feature_and_runtime_ids() {
    let feature_id = join_string_parts(&["net.", "reliable_udp"]);
    assert_eq!(feature_id, "net.reliable_udp");
    assert_eq!(
        join_string_parts(&[&feature_id, ".runtime"]),
        "net.reliable_udp.runtime"
    );
}
