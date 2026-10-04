const SOURCE: &str = include_str!("../scan_and_import.rs");

#[test]
fn project_import_collects_ibl_writes_without_calling_a_stage_entry_point() {
    let preparation = SOURCE
        .split("fn prepare_environment_ibl_import(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn append_shader_import_path_conflict_diagnostics(")
                .next()
        })
        .expect("project IBL preparation helper must exist");

    assert!(preparation.contains("prepare_environment_ibl_source"));
    assert!(preparation.contains("prepare_source_cubemap_texture"));
    assert!(preparation.contains("into_file_writes"));
    assert!(!preparation.contains("stage_environment_ibl_source("));
    assert!(!preparation.contains("stage_source_cubemap_texture("));
}
