use std::path::PathBuf;

#[test]
fn animation_evaluate_is_owned_by_the_folder_backed_evaluation_pipeline() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let runtime_system = std::fs::read_to_string(crate_root.join("src/runtime_system.rs")).unwrap();
    let pipeline_facade =
        std::fs::read_to_string(crate_root.join("src/evaluation/pipeline/mod.rs")).unwrap();

    assert!(runtime_system.contains("crate::evaluation::tick_animation_world"));
    assert!(!runtime_system.contains("#[path = \"scene_hook/"));
    assert!(!crate_root.join("src/scene_hook").exists());
    for owner in [
        "parameter_apply",
        "state_machine_step",
        "graph_evaluate",
        "pose_blend",
        "pose_apply",
    ] {
        assert!(
            pipeline_facade.contains(&format!("mod {owner};")),
            "missing pipeline phase owner {owner}"
        );
    }
}

#[test]
fn pipeline_facades_are_structural_and_do_not_hide_tick_behavior() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for relative in ["src/evaluation/mod.rs", "src/evaluation/pipeline/mod.rs"] {
        let source = std::fs::read_to_string(crate_root.join(relative)).unwrap();
        assert!(!source.contains("pub fn tick_animation_world("));
        assert!(!source.contains("fn sample_state_transition_pose("));
        assert!(
            source.lines().count() < 80,
            "{relative} must stay structural"
        );
    }
}
