#[test]
fn viewport_resize_patches_the_committed_projection_before_full_model_fallback() {
    let source = include_str!("../recompute_viewport.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("viewport recompute production source");
    let incremental = production
        .find("model.status_bar = StatusBarModel::from_chrome(chrome)")
        .expect("incremental viewport projection");
    let full = production
        .find("build_workbench_view_model")
        .expect("conservative model fallback");

    assert!(incremental < full);
}
