use super::*;

#[test]
fn scene_project_serialization_sources_do_not_store_editor_authoring_state() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    for relative in [
        "src/scene/world/world.rs",
        "src/scene/world/project_io.rs",
        "src/scene/world/project_io/camera.rs",
        "src/scene/world/project_io/document.rs",
        "src/scene/world/project_io/physics.rs",
        "src/scene/world/project_io/post_process.rs",
        "src/scene/world/project_io/references.rs",
        "src/scene/world/project_io/scene_asset.rs",
        "src/scene/world/project_io/script.rs",
        "src/scene/world/project_io/transform.rs",
        "src/scene/dynamic/document/mod.rs",
        "src/scene/dynamic/document/read.rs",
        "src/scene/dynamic/document/migration/mod.rs",
        "src/scene/dynamic/document/migration/project_world.rs",
        "src/scene/dynamic/document/write.rs",
        "src/scene/dynamic/entity/dynamic_component.rs",
        "src/scene/dynamic/entity/dynamic_entity.rs",
        "src/scene/dynamic/entity/dynamic_resource.rs",
        "src/scene/dynamic/entity/mod.rs",
        "src/scene/dynamic/scene/capture.rs",
        "src/scene/dynamic/scene/mod.rs",
        "src/scene/dynamic/scene/spawn/mod.rs",
        "src/scene/dynamic/scene/spawn/preview.rs",
        "src/scene/dynamic/scene/spawn/resource.rs",
        "src/scene/dynamic/scene/spawn/transaction.rs",
        "src/scene/dynamic/scene/validation.rs",
        "src/scene/dynamic/value/json.rs",
        "src/scene/dynamic/value/mod.rs",
        "src/scene/dynamic/value/remap.rs",
        "src/asset/assets/scene/mod.rs",
    ] {
        let path = manifest_root.join(relative);
        let source = std::fs::read_to_string(&path).unwrap();

        assert_text_excludes_authoring_tokens(
            &format!("runtime scene project serialization source {relative}"),
            &source,
            SOURCE_AUTHORING_TOKENS,
        );
    }
}
