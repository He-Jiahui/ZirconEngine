#[test]
fn runtime_scene_property_reflection_stays_internal() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let framework_scene_mod_source =
        std::fs::read_to_string(runtime_root.join("src/core/framework/scene/mod.rs"))
            .unwrap_or_default();
    let property_value_source =
        std::fs::read_to_string(runtime_root.join("src/core/framework/scene/property_value.rs"))
            .unwrap_or_default();
    let property_access_entries_source =
        std::fs::read_to_string(runtime_root.join("src/scene/world/property_access/entries.rs"))
            .unwrap_or_default();

    assert!(
        framework_scene_mod_source.contains("pub use property_value::ScenePropertyValue;"),
        "runtime framework scene surface should keep ScenePropertyValue public for animation/runtime mutation paths"
    );
    assert!(
        framework_scene_mod_source.contains("pub(crate) use property_value::ScenePropertyEntry;"),
        "runtime framework scene surface should keep ScenePropertyEntry crate-private"
    );
    assert!(
        !framework_scene_mod_source
            .contains("pub use property_value::{ScenePropertyEntry, ScenePropertyValue};")
            && !framework_scene_mod_source.contains("pub use property_value::ScenePropertyEntry;"),
        "runtime framework scene surface should stop publicly exporting ScenePropertyEntry"
    );
    assert!(
        property_value_source.contains("pub(crate) struct ScenePropertyEntry"),
        "scene property entry model should be crate-private"
    );
    assert!(
        !property_value_source.contains("pub struct ScenePropertyEntry"),
        "scene property entry model should no longer be public"
    );
    assert!(
        runtime_root
            .join("src/scene/world/property_access/mod.rs")
            .exists()
            && !runtime_root
                .join("src/scene/world/property_access.rs")
                .exists(),
        "world property reflection should stay in the folder-backed property_access subtree"
    );
    assert!(
        property_access_entries_source.contains("pub(super) fn property_entries("),
        "world property reflection listing should stay internal to the property_access subtree"
    );
    assert!(
        !property_access_entries_source.contains("pub fn property_entries(")
            && !property_access_entries_source.contains("pub(crate) fn property_entries("),
        "world property reflection listing should stay internal instead of leaking as runtime public API"
    );
}
