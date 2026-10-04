#[test]
fn visible_spatial_snapshot_query_releases_framework_state_after_copying_snapshot_handle() {
    let source = include_str!("../query_visible_spatial_snapshot.rs");
    let release = source
        .find("drop(state);")
        .expect("query releases framework state");
    let return_result = source.find("Ok(snapshot)").expect("query returns snapshot");

    assert!(release < return_result);
}
