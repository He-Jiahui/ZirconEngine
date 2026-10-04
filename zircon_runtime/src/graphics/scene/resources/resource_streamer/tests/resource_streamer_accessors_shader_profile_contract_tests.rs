#[test]
fn module_include_resolution_profiles_count_and_source_bytes() {
    let source = include_str!("../resource_streamer_accessors.rs")
        .split("pub(crate) fn shader_module_include_sources")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn required_plugin_shader_module_tokens")
                .next()
        })
        .expect("module include resolution function");

    assert!(source.contains("\"shader_pipeline\", \"module_include_resolution\""));
    assert!(source.contains("shader_module_include_count"));
    assert!(source.contains("shader_module_include_source_bytes"));
}
