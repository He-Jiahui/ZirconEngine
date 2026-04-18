#[test]
fn scene_components_are_split_by_schedule_scene_viewport_render_and_gizmo_domains() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("components");

    for relative in [
        "mod.rs",
        "schedule.rs",
        "scene.rs",
        "viewport.rs",
        "render_extract.rs",
        "gizmo.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected scene component module {relative} under {:?}",
            root
        );
    }
}
