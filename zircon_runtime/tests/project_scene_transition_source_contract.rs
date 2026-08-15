const HOST_REQUESTS: &str =
    include_str!("../../zircon_runtime_interface/src/runtime_api/host_requests.rs");
const GAMEPLAY_HOST: &str = include_str!("../src/script/vm/gameplay_host.rs");
const SCENE_TRANSITION: &str = include_str!("../src/script/vm/gameplay_host/scene_transition.rs");

#[test]
fn project_scene_transition_request_is_versioned_and_uri_scoped() {
    for required in [
        "pub struct ZrRuntimeProjectSceneTransitionRequestV1",
        "pub request_id: u64",
        "pub scene_uri: String",
        "pub policy: ZrRuntimeProjectSceneTransitionPolicyV1",
        "pub enum ZrRuntimeProjectSceneTransitionPolicyV1",
        "pub fn try_new(",
        "validate_project_scene_uri",
        "strip_prefix(\"res://\")",
    ] {
        assert!(
            HOST_REQUESTS.contains(required),
            "scene-transition ABI contract is missing {required}"
        );
    }

    for forbidden in ["std::path::Path", "canonicalize(", "to_absolute"] {
        assert!(
            !HOST_REQUESTS.contains(forbidden),
            "ABI scene URI validation must not expose host paths through {forbidden}"
        );
    }
}

#[test]
fn gameplay_scene_transition_submission_is_one_slot_and_generation_ordered() {
    for required in [
        "mod scene_transition;",
        ".with_capability(\"gameplay.scene_transition\")",
        "function(\"request_scene_transition\"",
        "HostExportFunction::new(\"request_scene_transition\", request_scene_transition)",
    ] {
        assert!(
            GAMEPLAY_HOST.contains(required),
            "gameplay host is missing {required}"
        );
    }

    for required in [
        "zircon_runtime_interface::runtime_api",
        "use crate::core::framework::scene::SceneResource;",
        "impl SceneResource for ProjectSceneTransitionSequence {}",
        "impl SceneResource for ZrRuntimeProjectSceneTransitionRequestV1 {}",
        "ZrRuntimeProjectSceneTransitionRequestV1::try_new",
        "world.world_generation().saturating_add(1)",
        "world.insert_resource(request)",
        "ScriptHostValue::Int(request_id as i64)",
    ] {
        assert!(
            SCENE_TRANSITION.contains(required),
            "scene-transition producer is missing {required}"
        );
    }

    for forbidden in ["VecDeque", "set_dynamic_component", "node_records()"] {
        assert!(
            !SCENE_TRANSITION.contains(forbidden),
            "scene-transition admission must stay one-slot and typed, found {forbidden}"
        );
    }
}
