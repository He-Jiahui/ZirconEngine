use super::super::sources::GuardSources;
use super::super::*;

pub(super) fn assert_ui_children(sources: &GuardSources) {
    assert_contains_all(
        "UI shared core test-budget parent mounts shared-core child owners",
        &sources.ui_shared_core,
        &[
            "use super::*;",
            "#[path = \"ui_shared_core/input_visibility.rs\"]",
            "mod input_visibility;",
            "#[path = \"ui_shared_core/layout_surface.rs\"]",
            "mod layout_surface;",
            "#[path = \"ui_shared_core/root.rs\"]",
            "mod root;",
            "#[path = \"ui_shared_core/scroll_mutation.rs\"]",
            "mod scroll_mutation;",
            "runtime_15_ui_shared_core_guard_child_owners_are_folder_backed",
        ],
    );
    assert_contains_all(
        "UI v2 asset test-budget child owns historical v2-asset guard",
        &sources.ui_v2_asset,
        &[
            "use super::*;",
            "fn runtime_15_ui_v2_asset_tests_are_folder_backed",
        ],
    );
}
