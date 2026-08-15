use super::sources::*;

#[test]
fn runtime_05_dynamic_scene_patch_preview_api_stays_read_only() {
    for public_export in [
        "ScenePatch",
        "ScenePatchPreviewComponentType",
        "ScenePatchPreviewEntityRemap",
        "ScenePatchPreviewReport",
        "ScenePatchPreviewResource",
    ] {
        assert!(
            DYNAMIC_SCENE_MOD_SOURCE.contains(public_export),
            "Runtime 05 patch preview export `{public_export}` must stay on the public dynamic-scene facade"
        );
    }
    assert!(
        PATCH_SOURCE.contains("pub struct ScenePatchPreviewReport")
            && PATCH_SOURCE.contains("pub struct ScenePatchPreviewEntityRemap")
            && PATCH_SOURCE.contains("pub source_entity: EntityId")
            && PATCH_SOURCE.contains("pub target_entity: EntityId")
            && PATCH_SOURCE.contains("pub entity_remaps: Vec<ScenePatchPreviewEntityRemap>")
            && PATCH_SOURCE.contains("pub fn has_entity_remaps(")
            && PATCH_SOURCE.contains("pub existing_component_type_count: usize")
            && PATCH_SOURCE.contains("pub new_component_type_count: usize")
            && PATCH_SOURCE.contains("pub struct ScenePatchPreviewComponentType")
            && PATCH_SOURCE.contains("pub component_types: Vec<ScenePatchPreviewComponentType>")
            && PATCH_SOURCE.contains("pub already_registered: bool")
            && PATCH_SOURCE.contains("pub struct ScenePatchPreviewResource")
            && PATCH_SOURCE.contains("pub resources: Vec<ScenePatchPreviewResource>")
            && PATCH_SOURCE.contains("pub already_present: bool")
            && PATCH_SOURCE.contains("pub can_create_on_apply: bool")
            && PATCH_SOURCE.contains("pub field_count: usize")
            && PATCH_SOURCE.contains("pub fn has_new_component_types(")
            && PATCH_SOURCE.contains("pub fn new_component_types(")
            && PATCH_SOURCE.contains("pub fn resources_requiring_creation(")
            && PATCH_SOURCE.contains("pub fn preview_apply(")
            && PATCH_SOURCE.contains("self.scene.preview_spawn_into(world)"),
        "ScenePatch must keep its preview API as a read-only DynamicScene facade call"
    );
    assert!(
        SCENE_MOD_SOURCE.contains("pub fn preview_spawn_into(")
            && SCENE_MOD_SOURCE.contains("spawn::preview_scene_spawn_into(self, world)"),
        "DynamicScene must keep preview_spawn_into routed to the read-only spawn preview helper"
    );

    let preview_body = SPAWN_SOURCE
        .split("pub(super) fn preview_scene_spawn_into")
        .nth(1)
        .expect("preview_scene_spawn_into should stay in the spawn transaction module")
        .split("fn install_component_type_descriptors")
        .next()
        .expect("install_component_type_descriptors should stay after preview helper");
    for forbidden_call in [
        "install_component_type_descriptors(",
        "insert_entity_records(",
        "apply_component_writes(",
        "apply_resource_writes(",
    ] {
        assert!(
            !preview_body.contains(forbidden_call),
            "preview_scene_spawn_into must not call mutating apply helper `{forbidden_call}`"
        );
    }
    for required_anchor in ["Ok(compile_scene_spawn(scene, world)?.into_preview())"] {
        assert!(
            preview_body.contains(required_anchor),
            "preview_scene_spawn_into should keep required read-only planning anchor `{required_anchor}`"
        );
    }
    for preflight_anchor in [
        "pub(crate) fn compile_scene_spawn(",
        "scene.ensure_supported()?",
        "ensure_component_type_descriptors_are_compatible(scene, world)?",
        "build_entity_remap(scene, world)?",
        "compile_entity_records(scene, world, &remap)?",
        "world.validate_owned_node_records(&records)?",
        "compile_component_writes(scene, world, &remap)?",
        "compile_resource_writes(scene, world, &remap)?",
        "build_preview_report(scene, world, &remap, resources)",
        "runtime_registration(&component.type_path)?",
        "runtime_registration(&resource.type_path)?",
        "ReflectError::NoComponentAdapter",
        "ReflectError::NoResourceAdapter",
        "ReflectError::MissingResource",
        "reflected_fields_to_json_object(&component.fields, remap)?",
        "remap_reflected_value(&field.value, remap)?",
        "component_type_count.saturating_sub(existing_component_type_count)",
        "component_type_descriptor(&descriptor.type_id)",
        "plugin_id: descriptor.plugin_id.clone()",
        "display_name: descriptor.display_name.clone()",
        "let already_present = adapter.contains(world)",
        "let can_create_on_apply = adapter.ensure.is_some()",
        "field_count: resource.fields.len()",
        "write.adapter.write_fields_by_slot(world, write.writes)?",
    ] {
        assert!(
            SPAWN_SOURCE.contains(preflight_anchor),
            "preview preflight should keep anchor `{preflight_anchor}`"
        );
    }
}
