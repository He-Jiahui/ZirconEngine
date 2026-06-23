#[test]
fn review_f5_world_spawn_bundle_surface_uses_scene_error() {
    let scene_mod = include_str!("../../../scene/mod.rs");
    let world_mod = include_str!("../../../scene/world/mod.rs");
    let world_error = include_str!("../../../scene/world/error.rs");
    let typed_api = include_str!("../../../scene/world/typed_api.rs");
    let fixed_components = include_str!("../../../scene/world/typed_api/fixed_components.rs");
    let bundle = include_str!("../../../scene/ecs/bundle.rs");
    let command_facade = include_str!("../../../scene/ecs/commands/commands/facade.rs");
    let entity_commands = include_str!("../../../scene/ecs/commands/commands/entity_commands.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_08_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../../docs/zircon_runtime/scene/ecs.md");

    for anchor in [
        "pub type SceneResult<T> = std::result::Result<T, SceneError>;",
        "pub enum SceneError",
        "MissingEntity {",
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
    ] {
        assert!(
            !typed_api.contains(forbidden)
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

    for required in [
        "pub fn spawn<B>(&mut self, bundle: B) -> SceneResult<EntityId>",
        "pub(crate) fn spawn_at<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<EntityId>",
        "pub(crate) fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<()>",
        "pub fn insert<T>(&mut self, entity: EntityId, component: T) -> SceneResult<Option<T>>",
        "pub fn remove<T>(&mut self, entity: EntityId) -> SceneResult<Option<T>>",
        "SceneError::missing_entity(\"insert component on\", entity)",
        "Err(error) => return Err(error.into())",
        "fn insert_into(self, world: &mut World, entity: EntityId) -> SceneResult<()>",
    ] {
        assert!(
            typed_api.contains(required)
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
}

#[test]
fn review_f5_dynamic_component_errors_preserve_scene_error_sources() {
    let world_error = include_str!("../../../scene/world/error.rs");
    let dynamic_components = include_str!("../../../scene/world/dynamic_components.rs");
    let component_type_registry = include_str!("../../../scene/world/component_type_registry.rs");
    let dynamic_scene_error = include_str!("../../../scene/dynamic_scene/error.rs");
    let dynamic_scene_spawn = include_str!("../../../scene/dynamic_scene/scene/spawn.rs");
    let reflect_dynamic_component = include_str!("../../../scene/reflect/dynamic_component.rs");
    let plugin_component_registry =
        include_str!("../../../plugin/extension_registry/apply_to_world/component.rs");
    let ecs_typed_tests = include_str!("../../../scene/tests/ecs_typed_api.rs");
    let dynamic_scene_tests = include_str!("../../../scene/tests/dynamic_scene.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_08_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../../docs/zircon_runtime/scene/ecs.md");

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
fn review_f6_core_resource_registry_rename_uses_core_error() {
    let registry = include_str!("../../../core/resource/registry.rs");
    let registry_ops = include_str!("../../../core/resource/manager/registry_ops.rs");
    let core_error = include_str!("../../../core/framework/error.rs");
    let core_mod = include_str!("../../../core/mod.rs");
    let resource_tests = include_str!("../../../core/resource/tests.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_02_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let resource_doc = include_str!("../../../../../docs/zircon_runtime/core/resource.md");

    for required in [
        "pub type CoreResult<T> = std::result::Result<T, CoreError>;",
        "MissingResourceRecordForLocator { locator: String }",
        "MissingResourceRecordForId { id: String }",
    ] {
        assert!(
            core_error.contains(required),
            "F6 CoreError contract should contain `{required}`"
        );
    }
    assert!(
        core_mod.contains("pub use framework::error::{CoreError, CoreResult, ZirconError};"),
        "CoreResult should be exported beside CoreError"
    );

    for forbidden in [
        ") -> Result<ResourceRecord, String>",
        "Err(format!(\"missing resource record",
    ] {
        assert!(
            !registry.contains(forbidden) && !registry_ops.contains(forbidden),
            "F6 should not keep resource registry String error surface `{forbidden}`"
        );
    }

    for required in [
        ") -> CoreResult<ResourceRecord>",
        "CoreError::MissingResourceRecordForLocator",
        "CoreError::MissingResourceRecordForId",
        "self.id_by_locator.get(from).copied()",
        "self.id_by_locator.remove(from);",
    ] {
        assert!(
            registry.contains(required) || registry_ops.contains(required),
            "F6 resource registry rename should contain `{required}`"
        );
    }
    assert!(
        resource_tests.contains("registry_rename_reports_missing_locator_with_core_error")
            && resource_tests.contains("CoreError::MissingResourceRecordForLocator"),
        "F6 should keep focused behavior coverage for missing resource locator errors"
    );

    for doc_anchor in [
        "F6 core resource registry typed errors",
        "core_resource_registry_typed_errors_coremin_check_passed",
        "review_f6_core_resource_registry_rename_uses_core_error",
        "registry_rename_reports_missing_locator_with_core_error",
        "MissingResourceRecordForLocator",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_02_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || resource_doc.contains(doc_anchor),
            "F6 docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f7_asset_artifact_errors_use_asset_import_error_sources() {
    let importer_error = include_str!("../../../asset/importer/error.rs");
    let cache_payload = include_str!("../../../asset/artifact/cache_payload.rs");
    let toml_value = include_str!("../../../asset/artifact/cache_payload/toml_value.rs");
    let artifact_store = include_str!("../../../asset/artifact/store.rs");
    let importer_tests = include_str!("../../../asset/tests/assets/importer.rs");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_04_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let artifact_doc = include_str!("../../../../../docs/zircon_runtime/asset/artifact.md");

    for forbidden in [
        "Registry(String)",
        "Self::Registry(error.to_string())",
        "impl From<AssetImporterRegistryError> for AssetImportError",
    ] {
        assert!(
            !importer_error.contains(forbidden),
            "F7 should not preserve lossy registry error conversion `{forbidden}`"
        );
    }
    for required in [
        "Registry(#[from] AssetImporterRegistryError)",
        "TomlSerialize {",
        "TomlDeserialize {",
        "CachedTomlDatetime {",
        "UiDocument {",
        "UiV2Document {",
        "ArtifactCacheSerialize(#[source] bincode::Error)",
        "ArtifactCacheDeserialize(#[source] bincode::Error)",
    ] {
        assert!(
            importer_error.contains(required),
            "F7 AssetImportError should expose typed source anchor `{required}`"
        );
    }

    for forbidden in [
        "pub(super) fn from_imported(asset: &ImportedAsset) -> Result<Self, String>",
        "pub(super) fn into_imported(self) -> Result<ImportedAsset, String>",
        "fn into_asset(self) -> Result<MaterialAsset, String>",
        "fn into_asset(self) -> Result<ShaderAsset, String>",
        "fn into_asset(self) -> Result<ShaderMaterialPropertyAsset, String>",
        "format!(\"serialize ui asset document cache",
        "format!(\"deserialize ui layout document cache",
        "format!(\"deserialize ui v2 view document cache",
    ] {
        assert!(
            !cache_payload.contains(forbidden),
            "F7 cache payload should not keep String error/string-format anchor `{forbidden}`"
        );
    }
    for required in [
        "use crate::asset::{",
        "AssetImportError,",
        "pub(super) fn from_imported(asset: &ImportedAsset) -> Result<Self, AssetImportError>",
        "pub(super) fn into_imported(self) -> Result<ImportedAsset, AssetImportError>",
        "AssetImportError::TomlSerialize",
        "AssetImportError::UiDocument",
        "AssetImportError::UiV2Document",
    ] {
        assert!(
            cache_payload.contains(required),
            "F7 cache payload should use AssetImportError anchor `{required}`"
        );
    }

    assert!(
        toml_value.contains("Result<toml::Value, AssetImportError>")
            && toml_value.contains("AssetImportError::CachedTomlDatetime")
            && !toml_value.contains("format!(\"invalid cached TOML datetime"),
        "F7 TOML cache conversion should report typed cached datetime errors"
    );
    for required in [
        "map_err(AssetImportError::ArtifactCacheSerialize)",
        "map_err(AssetImportError::ArtifactCacheDeserialize)",
        "let cache_asset = ArtifactCacheAsset::from_imported(asset)?;",
        "let asset = cache_asset.into_imported()?;",
    ] {
        assert!(
            artifact_store.contains(required),
            "F7 artifact store should preserve typed source anchor `{required}`"
        );
    }
    assert!(
        !artifact_store
            .contains("map_err(|error| AssetImportError::Parse(format!(\"serialize artifact cache")
            && !artifact_store.contains(
                "map_err(|error| AssetImportError::Parse(format!(\"deserialize artifact cache"
            ),
        "F7 artifact store should not lossy-wrap cache conversion sources in Parse(String)"
    );
    assert!(
        importer_tests.contains("asset_import_error_preserves_registry_error_source")
            && importer_tests.contains(
                "AssetImportError::Registry(AssetImporterRegistryError::DuplicateMatcher"
            ),
        "F7 should keep behavior coverage for typed registry error preservation"
    );

    for doc_anchor in [
        "F7 asset artifact/importer typed errors",
        "asset_artifact_importer_typed_errors_coremin_passed",
        "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        "asset_import_error_preserves_registry_error_source",
        "AssetImportError::CachedTomlDatetime",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_04_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || artifact_doc.contains(doc_anchor),
            "F7 docs should record `{doc_anchor}`"
        );
    }
}
