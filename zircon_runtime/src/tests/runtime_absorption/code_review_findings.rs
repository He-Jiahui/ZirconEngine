#[test]
fn review_f5_world_spawn_bundle_surface_uses_scene_error() {
    let scene_mod = include_str!("../../scene/mod.rs");
    let world_mod = include_str!("../../scene/world/mod.rs");
    let world_error = include_str!("../../scene/world/error.rs");
    let typed_api = include_str!("../../scene/world/typed_api.rs");
    let fixed_components = include_str!("../../scene/world/typed_api/fixed_components.rs");
    let bundle = include_str!("../../scene/ecs/bundle.rs");
    let command_facade = include_str!("../../scene/ecs/commands/commands/facade.rs");
    let entity_commands = include_str!("../../scene/ecs/commands/commands/entity_commands.rs");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_08_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../docs/zircon_runtime/scene/ecs.md");

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
    let world_error = include_str!("../../scene/world/error.rs");
    let dynamic_components = include_str!("../../scene/world/dynamic_components.rs");
    let component_type_registry = include_str!("../../scene/world/component_type_registry.rs");
    let dynamic_scene_error = include_str!("../../scene/dynamic_scene/error.rs");
    let dynamic_scene_spawn = include_str!("../../scene/dynamic_scene/scene/spawn.rs");
    let reflect_dynamic_component = include_str!("../../scene/reflect/dynamic_component.rs");
    let plugin_component_registry =
        include_str!("../../plugin/extension_registry/apply_to_world/component.rs");
    let ecs_typed_tests = include_str!("../../scene/tests/ecs_typed_api.rs");
    let dynamic_scene_tests = include_str!("../../scene/tests/dynamic_scene.rs");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_08_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let ecs_doc = include_str!("../../../../docs/zircon_runtime/scene/ecs.md");

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
    let registry = include_str!("../../core/resource/registry.rs");
    let registry_ops = include_str!("../../core/resource/manager/registry_ops.rs");
    let core_error = include_str!("../../core/framework/error.rs");
    let core_mod = include_str!("../../core/mod.rs");
    let resource_tests = include_str!("../../core/resource/tests.rs");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_02_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let resource_doc = include_str!("../../../../docs/zircon_runtime/core/resource.md");

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
    let importer_error = include_str!("../../asset/importer/error.rs");
    let cache_payload = include_str!("../../asset/artifact/cache_payload.rs");
    let toml_value = include_str!("../../asset/artifact/cache_payload/toml_value.rs");
    let artifact_store = include_str!("../../asset/artifact/store.rs");
    let importer_tests = include_str!("../../asset/tests/assets/importer.rs");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_04_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let artifact_doc = include_str!("../../../../docs/zircon_runtime/asset/artifact.md");

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

#[test]
fn review_f8_texture_import_settings_use_fallible_apply_not_with() {
    let descriptor = include_str!("../../asset/assets/texture/descriptor.rs");
    let texture_asset = include_str!("../../asset/assets/texture/texture_asset.rs");
    let runtime_importer = include_str!("../../asset/importer/ingest/import_texture.rs");
    let plugin_importer =
        include_str!("../../../../zircon_plugins/texture_importer/runtime/src/importers.rs");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_04_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let importer_doc = include_str!("../../../../docs/zircon_runtime/asset/importer.md");
    let render_asset_doc = include_str!("../../../../docs/zircon_runtime/asset/render-assets.md");

    let old_fallible_with_name = ["with", "import", "settings"].join("_");
    for (name, source) in [
        ("TextureAssetDescriptor", descriptor),
        ("TextureAsset", texture_asset),
        ("runtime texture importer", runtime_importer),
        ("texture importer plugin", plugin_importer),
    ] {
        assert!(
            source.contains("apply_import_settings"),
            "F8 texture import settings source `{name}` should use the fallible apply_* API"
        );
        assert!(
            !source.contains(&old_fallible_with_name),
            "F8 texture import settings source `{name}` should not keep the fallible with_* API"
        );
    }
    assert!(
        descriptor
            .contains("pub fn apply_import_settings(mut self, settings: &toml::Table) -> Result<Self, String>")
            && texture_asset.contains(
                "pub fn apply_import_settings(mut self, settings: &toml::Table) -> Result<Self, String>"
            ),
        "Texture import settings should remain fallible but no longer use a builder-style with_* verb"
    );
    assert!(
        texture_asset.contains(".apply_import_settings(settings)?")
            && runtime_importer.contains(".apply_import_settings(&context.import_settings)")
            && plugin_importer.contains(".apply_import_settings(&context.import_settings)"),
        "Runtime and plugin importers should call the fallible apply_import_settings entry"
    );

    for doc_anchor in [
        "F8 texture import settings apply API",
        "texture_import_settings_apply_api_coremin_check_passed",
        "review_f8_texture_import_settings_use_fallible_apply_not_with",
        "apply_import_settings",
        "RuntimePluginDescriptor public-field convergence remains pending",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_04_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || importer_doc.contains(doc_anchor)
                || render_asset_doc.contains(doc_anchor),
            "F8 texture import settings docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f8_runtime_plugin_descriptor_exposes_builder_scaffold() {
    let descriptor = include_str!("../../plugin/runtime_plugin/descriptor.rs");
    let builder_mod = include_str!("../../plugin/runtime_plugin/descriptor/builder.rs");
    let builder_source = include_str!(
        "../../plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs"
    );
    let runtime_plugin_mod = include_str!("../../plugin/runtime_plugin/mod.rs");
    let plugin_mod = include_str!("../../plugin/mod.rs");
    let plugin_descriptor_tests =
        include_str!("../../tests/plugin_extensions/runtime_plugin_descriptor.rs");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../docs/zircon_runtime/plugin/package_manifest.md");

    assert!(
        descriptor.contains("pub use builder::RuntimePluginDescriptorBuilder;"),
        "RuntimePluginDescriptor should re-export RuntimePluginDescriptorBuilder from its descriptor owner"
    );
    assert!(
        builder_mod.contains("mod runtime_plugin_descriptor_builder;")
            && builder_mod.contains(
                "pub use runtime_plugin_descriptor_builder::RuntimePluginDescriptorBuilder;"
            ),
        "descriptor builder module should export the RuntimePluginDescriptorBuilder owner"
    );
    for required in [
        "pub struct RuntimePluginDescriptorBuilder",
        "descriptor: RuntimePluginDescriptor",
        "pub fn builder(",
        ") -> RuntimePluginDescriptorBuilder",
        "pub fn build(self) -> RuntimePluginDescriptor",
        "with_optional_feature",
        "with_default_packaging",
    ] {
        assert!(
            builder_source.contains(required),
            "RuntimePluginDescriptorBuilder source should contain `{required}`"
        );
    }
    assert!(
        runtime_plugin_mod
            .contains("pub use descriptor::{RuntimePluginDescriptor, RuntimePluginDescriptorBuilder};")
            && plugin_mod.contains("RuntimePluginDescriptorBuilder,"),
        "RuntimePluginDescriptorBuilder should be exported through runtime_plugin and plugin facades"
    );
    assert!(
        plugin_descriptor_tests
            .contains("runtime_plugin_descriptor_builder_matches_fluent_descriptor_projection")
            && plugin_descriptor_tests.contains("RuntimePluginDescriptor::builder(")
            && plugin_descriptor_tests.contains(".build()"),
        "RuntimePluginDescriptor builder behavior should stay covered by focused plugin descriptor tests"
    );

    for doc_anchor in [
        "F8 RuntimePluginDescriptor builder scaffold",
        "runtime_plugin_descriptor_builder_scaffold_coremin_check_passed",
        "review_f8_runtime_plugin_descriptor_exposes_builder_scaffold",
        "RuntimePluginDescriptorBuilder",
        "RuntimePluginDescriptor public-field convergence remains pending",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_06_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || package_manifest_doc.contains(doc_anchor),
            "RuntimePluginDescriptor builder docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f8_first_party_runtime_plugin_descriptors_use_builder() {
    let plugin_sources = [
        (
            "ai",
            include_str!("../../../../zircon_plugins/ai/runtime/src/lib.rs"),
        ),
        (
            "animation",
            include_str!("../../../../zircon_plugins/animation/runtime/src/lib.rs"),
        ),
        (
            "hybrid_gi",
            include_str!("../../../../zircon_plugins/hybrid_gi/runtime/src/lib.rs"),
        ),
        (
            "navigation",
            include_str!("../../../../zircon_plugins/navigation/runtime/src/lib.rs"),
        ),
        (
            "net",
            include_str!("../../../../zircon_plugins/net/runtime/src/lib.rs"),
        ),
        (
            "particles",
            include_str!("../../../../zircon_plugins/particles/runtime/src/lib.rs"),
        ),
        (
            "physics",
            include_str!("../../../../zircon_plugins/physics/runtime/src/lib.rs"),
        ),
        (
            "prefab_tools",
            include_str!("../../../../zircon_plugins/prefab_tools/runtime/src/lib.rs"),
        ),
        (
            "rendering",
            include_str!("../../../../zircon_plugins/rendering/runtime/src/lib.rs"),
        ),
        (
            "solari",
            include_str!("../../../../zircon_plugins/solari/runtime/src/lib.rs"),
        ),
        (
            "sound",
            include_str!(
                "../../../../zircon_plugins/sound/runtime/src/runtime_plugin/descriptor.rs"
            ),
        ),
        (
            "terrain",
            include_str!("../../../../zircon_plugins/terrain/runtime/src/lib.rs"),
        ),
        (
            "texture",
            include_str!("../../../../zircon_plugins/texture/runtime/src/lib.rs"),
        ),
        (
            "tilemap_2d",
            include_str!("../../../../zircon_plugins/tilemap_2d/runtime/src/lib.rs"),
        ),
        (
            "virtual_geometry",
            include_str!("../../../../zircon_plugins/virtual_geometry/runtime/src/lib.rs"),
        ),
        (
            "zr_vm_language",
            include_str!("../../../../zircon_plugins/zr_vm_language/runtime/src/lib.rs"),
        ),
    ];
    assert_eq!(
        plugin_sources.len(),
        16,
        "F8 first-party runtime plugin descriptor migration should enumerate every production runtime plugin"
    );

    for (name, source) in plugin_sources {
        assert!(
            source.contains("RuntimePluginDescriptor::builder("),
            "first-party runtime plugin `{name}` should construct descriptors through the builder"
        );
        assert!(
            !source.contains("RuntimePluginDescriptor::new("),
            "first-party runtime plugin `{name}` should not keep the old descriptor constructor"
        );
        assert!(
            source.contains(".build()"),
            "first-party runtime plugin `{name}` should finish descriptor construction with build()"
        );
    }

    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../docs/zircon_runtime/plugin/package_manifest.md");
    let first_party_catalog_doc =
        include_str!("../../../../docs/zircon_plugins/first_party_runtime_catalog.md");

    for doc_anchor in [
        "F8 first-party RuntimePluginDescriptor builder migration",
        "runtime_plugin_descriptor_first_party_builder_migration_coremin_check_passed",
        "review_f8_first_party_runtime_plugin_descriptors_use_builder",
        "first-party runtime plugin descriptor production files 16/16",
        "RuntimePluginDescriptor public-field convergence remains pending",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_06_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || package_manifest_doc.contains(doc_anchor)
                || first_party_catalog_doc.contains(doc_anchor),
            "first-party RuntimePluginDescriptor builder migration docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f8_runtime_plugin_descriptor_test_fixtures_use_builder() {
    let fixture_sources = [
        (
            "asset_importer_install",
            include_str!("../plugin_extensions/asset_importer_install.rs"),
        ),
        (
            "extension_registry",
            include_str!("../plugin_extensions/extension_registry.rs"),
        ),
        (
            "extension_registry_components",
            include_str!("../plugin_extensions/extension_registry_components.rs"),
        ),
        (
            "extension_registry_event_catalogs",
            include_str!("../plugin_extensions/extension_registry_event_catalogs.rs"),
        ),
        (
            "extension_registry_features",
            include_str!("../plugin_extensions/extension_registry_features.rs"),
        ),
        (
            "extension_registry_managers",
            include_str!("../plugin_extensions/extension_registry_managers.rs"),
        ),
        (
            "extension_registry_metadata",
            include_str!("../plugin_extensions/extension_registry_metadata.rs"),
        ),
        (
            "extension_registry_modules",
            include_str!("../plugin_extensions/extension_registry_modules.rs"),
        ),
        (
            "extension_registry_options",
            include_str!("../plugin_extensions/extension_registry_options.rs"),
        ),
        (
            "profile_maturity",
            include_str!("../plugin_extensions/profile_maturity.rs"),
        ),
        (
            "runtime_plugin_catalog_features",
            include_str!("../plugin_extensions/runtime_plugin_catalog_features.rs"),
        ),
        (
            "runtime_plugin_descriptor",
            include_str!("../plugin_extensions/runtime_plugin_descriptor.rs"),
        ),
        (
            "runtime_plugin_lifecycle",
            include_str!("../plugin_extensions/runtime_plugin_lifecycle.rs"),
        ),
        (
            "runtime_plugin_package_manifest",
            include_str!("../plugin_extensions/runtime_plugin_package_manifest.rs"),
        ),
    ];
    assert_eq!(
        fixture_sources.len(),
        14,
        "F8 RuntimePluginDescriptor fixture migration should enumerate every plugin extension test file that used the old constructor"
    );

    let builder_count: usize = fixture_sources
        .iter()
        .map(|(_, source)| {
            source
                .match_indices("RuntimePluginDescriptor::builder(")
                .count()
        })
        .sum();
    assert_eq!(
        builder_count, 64,
        "F8 runtime/plugin extension test fixtures should keep the current builder call count"
    );

    for (name, source) in fixture_sources {
        assert!(
            !source.contains("RuntimePluginDescriptor::new("),
            "plugin extension test fixture `{name}` should not keep RuntimePluginDescriptor::new"
        );
        assert!(
            source.contains("RuntimePluginDescriptor::builder("),
            "plugin extension test fixture `{name}` should construct RuntimePluginDescriptor values through the builder"
        );
    }

    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../docs/zircon_runtime/plugin/package_manifest.md");

    for doc_anchor in [
        "F8 RuntimePluginDescriptor test fixture builder migration",
        "runtime_plugin_descriptor_test_fixture_builder_migration_coremin_check_passed",
        "review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
        "plugin extension RuntimePluginDescriptor test fixtures 14/14",
        "RuntimePluginDescriptor public-field convergence remains pending",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_06_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || package_manifest_doc.contains(doc_anchor),
            "RuntimePluginDescriptor test fixture builder migration docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors() {
    let descriptor = include_str!("../../plugin/runtime_plugin/descriptor.rs");
    let accessors = include_str!("../../plugin/runtime_plugin/descriptor/access.rs");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../docs/zircon_runtime/plugin/package_manifest.md");

    assert!(
        descriptor.contains("mod access;"),
        "RuntimePluginDescriptor should keep field accessors in a dedicated descriptor/access owner"
    );

    for field in [
        "package_id",
        "display_name",
        "category",
        "runtime_id",
        "crate_name",
        "enabled_by_default",
        "required_by_default",
        "target_modes",
        "capabilities",
        "system_sets",
        "system_anchors",
        "capability_statuses",
        "maturity",
        "optional_features",
        "default_packaging",
    ] {
        assert!(
            !descriptor.contains(&format!("pub {field}:")),
            "RuntimePluginDescriptor field `{field}` should not remain public"
        );
    }

    for accessor in [
        "pub fn package_id(&self) -> &str",
        "pub fn display_name(&self) -> &str",
        "pub fn category(&self) -> &str",
        "pub fn runtime_id(&self) -> RuntimePluginId",
        "pub fn crate_name(&self) -> &str",
        "pub fn enabled_by_default(&self) -> bool",
        "pub fn required_by_default(&self) -> bool",
        "pub fn target_modes(&self) -> &[RuntimeTargetMode]",
        "pub fn capabilities(&self) -> &[String]",
        "pub fn system_sets(&self) -> &[String]",
        "pub fn system_anchors(&self) -> &[String]",
        "pub fn capability_statuses(&self) -> &[CapabilityStatusManifest]",
        "pub fn maturity(&self) -> PluginMaturity",
        "pub fn optional_features(&self) -> &[PluginFeatureBundleManifest]",
        "pub fn default_packaging(&self) -> &[ExportPackagingStrategy]",
    ] {
        assert!(
            accessors.contains(accessor),
            "RuntimePluginDescriptor accessors should expose `{accessor}`"
        );
    }

    for doc_anchor in [
        "F8 RuntimePluginDescriptor public-field convergence",
        "runtime_plugin_descriptor_public_field_convergence_coremin_check_passed",
        "review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors",
        "RuntimePluginDescriptor private fields 15/15",
        "RuntimePluginDescriptor public-field convergence complete",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_06_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || package_manifest_doc.contains(doc_anchor),
            "RuntimePluginDescriptor public-field convergence docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f8_runtime_plugin_descriptor_public_constructor_is_retired() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("runtime crate should have repository parent");
    let descriptor_builder_mod = include_str!("../../plugin/runtime_plugin/descriptor/builder.rs");
    let descriptor_builder = include_str!(
        "../../plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs"
    );
    let builtin_catalog_root = include_str!("../../plugin/runtime_plugin/builtin_catalog.rs");
    let plugin_sdk_runtime =
        std::fs::read_to_string(repo_root.join("zircon_plugins/plugin_sdk/src/runtime.rs"))
            .expect("read plugin SDK runtime declaration source");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_06_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let package_manifest_doc =
        include_str!("../../../../docs/zircon_runtime/plugin/package_manifest.md");

    for retired_owner in [
        manifest_dir.join("src/plugin/runtime_plugin/descriptor/builder/construction.rs"),
        manifest_dir.join("src/plugin/runtime_plugin/descriptor/builder/fluent.rs"),
    ] {
        assert!(
            !retired_owner.exists(),
            "RuntimePluginDescriptor retired constructor/fluent owner should stay absent: {}",
            retired_owner.display()
        );
    }
    for forbidden in ["mod construction;", "mod fluent;"] {
        assert!(
            !descriptor_builder_mod.contains(forbidden),
            "RuntimePluginDescriptor builder module should not mount retired owner `{forbidden}`"
        );
    }
    for forbidden in ["RuntimePluginDescriptor::new("] {
        assert!(
            !descriptor_builder.contains(forbidden) && !plugin_sdk_runtime.contains(forbidden),
            "RuntimePluginDescriptor builder/SDK should not keep retired public constructor surface `{forbidden}`"
        );
    }
    assert!(
        descriptor_builder.contains("descriptor: RuntimePluginDescriptor {")
            && plugin_sdk_runtime.contains("builder: RuntimePluginDescriptorBuilder")
            && plugin_sdk_runtime.contains("RuntimePluginDescriptor::builder(")
            && builtin_catalog_root.contains("Self::builder(")
            && builtin_catalog_root.contains("type BuiltinCatalogDescriptorBuilder = RuntimePluginDescriptorBuilder;")
            && builtin_catalog_root.contains(".map(RuntimePluginDescriptorBuilder::build)")
            && !plugin_sdk_runtime.contains("self.descriptor = self.descriptor.with_"),
        "RuntimePluginDescriptor builder and plugin SDK declaration should use the blessed builder storage path"
    );

    fn collect_builtin_catalog_sources(
        directory: &std::path::Path,
        sources: &mut Vec<(std::path::PathBuf, String)>,
    ) {
        for entry in std::fs::read_dir(directory).expect("read builtin catalog directory") {
            let entry = entry.expect("read builtin catalog entry");
            let path = entry.path();
            if path.is_dir() {
                collect_builtin_catalog_sources(&path, sources);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                let source =
                    std::fs::read_to_string(&path).expect("read builtin catalog source file");
                sources.push((path, source));
            }
        }
    }

    let mut builtin_catalog_sources = Vec::new();
    collect_builtin_catalog_sources(
        &manifest_dir.join("src/plugin/runtime_plugin/builtin_catalog"),
        &mut builtin_catalog_sources,
    );
    for (path, source) in builtin_catalog_sources {
        assert!(
            !source.contains("RuntimePluginDescriptor"),
            "builtin catalog child modules should pass the descriptor builder instead of direct descriptor values: {}",
            path.display()
        );
    }

    for doc_anchor in [
        "F8 RuntimePluginDescriptor public constructor retired",
        "runtime_plugin_descriptor_public_constructor_retired_coremin_check_passed",
        "review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
        "RuntimePluginDescriptor::new retired",
        "descriptor/builder/construction.rs retired",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_06_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || package_manifest_doc.contains(doc_anchor),
            "RuntimePluginDescriptor public constructor retirement docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f11_shading_model_registry_has_no_dead_plugin_registration_surface() {
    let registry = include_str!("../../graphics/material/shading_models/registry.rs");
    let core_contract = include_str!("../../core/framework/render/material/shading_model.rs");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let render_index = include_str!("../../../../docs/plans/zircon_runtime/render/index.md");
    let material_doc =
        include_str!("../../../../docs/zircon_runtime/core/framework/render/material.md");

    assert!(
        !registry.contains("#[allow(dead_code)]"),
        "shading-model registry should not preserve disconnected production API behind dead_code"
    );
    for forbidden_dead_surface in [
        "pub(crate) fn supported_channels(",
        "pub(crate) fn len(",
        "pub(crate) fn register_plugin(",
        "PluginIdBelowReservedRange",
    ] {
        assert!(
            !registry.contains(forbidden_dead_surface)
                && !core_contract.contains(forbidden_dead_surface),
            "F11 should not reintroduce disconnected shading-model registry surface `{forbidden_dead_surface}`"
        );
    }

    for live_registry_anchor in [
        "fn resolve_token(&self, token: &str)",
        "self.resolve_token(&model.as_token())",
        "pub(crate) fn register_builtin(",
        "RequiredChannelsUnsupported",
    ] {
        assert!(
            registry.contains(live_registry_anchor),
            "shading-model registry should retain live built-in resolver anchor `{live_registry_anchor}`"
        );
    }

    for doc_anchor in [
        "F11 shading-model registry dead API removal",
        "render_shading_model_registry_dead_api_removed_coremin_passed",
        "review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
        "custom shading-model plugin registration remains a future Plan 08 surface",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || render_index.contains(doc_anchor)
                || material_doc.contains(doc_anchor),
            "F11 docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f17_entity_path_option_lookup_uses_get_verb() {
    let path_resolution = include_str!("../../scene/world/property_access/path_resolution.rs");
    let runtime_apply = include_str!("../../animation/sequence/apply.rs");
    let runtime_target = include_str!("../../animation/sequence/target.rs");
    let plugin_apply =
        include_str!("../../../../zircon_plugins/animation/runtime/src/sequence/apply.rs");
    let plugin_target =
        include_str!("../../../../zircon_plugins/animation/runtime/src/sequence/target.rs");
    let property_paths = include_str!("../../scene/tests/property_paths.rs");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let runtime_08 = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let ecs_doc = include_str!("../../../../docs/zircon_runtime/scene/ecs.md");
    let animation_doc =
        include_str!("../../../../docs/assets-and-rendering/runtime-physics-animation-assets.md");
    let editor_boundary_doc =
        include_str!("../../../../docs/editor-and-tooling/runtime-editor-boundary-cleanup.md");

    let old_option_lookup = ["resolve", "entity", "path"].join("_");
    assert!(
        path_resolution
            .contains("pub fn get_entity_by_path(&self, path: &EntityPath) -> Option<EntityId>"),
        "F17 entity path Option lookup should use get_* naming"
    );
    assert!(
        !path_resolution.contains(&old_option_lookup),
        "F17 should hard-cut the old resolve-verb entity path Option API"
    );

    for (name, source) in [
        ("runtime animation apply", runtime_apply),
        ("runtime animation target", runtime_target),
        ("plugin animation apply", plugin_apply),
        ("plugin animation target", plugin_target),
        ("property path tests", property_paths),
    ] {
        assert!(
            source.contains("get_entity_by_path("),
            "F17 consumer `{name}` should use get_entity_by_path"
        );
        assert!(
            !source.contains(&old_option_lookup),
            "F17 consumer `{name}` should not keep the old resolve-verb entity path lookup"
        );
    }

    for doc_anchor in [
        "F17 entity path Option lookup verb rename",
        "runtime_08_entity_path_lookup_getter_rename_coremin_check_passed",
        "review_f17_entity_path_option_lookup_uses_get_verb",
        "get_entity_by_path",
        "old resolve-verb entity path method absent",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || runtime_08.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || ecs_doc.contains(doc_anchor)
                || animation_doc.contains(doc_anchor)
                || editor_boundary_doc.contains(doc_anchor),
            "F17 docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f18_asset_manager_resolution_returns_registered_handle() {
    let resolver =
        include_str!("../../asset/pipeline/manager/asset_manager/resolve_asset_manager.rs");
    let handle = include_str!("../../asset/pipeline/manager/asset_manager/asset_manager_handle.rs");
    let runtime = include_str!("../../core/runtime/runtime.rs");
    let runtime_handle = include_str!("../../core/runtime/handle/resolution.rs");
    let project_session = include_str!("../../dynamic_api/session/project.rs");
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let runtime_10 = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let dynamic_session_doc =
        include_str!("../../../../docs/zircon_runtime/dynamic_api/session.md");
    let asset_facade_doc = include_str!("../../../../docs/zircon_runtime/asset/facade.md");

    for generic_manager_anchor in [
        "pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError>",
        "self.handle().resolve_manager(name)",
        "let service = self.resolve_named_service(name, Some(ServiceKind::Manager))?;",
        "downcast_resolved_service(name, service)",
    ] {
        assert!(
            runtime.contains(generic_manager_anchor)
                || runtime_handle.contains(generic_manager_anchor),
            "F18 generic manager resolution should keep registered-handle shape `{generic_manager_anchor}`"
        );
    }

    for required in [
        "pub fn resolve_asset_manager(core: &CoreHandle) -> Result<Arc<AssetManagerHandle>, CoreError>",
        "core.resolve_manager::<AssetManagerHandle>(ASSET_MANAGER_NAME)",
    ] {
        assert!(
            resolver.contains(required),
            "F18 asset manager resolver should contain `{required}`"
        );
    }
    for forbidden in [
        "Result<Arc<dyn AssetManager>, CoreError>",
        ".map(|holder| holder.shared())",
    ] {
        assert!(
            !resolver.contains(forbidden),
            "F18 asset manager resolver should not return trait objects directly or hide handle conversion `{forbidden}`"
        );
    }
    assert!(
        handle.contains("pub struct AssetManagerHandle")
            && handle.contains("inner: Arc<dyn AssetManager>")
            && handle.contains("pub fn shared(&self) -> Arc<dyn AssetManager>"),
        "AssetManagerHandle should remain the registered manager handle that owns the object-safe shared service"
    );
    assert!(
        project_session.contains("resolve_asset_manager(core)")
            && project_session.contains("let asset_manager = asset_manager.shared();")
            && project_session.contains(".open_project(&self.root_display())"),
        "dynamic project startup should make the trait-object conversion explicit at the caller boundary"
    );

    for doc_anchor in [
        "F18 asset manager resolution return shape",
        "runtime_10_asset_manager_resolution_handle_shape_coremin_check_passed",
        "review_f18_asset_manager_resolution_returns_registered_handle",
        "Result<Arc<AssetManagerHandle>, CoreError>",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || runtime_10.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || dynamic_session_doc.contains(doc_anchor)
                || asset_facade_doc.contains(doc_anchor),
            "F18 docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f19_scene_renderer_construction_modules_use_construct_names() {
    let core_mod = include_str!("../../graphics/scene/scene_renderer/core/mod.rs");
    let core_construct_mod = include_str!(
        "../../graphics/scene/scene_renderer/core/scene_renderer_core_construct/mod.rs"
    );
    let core_construct_layouts = include_str!(
        "../../graphics/scene/scene_renderer/core/scene_renderer_core_construct/layouts/mod.rs"
    );
    let core_construct_scene_bind_group = include_str!(
        "../../graphics/scene/scene_renderer/core/scene_renderer_core_construct/scene_bind_group_bundle/mod.rs"
    );
    let renderer_construct_mod =
        include_str!("../../graphics/scene/scene_renderer/core/scene_renderer_construct/mod.rs");
    let renderer_construct_new =
        include_str!("../../graphics/scene/scene_renderer/core/scene_renderer_construct/new.rs");
    let renderer_construct_new_with_icon_source = include_str!(
        "../../graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs"
    );
    let review_findings =
        include_str!("../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let convention = include_str!("../../../../docs/plans/engine-code-structure-convention.md");
    let render_index = include_str!("../../../../docs/plans/zircon_runtime/render/index.md");
    let runtime_15 = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let shadow_doc =
        include_str!("../../../../docs/zircon_runtime/graphics/scene/scene_renderer/shadow.md");

    let core_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/graphics/scene/scene_renderer/core");
    let old_core_construct_owner = ["scene_renderer_core", "new"].join("_");
    let old_renderer_construct_owner = ["scene_renderer", "new"].join("_");
    assert!(
        core_dir.join("scene_renderer_core_construct").is_dir()
            && core_dir.join("scene_renderer_construct").is_dir(),
        "F19 scene renderer construction owners should live in construct-named directories"
    );
    assert!(
        !core_dir.join(&old_core_construct_owner).exists()
            && !core_dir.join(&old_renderer_construct_owner).exists(),
        "F19 should hard-cut old *_new construction directories instead of keeping migration paths"
    );

    for required in [
        "mod scene_renderer_core_construct;",
        "mod scene_renderer_construct;",
    ] {
        assert!(
            core_mod.contains(required),
            "scene renderer core module wiring should contain `{required}`"
        );
    }

    for (name, source) in [
        ("core/mod.rs", core_mod),
        ("scene_renderer_core_construct/mod.rs", core_construct_mod),
        (
            "scene_renderer_core_construct/layouts/mod.rs",
            core_construct_layouts,
        ),
        (
            "scene_renderer_core_construct/scene_bind_group_bundle/mod.rs",
            core_construct_scene_bind_group,
        ),
        ("scene_renderer_construct/mod.rs", renderer_construct_mod),
        ("scene_renderer_construct/new.rs", renderer_construct_new),
        (
            "scene_renderer_construct/new_with_icon_source.rs",
            renderer_construct_new_with_icon_source,
        ),
    ] {
        for forbidden in [&old_core_construct_owner, &old_renderer_construct_owner] {
            assert!(
                !source.contains(forbidden),
                "F19 should not leave old construction owner `{forbidden}` in {name}"
            );
        }
    }

    for doc_anchor in [
        "F19 scene renderer construction module rename",
        "render_scene_renderer_construct_modules_coremin_passed",
        "review_f19_scene_renderer_construction_modules_use_construct_names",
        "scene_renderer_core_construct",
        "scene_renderer_construct",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || render_index.contains(doc_anchor)
                || runtime_15.contains(doc_anchor)
                || shadow_doc.contains(doc_anchor),
            "F19 docs should record `{doc_anchor}`"
        );
    }
}
