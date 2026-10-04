#[test]
fn runtime_library_template_does_not_rescan_the_completed_source() {
    let source = include_str!("../platform_host_files.rs");
    let replacement_pass = ["    .rep", "lace("].concat();
    let template_body = source
        .split("fn runtime_library_template")
        .nth(1)
        .and_then(|body| body.split("fn native_library_stem").next())
        .expect("runtime library template body should remain available");

    assert!(!template_body.contains(&replacement_pass));
}
