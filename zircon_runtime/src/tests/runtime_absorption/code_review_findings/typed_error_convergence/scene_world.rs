#[test]
fn review_f5_world_spawn_bundle_surface_uses_scene_error() {
    let scene_mod = include_str!("../../../../scene/mod.rs");
    let world_mod = include_str!("../../../../scene/world/mod.rs");
    let world_error = include_str!("../../../../scene/world/error.rs");
    let typed_api = include_str!("../../../../scene/world/typed_api.rs");
    let identity = include_str!("../../../../scene/world/identity.rs");
    let fixed_components = include_str!("../../../../scene/world/typed_api/fixed_components.rs");
    let bundle = include_str!("../../../../scene/ecs/bundle.rs");
    let command_facade = include_str!("../../../../scene/ecs/commands/commands/facade.rs");
    let entity_commands =
        include_str!("../../../../scene/ecs/commands/commands/entity_commands.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_08_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../../../docs/zircon_runtime/scene/ecs.md");

    for anchor in [
        "pub type SceneResult<T> = std::result::Result<T, SceneError>;",
        "pub enum SceneError",
        "MissingEntity {",
        "EntityRegistry(#[from] EntityRegistryError)",
        "Storage(",
        "#[from] StorageError",
        "impl From<String> for SceneError",
    ] {
        assert!(
            world_error.contains(anchor),
            "F5 world error owner should expose typed error anchor `{anchor}`"
        );
    }
    assert!(
        world_mod.contains("pub use error::{SceneError, SceneResult};")
            && scene_mod.contains("SceneError")
            && scene_mod.contains("SceneResult")
            && scene_mod.contains("World"),
        "SceneError/SceneResult should be exported through the world and scene façades"
    );

    for forbidden in [
        "pub fn spawn<B>(&mut self, bundle: B) -> Result<EntityId, String>",
        "pub(crate) fn spawn_at<B>(&mut self, entity: EntityId, bundle: B) -> Result<EntityId, String>",
        "pub(crate) fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B) -> Result<(), String>",
        "pub fn insert<T>(&mut self, entity: EntityId, component: T) -> Result<Option<T>, String>",
        "pub fn remove<T>(&mut self, entity: EntityId) -> Result<Option<T>, String>",
        "fn insert_into(self, world: &mut World, entity: EntityId) -> Result<(), String>",
        "Result<InternalEntity, String>",
        "Result<(), String>",
    ] {
        assert!(
            !typed_api.contains(forbidden)
                && !identity.contains(forbidden)
                && !fixed_components.contains(forbidden)
                && !bundle.contains(forbidden),
            "F5 should not keep public typed ECS mutation surface as String error `{forbidden}`"
        );
    }
    let insert_body = typed_api
        .split("pub fn insert<T>")
        .nth(1)
        .and_then(|source| source.split("pub fn get<T>").next())
        .expect("read World::insert body");
    let remove_body = typed_api
        .split("pub fn remove<T>")
        .nth(1)
        .and_then(|source| source.split("pub fn resource_id").next())
        .expect("read World::remove body");
    assert!(
        !insert_body.contains("error.to_string()") && !remove_body.contains("error.to_string()"),
        "World::insert/remove should preserve storage errors through SceneError instead of stringifying them"
    );
    assert!(
        !typed_api.contains("error.to_string()") && !identity.contains("error.to_string()"),
        "World typed API identity and presence helpers should preserve typed source errors instead of stringifying them"
    );

    for required in [
        "pub fn spawn<B>(&mut self, bundle: B) -> SceneResult<EntityId>",
        "pub(crate) fn spawn_at<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<EntityId>",
        "pub(crate) fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<()>",
        "pub fn insert<T>(&mut self, entity: EntityId, component: T) -> SceneResult<Option<T>>",
        "pub fn remove<T>(&mut self, entity: EntityId) -> SceneResult<Option<T>>",
        "pub(super) fn register_stable_entity",
        "SceneResult<InternalEntity>",
        "pub(super) fn insert_dynamic_component_presence",
        "pub(super) fn remove_dynamic_component_presence",
        "SceneError::missing_entity(\"insert component on\", entity)",
        "Err(error) => return Err(error.into())",
        ".spawn(entity, EntityLocation::new(ArchetypeId::EMPTY, row))?",
        "fn insert_into(self, world: &mut World, entity: EntityId) -> SceneResult<()>",
    ] {
        assert!(
            typed_api.contains(required)
                || identity.contains(required)
                || fixed_components.contains(required)
                || bundle.contains(required),
            "F5 typed ECS mutation surface should contain `{required}`"
        );
    }
    for command_anchor in [
        "DeferredCommandOperation::Spawn",
        "DeferredCommandOperation::Insert",
        "DeferredCommandOperation::InsertBundle",
        "DeferredCommandOperation::Remove",
        "error.to_string()",
    ] {
        assert!(
            command_facade.contains(command_anchor) || entity_commands.contains(command_anchor),
            "deferred command reporting should stringify typed SceneError only at the report boundary: `{command_anchor}`"
        );
    }

    for doc_anchor in [
        "F5 world typed mutation errors",
        "world_typed_mutation_errors_coremin_check_passed_partial",
        "review_f5_world_spawn_bundle_surface_uses_scene_error",
        "SceneError::MissingEntity",
        "SceneResult",
        "F5 typed API residual typed errors",
        "runtime_15_typed_api_residual_typed_errors_static_passed_cargo_deferred",
        "f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_08_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || ecs_doc.contains(doc_anchor),
            "F5 docs should record `{doc_anchor}`"
        );
    }
    let f5_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F5 |"))
        .expect("F5 review findings top row");
    assert!(
        f5_row.contains("f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred")
            && f5_row.ends_with("| Runtime 08 + Runtime 15 / review closed |"),
        "F5 top row should record typed-error review closed status"
    );
}

#[test]
fn review_f5_fixed_world_mutation_uses_scene_error_variants() {
    let world_error = include_str!("../../../../scene/world/error.rs");
    let component_access = include_str!("../../../../scene/world/component_access.rs");
    let hierarchy = include_str!("../../../../scene/world/hierarchy.rs");
    let query = include_str!("../../../../scene/world/query.rs");
    let records = include_str!("../../../../scene/world/records.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../../../docs/zircon_runtime/scene/ecs.md");

    for required in [
        "MissingRequiredComponent",
        "DuplicateEntity",
        "EmptyNodeName",
        "JointConnectsToSelf",
        "EntityCannotParentItself",
        "MissingParent",
        "HierarchyCycle",
        "DynamicMobilityWithStaticChildren",
        "StaticMobilityUnderDynamicParent",
        "StaticTransformMutation",
        "StaticReparentMutation",
    ] {
        assert!(
            world_error.contains(required),
            "F5 fixed world mutation SceneError should expose `{required}`"
        );
    }

    for (label, source) in [
        ("component access", component_access),
        ("hierarchy", hierarchy),
        ("query", query),
        ("records", records),
    ] {
        for forbidden in [
            "Err(format!(",
            "to_string().into()",
            "Result<(), String>",
            "Result<bool, String>",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep fixed world mutation String error branch `{forbidden}`"
            );
        }
    }

    for required in [
        "SceneError::missing_entity(\"update rigid body for\", entity)",
        "SceneError::JointConnectsToSelf",
        "SceneError::missing_entity(\"reparent\", child)",
        "SceneError::EntityCannotParentItself",
        "SceneError::MissingParent",
        "SceneError::HierarchyCycle",
        "SceneError::DynamicMobilityWithStaticChildren",
        "SceneError::StaticMobilityUnderDynamicParent",
        "SceneError::StaticTransformMutation",
        "SceneError::StaticReparentMutation",
        "SceneError::MissingRequiredComponent",
        "SceneError::missing_entity(\"update active state for\", entity)",
        "SceneError::missing_entity(\"update mobility for\", entity)",
        "SceneError::DuplicateEntity",
        "SceneError::EmptyNodeName",
        "SceneError::missing_entity(\"rename\", entity)",
    ] {
        assert!(
            component_access.contains(required)
                || hierarchy.contains(required)
                || query.contains(required)
                || records.contains(required),
            "fixed world mutation owners should contain `{required}`"
        );
    }

    for doc_anchor in [
        "F5 fixed world mutation typed errors",
        "runtime_15_fixed_world_mutation_typed_errors_static_passed_cargo_deferred",
        "review_f5_fixed_world_mutation_uses_scene_error_variants",
        "SceneError::MissingRequiredComponent",
        "scene/world/component_access.rs",
        "scene/world/hierarchy.rs",
        "scene/world/query.rs",
        "scene/world/records.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || ecs_doc.contains(doc_anchor),
            "F5 fixed world mutation docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_dynamic_component_errors_preserve_scene_error_sources() {
    let world_error = include_str!("../../../../scene/world/error.rs");
    let dynamic_components = include_str!("../../../../scene/world/dynamic_components.rs");
    let component_type_registry =
        include_str!("../../../../scene/world/component_type_registry.rs");
    let dynamic_scene_error = include_str!("../../../../scene/dynamic_scene/error.rs");
    let dynamic_scene_spawn = include_str!("../../../../scene/dynamic_scene/scene/spawn.rs");
    let reflect_dynamic_component = include_str!("../../../../scene/reflect/dynamic_component.rs");
    let plugin_component_registry =
        include_str!("../../../../plugin/extension_registry/apply_to_world/component.rs");
    let ecs_typed_tests = include_str!("../../../../scene/tests/ecs_typed_api.rs");
    let dynamic_scene_tests = include_str!("../../../../scene/tests/dynamic_scene.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_08_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../../../docs/zircon_runtime/scene/ecs.md");

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
            "pub fn register(&mut self, descriptor: ComponentTypeDescriptor) -> SceneResult<()>"
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
            && dynamic_scene_tests
                .contains("dynamic_scene_world_mutation_preserves_scene_error_source"),
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

#[test]
fn review_f5_scene_property_access_uses_scene_error() {
    let world_error = include_str!("../../../../scene/world/error.rs");
    let read = include_str!("../../../../scene/world/property_access/read.rs");
    let write = include_str!("../../../../scene/world/property_access/write.rs");
    let write_physics = include_str!("../../../../scene/world/property_access/write/physics.rs");
    let value_conversion =
        include_str!("../../../../scene/world/property_access/value_conversion.rs");
    let read_path_tests = include_str!("../../../../scene/tests/property_paths/read_paths.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../../../docs/zircon_runtime/scene/ecs.md");

    for required in [
        "PropertyUnavailable",
        "PropertySegmentCount",
        "UnknownProperty",
        "MissingPropertyComponent",
        "PropertyTypeMismatch",
        "UnknownPropertyAxis",
        "ZeroLengthQuaternion",
        "NonFinitePropertyValue",
        "InvalidPropertyResourceId",
        "UnsupportedPropertyValue",
        "ReadOnlyProperty",
        "InvalidPropertyIndex",
    ] {
        assert!(
            world_error.contains(required),
            "F5 scene property access SceneError should expose `{required}`"
        );
    }

    for (label, source) in [
        ("property read", read),
        ("property write", write),
        ("property physics write", write_physics),
        ("property conversion", value_conversion),
    ] {
        for forbidden in [
            "Result<ScenePropertyValue, String>",
            "Result<bool, String>",
            "Result<(), String>",
            "Err(format!(",
            "map_err(|error| error.to_string())",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String property error branch `{forbidden}`"
            );
        }
    }

    for required in [
        ") -> SceneResult<ScenePropertyValue>",
        "SceneError::PropertyUnavailable",
    ] {
        assert!(
            read.contains(required),
            "property read should contain typed error anchor `{required}`"
        );
    }
    for required in [
        "pub fn set_property(",
        ") -> SceneResult<bool>",
        "SceneError::missing_entity(\"update\", entity)",
        "SceneError::ReadOnlyProperty",
        "SceneError::InvalidPropertyIndex",
        "self.set_dynamic_component_property(entity, property_path, value)",
    ] {
        assert!(
            write.contains(required) || write_physics.contains(required),
            "property writer should contain typed error anchor `{required}`"
        );
    }
    for required in [
        "pub(super) fn expect_segment_count",
        "SceneError::PropertySegmentCount",
        "SceneError::PropertyTypeMismatch",
        "SceneError::UnknownPropertyAxis",
        "SceneError::InvalidPropertyResourceId",
        "SceneError::UnsupportedPropertyValue",
        ") -> SceneResult<bool>",
    ] {
        assert!(
            value_conversion.contains(required) || write_physics.contains(required),
            "property conversion should contain typed error anchor `{required}`"
        );
    }
    assert!(
        read_path_tests.contains("SceneError::PropertyUnavailable")
            && read_path_tests.contains(") -> SceneResult<ScenePropertyValue>"),
        "property read path source guard should be updated to the typed SceneError contract"
    );

    for doc_anchor in [
        "F5 scene property access typed errors",
        "runtime_15_scene_property_access_typed_errors_static_passed_cargo_deferred",
        "review_f5_scene_property_access_uses_scene_error",
        "SceneError::PropertyUnavailable",
        "World::property",
        "World::set_property",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || ecs_doc.contains(doc_anchor),
            "F5 property access docs should record `{doc_anchor}`"
        );
    }
}
