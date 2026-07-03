#[test]
fn review_f5_dynamic_component_errors_preserve_scene_error_sources() {
    let world_error = include_str!("../../../../../scene/world/error.rs");
    let dynamic_components = include_str!("../../../../../scene/world/dynamic_components.rs");
    let component_type_registry =
        include_str!("../../../../../scene/world/component_type_registry.rs");
    let dynamic_scene_error = include_str!("../../../../../scene/dynamic_scene/error.rs");
    let dynamic_scene_spawn = include_str!("../../../../../scene/dynamic_scene/scene/spawn.rs");
    let reflect_dynamic_component =
        include_str!("../../../../../scene/reflect/dynamic_component.rs");
    let plugin_component_registry =
        include_str!("../../../../../plugin/extension_registry/apply_to_world/component.rs");
    let ecs_typed_tests = include_str!("../../../../../scene/tests/ecs_typed_api.rs");
    let dynamic_scene_tests = include_str!("../../../../../scene/tests/dynamic_scene.rs");
    let dynamic_scene_patch_tests =
        include_str!("../../../../../scene/tests/dynamic_scene/scene_patch_document.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_08_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../../../../docs/zircon_runtime/scene/ecs.md");

    for required in [
        "Reflect(#[from] ReflectError)",
        "ComponentTypePluginPrefixMismatch",
        "DuplicateComponentType",
        "UnregisteredDynamicComponentType",
        "PluginComponentsActive",
        "UnknownDynamicComponentProperty",
        "NonEditableDynamicComponentProperty",
    ] {
        assert!(
            world_error.contains(required),
            "F5 dynamic component SceneError should expose `{required}`"
        );
    }

    for forbidden in [
        "pub fn register_component_type(\n        &mut self,\n        descriptor: ComponentTypeDescriptor,\n    ) -> Result<(), String>",
        "pub fn set_dynamic_component(",
        "pub fn remove_dynamic_component(",
        "pub fn ensure_plugin_components_can_unload(&self, plugin_id: &str) -> Result<(), String>",
        "pub(crate) fn set_dynamic_component_property(",
        "Err(format!(",
        "error.to_string()",
    ] {
        if forbidden == "pub fn set_dynamic_component("
            || forbidden == "pub fn remove_dynamic_component("
            || forbidden == "pub(crate) fn set_dynamic_component_property("
        {
            continue;
        }
        assert!(
            !dynamic_components.contains(forbidden),
            "F5 dynamic component owner should not keep lossy String/error branch `{forbidden}`"
        );
    }
    assert!(
        dynamic_components.contains("pub fn set_dynamic_component(")
            && dynamic_components.contains(") -> SceneResult<bool>")
            && dynamic_components.contains("self.validate_dynamic_component_type(&component_id)?;")
            && dynamic_components.contains("SceneError::PluginComponentsActive")
            && dynamic_components.contains("SceneError::UnknownDynamicComponentProperty")
            && !dynamic_components.contains("Result<bool, String>")
            && !dynamic_components.contains("Result<(), String>")
            && !dynamic_components.contains("Err(format!(")
            && !dynamic_components.contains("error.to_string()"),
        "World dynamic component mutation methods should return SceneResult without stringifying sources"
    );
    assert!(
        component_type_registry.contains(
            "pub fn register(&mut self, descriptor: ComponentTypeDescriptor) -> SceneResult<()>",
        ) && component_type_registry.contains("SceneError::ComponentTypePluginPrefixMismatch")
            && component_type_registry.contains("SceneError::DuplicateComponentType")
            && !component_type_registry.contains("Result<(), String>")
            && !component_type_registry.contains("Err(format!("),
        "ComponentTypeRegistry should report typed SceneError variants"
    );

    assert!(
        dynamic_scene_error.contains("WorldMutation(#[from] SceneError)")
            && !dynamic_scene_error.contains("WorldMutation(String)")
            && dynamic_scene_spawn.contains("world.register_component_type(descriptor.clone())?;")
            && dynamic_scene_spawn.contains("world.insert_node_record(record)?;")
            && dynamic_scene_spawn.contains(
                "world.set_dynamic_component(entity, component.type_path.clone(), value)?;"
            )
            && !dynamic_scene_spawn.contains("WorldMutation(error.to_string())"),
        "DynamicScene world mutation errors should preserve SceneError sources"
    );
    assert!(
        reflect_dynamic_component.contains("source: error.to_string()")
            && plugin_component_registry.contains("SceneError::DuplicateComponentType")
            && plugin_component_registry.contains("ReflectError::DuplicateTypePath")
            && !plugin_component_registry.contains("contains(\"already registered\")"),
        "reflection/plugin boundaries should stringify or classify typed errors only at their external boundary"
    );
    assert!(
        ecs_typed_tests.contains("dynamic_component_mutation_errors_report_scene_error_variants")
            && (dynamic_scene_tests
                .contains("dynamic_scene_world_mutation_preserves_scene_error_source")
                || dynamic_scene_patch_tests
                    .contains("dynamic_scene_world_mutation_preserves_scene_error_source")),
        "F5 should keep behavior coverage for dynamic component and DynamicScene typed errors"
    );

    for doc_anchor in [
        "F5 dynamic component typed errors",
        "dynamic_component_typed_errors_coremin_check_passed",
        "review_f5_dynamic_component_errors_preserve_scene_error_sources",
        "dynamic_component_mutation_errors_report_scene_error_variants",
        "dynamic_scene_world_mutation_preserves_scene_error_source",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_08_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || ecs_doc.contains(doc_anchor),
            "F5 dynamic component docs should record `{doc_anchor}`"
        );
    }
}
