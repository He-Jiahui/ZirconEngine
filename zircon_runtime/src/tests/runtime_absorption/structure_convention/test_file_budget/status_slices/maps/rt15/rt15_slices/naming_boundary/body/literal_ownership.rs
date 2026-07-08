use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_expected_slice_literals_are_child_owned() {
    let status_parent = read_runtime_src(STATUS_PARENT_PATH);
    let date_parent = read_runtime_src(DATE_PARENT_PATH);
    let status_child_sources = read_status_naming_boundary_sources();
    let date_child_sources = read_date_naming_boundary_sources();

    for moved_literal in [
        "Runtime 15 M2 core runtime state module naming hard cutover",
        "Runtime 15 M2 scene ECS observer callback registry module naming hard cutover",
        "Runtime 15 M2 render framework trait/construction owner naming hard cutover",
        "Runtime 15 M2 Net HTTP backend Hyper HTTP/1 client policy hard cutover",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "status naming-boundary parent should delegate `{moved_literal}`"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "date naming-boundary parent should delegate `{moved_literal}`"
        );
        assert!(
            status_child_sources.contains(moved_literal),
            "status naming-boundary children should own `{moved_literal}`"
        );
        assert!(
            date_child_sources.contains(moved_literal),
            "date naming-boundary children should own `{moved_literal}`"
        );
    }
}
