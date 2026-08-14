use super::{assert_contains_all, runtime_src_path};

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

#[test]
fn runtime_15_animation_manager_is_folder_backed() {
    let old_manager = runtime_src_path("animation/manager.rs");
    let animation_mod = read_runtime_src("animation/mod.rs");
    let manager_mod = read_runtime_src("animation/manager/mod.rs");
    let graph = read_runtime_src("animation/manager/graph.rs");
    let parameters = read_runtime_src("animation/manager/parameters.rs");
    let pose = read_runtime_src("animation/manager/pose.rs");
    let sampling = read_runtime_src("animation/manager/sampling.rs");
    let state_machine = read_runtime_src("animation/manager/state_machine.rs");
    assert!(
        !old_manager.exists(),
        "animation manager root should live at animation/manager/mod.rs, not the retired flat animation/manager.rs"
    );
    assert_contains_all(
        "animation root mounts manager folder",
        &animation_mod,
        &["mod manager;", "pub use manager::DefaultAnimationManager;"],
    );
    assert_contains_all(
        "animation manager folder root owns manager facade and child mounts",
        &manager_mod,
        &[
            "mod graph;",
            "mod parameters;",
            "mod pose;",
            "mod sampling;",
            "mod state_machine;",
            "pub struct DefaultAnimationManager",
            "impl AnimationManager for DefaultAnimationManager",
        ],
    );
    assert_contains_all(
        "animation manager children retain behavior owners",
        &format!("{graph}\n{parameters}\n{pose}\n{sampling}\n{state_machine}"),
        &[
            "fn evaluate_graph",
            "fn parameter_defaults",
            "fn sample_clip_pose",
            "fn resolve_sample_time",
            "fn sample_vec3",
            "fn evaluate_state_machine",
        ],
    );

    for (path, source) in [
        ("animation/mod.rs", animation_mod.as_str()),
        ("animation/manager/mod.rs", manager_mod.as_str()),
        ("animation/manager/graph.rs", graph.as_str()),
        ("animation/manager/parameters.rs", parameters.as_str()),
        ("animation/manager/pose.rs", pose.as_str()),
        ("animation/manager/sampling.rs", sampling.as_str()),
        ("animation/manager/state_machine.rs", state_machine.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production owner budget; got {line_count} lines"
        );
    }
}
