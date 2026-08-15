use std::path::Path;

use super::inventory::{
    GAMEPLAY_HOST_MODULE_ANCHORS, GAMEPLAY_HOST_OWNER_FILES, GAMEPLAY_HOST_OWNER_MAX_LINES,
    GAMEPLAY_HOST_REGISTRATION_ANCHORS, GAMEPLAY_VALUE_ANCHORS,
};
use super::support::{assert_file_line_budget, assert_files_exist};

#[test]
fn runtime_13_gameplay_host_owner_split_keeps_domain_files() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_files_exist(
        runtime_root,
        GAMEPLAY_HOST_OWNER_FILES,
        "Runtime 13 gameplay host owner",
    );
    for file in GAMEPLAY_HOST_OWNER_FILES {
        assert_file_line_budget(
            runtime_root,
            file,
            GAMEPLAY_HOST_OWNER_MAX_LINES,
            "Runtime 13 gameplay host owner",
        );
    }

    let gameplay_host = include_str!("../../../script/vm/gameplay_host.rs");
    for module_anchor in GAMEPLAY_HOST_MODULE_ANCHORS {
        assert!(
            gameplay_host.contains(module_anchor),
            "gameplay_host.rs should keep domain owner module `{module_anchor}`"
        );
    }
    for registration_anchor in GAMEPLAY_HOST_REGISTRATION_ANCHORS {
        assert!(
            gameplay_host.contains(registration_anchor),
            "gameplay_host.rs should keep callback registration anchor `{registration_anchor}`"
        );
    }

    let values = include_str!("../../../script/vm/gameplay_host/values.rs");
    for value_anchor in GAMEPLAY_VALUE_ANCHORS {
        assert!(
            values.contains(value_anchor),
            "gameplay_host/values.rs should own shared value helper `{value_anchor}`"
        );
    }
    let transform = include_str!("../../../script/vm/gameplay_host/transform.rs");
    assert!(
        transform.contains("navigation_next_point"),
        "gameplay_host/transform.rs should keep navigation-aware movement dependency explicit"
    );
    let navigation = include_str!("../../../script/vm/gameplay_host/navigation.rs");
    assert!(
        navigation.contains("NavMeshAgentDescriptor"),
        "gameplay_host/navigation.rs should own nav-agent mutation"
    );
    let scene_transition = include_str!("../../../script/vm/gameplay_host/scene_transition.rs");
    for anchor in [
        "ZrRuntimeProjectSceneTransitionRequestV1::try_new",
        "ZrRuntimeProjectSceneTransitionPolicyV1::ReplaceActive",
        "world.insert_resource(request)",
    ] {
        assert!(
            scene_transition.contains(anchor),
            "gameplay_host/scene_transition.rs should retain transition request anchor `{anchor}`"
        );
    }
}
