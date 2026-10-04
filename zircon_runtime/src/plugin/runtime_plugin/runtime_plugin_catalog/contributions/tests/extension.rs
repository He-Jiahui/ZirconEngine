use std::collections::HashSet;
use std::fs;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::assets::{SceneAsset, SceneComponentAssetRecord};
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::AssetUri;
use crate::core::framework::bridge::{BridgeError, PluginInterface};
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::core::ModuleDescriptor;
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::components::{NodeKind, NodeRecord};
use crate::scene::world::{SceneComponentSerializer, SceneProjectError, World};
use serde_json::Value;
use zircon_runtime_interface::project::RelPath;

use super::{
    merge_extension_registry_contributions,
    merge_extension_registry_contributions_for_runtime_modules,
};

trait MergeTestBridge: Send + Sync {
    fn sample(&self) -> i32;
}

impl PluginInterface for dyn MergeTestBridge {
    const INTERFACE_ID: &'static str = "test.final.merge.bridge.v1";
}

struct MergeTestProvider(i32);

fn merge_codec_capture(
    _project: &ProjectManager,
    _world: &World,
    _record: &NodeRecord,
) -> Result<Option<SceneComponentAssetRecord>, SceneProjectError> {
    Ok(None)
}

fn merge_codec_instantiate(
    _project: &ProjectManager,
    _row: &SceneComponentAssetRecord,
    _record: &mut NodeRecord,
) -> Result<Option<(String, Value)>, SceneProjectError> {
    Ok(None)
}

const CATALOG_TYPE_ID: &str = "catalog_scene.component_probe";
const CATALOG_SCHEMA_ID: &str = "catalog_scene.component_probe.v1";
const CATALOG_PROVIDER_ID: &str = "catalog_scene";

fn catalog_codec_capture(
    _project: &ProjectManager,
    world: &World,
    record: &NodeRecord,
) -> Result<Option<SceneComponentAssetRecord>, SceneProjectError> {
    let Some(payload) = world.dynamic_component(record.id, CATALOG_TYPE_ID) else {
        return Ok(None);
    };
    SceneComponentAssetRecord::from_typed(
        CATALOG_TYPE_ID,
        CATALOG_SCHEMA_ID,
        1,
        CATALOG_PROVIDER_ID,
        payload,
    )
    .map(Some)
    .map_err(SceneProjectError::SceneAsset)
}

fn catalog_codec_instantiate(
    _project: &ProjectManager,
    row: &SceneComponentAssetRecord,
    _record: &mut NodeRecord,
) -> Result<Option<(String, Value)>, SceneProjectError> {
    Ok(Some((
        CATALOG_TYPE_ID.to_owned(),
        row.decode_typed().map_err(SceneProjectError::SceneAsset)?,
    )))
}

fn catalog_codec() -> SceneComponentSerializer {
    SceneComponentSerializer {
        type_id: CATALOG_TYPE_ID,
        schema_id: CATALOG_SCHEMA_ID,
        schema_version: 1,
        provider_id: CATALOG_PROVIDER_ID,
        capture: catalog_codec_capture,
        instantiate: catalog_codec_instantiate,
    }
}

fn rollback_first_capture(
    _project: &ProjectManager,
    world: &World,
    record: &NodeRecord,
) -> Result<Option<SceneComponentAssetRecord>, SceneProjectError> {
    let Some(payload) = world.dynamic_component(record.id, "catalog_scene.rollback.first") else {
        return Ok(None);
    };
    SceneComponentAssetRecord::from_typed(
        "catalog_scene.rollback.first",
        "catalog_scene.rollback.shared.v1",
        1,
        "catalog_scene.rollback",
        payload,
    )
    .map(Some)
    .map_err(SceneProjectError::SceneAsset)
}

fn rollback_first_instantiate(
    _project: &ProjectManager,
    row: &SceneComponentAssetRecord,
    _record: &mut NodeRecord,
) -> Result<Option<(String, Value)>, SceneProjectError> {
    Ok(Some((
        "catalog_scene.rollback.first".to_owned(),
        row.decode_typed().map_err(SceneProjectError::SceneAsset)?,
    )))
}

fn rollback_second_capture(
    _project: &ProjectManager,
    world: &World,
    record: &NodeRecord,
) -> Result<Option<SceneComponentAssetRecord>, SceneProjectError> {
    let Some(payload) = world.dynamic_component(record.id, "catalog_scene.rollback.second") else {
        return Ok(None);
    };
    SceneComponentAssetRecord::from_typed(
        "catalog_scene.rollback.second",
        "catalog_scene.rollback.shared.v1",
        1,
        "catalog_scene.rollback",
        payload,
    )
    .map(Some)
    .map_err(SceneProjectError::SceneAsset)
}

fn rollback_second_instantiate(
    _project: &ProjectManager,
    row: &SceneComponentAssetRecord,
    _record: &mut NodeRecord,
) -> Result<Option<(String, Value)>, SceneProjectError> {
    Ok(Some((
        "catalog_scene.rollback.second".to_owned(),
        row.decode_typed().map_err(SceneProjectError::SceneAsset)?,
    )))
}

fn catalog_test_project(root: &std::path::Path) -> ProjectManager {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "Catalog Linked Provider Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    fs::create_dir_all(paths.asset_root(&RelPath::project_assets()).join("scenes")).unwrap();
    fs::write(
        paths
            .asset_root(&RelPath::project_assets())
            .join("scenes/main.scene.toml"),
        SceneAsset {
            entities: Vec::new(),
        }
        .to_toml_string()
        .unwrap(),
    )
    .unwrap();
    let mut project = ProjectManager::open(root).unwrap();
    project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    project.scan_and_import().unwrap();
    project
}

impl MergeTestBridge for MergeTestProvider {
    fn sample(&self) -> i32 {
        self.0
    }
}

#[test]
fn target_filtered_merge_excludes_unselected_module_owned_interfaces() {
    let mut source = RuntimeExtensionRegistry::default();
    source
        .register_module(ModuleDescriptor::new("client.runtime", "Client"))
        .unwrap();
    source
        .register_module(ModuleDescriptor::new("server.runtime", "Server"))
        .unwrap();
    let server_owner = source.intern_plugin_module("server.runtime").unwrap();
    source
        .export_interface::<dyn MergeTestBridge>(server_owner, Arc::new(MergeTestProvider(7)))
        .unwrap();

    let mut merged = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    merge_extension_registry_contributions_for_runtime_modules(
        &source,
        &HashSet::from(["client.runtime"]),
        &mut merged,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );
    merged.finalize();

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(fatal_diagnostics.is_empty(), "{fatal_diagnostics:?}");
    assert_eq!(merged.modules()[0].name, "client.runtime");
    assert!(merged
        .frozen_bridge_table()
        .resolve_slot(<dyn MergeTestBridge as PluginInterface>::INTERFACE_ID)
        .is_none());
}

#[test]
fn interface_import_binds_to_final_merged_table_and_tracks_lifecycle() {
    let mut consumer = RuntimeExtensionRegistry::default();
    let consumer_owner = consumer.intern_plugin_module("consumer.runtime").unwrap();
    let imported = consumer
        .import_interface::<dyn MergeTestBridge>(consumer_owner)
        .unwrap();
    assert_eq!(
        imported.call(MergeTestBridge::sample),
        Err(BridgeError::Absent)
    );

    let mut provider = RuntimeExtensionRegistry::default();
    let provider_owner = provider.intern_plugin_module("provider.runtime").unwrap();
    provider
        .export_interface::<dyn MergeTestBridge>(provider_owner, Arc::new(MergeTestProvider(7)))
        .unwrap();

    let mut merged = RuntimeExtensionRegistry::default();
    let merged_provider_owner = merged.intern_plugin_module("provider.runtime").unwrap();
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    merge_extension_registry_contributions(
        &consumer,
        &mut merged,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );
    merge_extension_registry_contributions(
        &provider,
        &mut merged,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(fatal_diagnostics.is_empty(), "{fatal_diagnostics:?}");

    merged.finalize();
    let table = merged.frozen_bridge_table();
    assert_eq!(imported.call(MergeTestBridge::sample), Ok(7));

    table.set_owner_enabled(merged_provider_owner, false);
    assert_eq!(
        imported.call(MergeTestBridge::sample),
        Err(BridgeError::NotEnabled)
    );

    let slot = table
        .resolve_slot(<dyn MergeTestBridge as PluginInterface>::INTERFACE_ID)
        .unwrap();
    table
        .reload_provider::<dyn MergeTestBridge>(slot, Arc::new(MergeTestProvider(11)))
        .unwrap();
    table.set_owner_enabled(merged_provider_owner, true);
    assert_eq!(imported.call(MergeTestBridge::sample), Ok(11));
    assert_eq!(table.diagnostics(slot).unwrap().not_enabled_calls, 1);

    merged.revoke_owner_registrations(merged_provider_owner);
    let current_table = merged.frozen_bridge_table();
    assert!(current_table
        .resolve_slot(<dyn MergeTestBridge as PluginInterface>::INTERFACE_ID)
        .is_none());
    table
        .reload_provider::<dyn MergeTestBridge>(slot, Arc::new(MergeTestProvider(13)))
        .unwrap();
    table.set_owner_enabled(merged_provider_owner, true);
    assert_eq!(
        imported.call(MergeTestBridge::sample),
        Err(BridgeError::Absent),
        "surviving imports must be rebound away from the revoked table"
    );
}

#[test]
fn linked_catalog_projects_scene_codec_and_revocation_listener_to_prepared_registry() {
    let mut source = RuntimeExtensionRegistry::default();
    let source_owner = source
        .intern_plugin_module("catalog_scene.runtime")
        .unwrap();
    source
        .register_component_for_owner(
            source_owner,
            ComponentTypeDescriptor::new(
                "catalog.scene.component",
                "catalog_scene",
                "Catalog scene component",
            ),
        )
        .unwrap();
    source
        .register_scene_component_codec_for_owner(
            source_owner,
            SceneComponentSerializer {
                type_id: "catalog.scene.component",
                schema_id: "catalog.scene.component.v1",
                schema_version: 1,
                provider_id: "catalog_scene",
                capture: merge_codec_capture,
                instantiate: merge_codec_instantiate,
            },
        )
        .unwrap();
    source.register_owner_revocation_listener(source_owner, |_| {});

    let mut merged = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    merge_extension_registry_contributions(
        &source,
        &mut merged,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(fatal_diagnostics.is_empty(), "{fatal_diagnostics:?}");
    let target_owner = merged
        .intern_plugin_module("catalog_scene.runtime")
        .unwrap();
    assert_eq!(merged.scene_component_codecs().count(), 1);
    assert_eq!(
        merged.scene_component_codec_owners().next(),
        Some(target_owner)
    );
    assert_eq!(
        merged.owner_revocation_listener_owners().next(),
        Some(target_owner)
    );
}

#[test]
fn linked_catalog_codec_consumes_world_and_revoke_separates_source_and_target_owners() {
    let root = std::env::temp_dir().join(format!(
        "zircon_catalog_linked_provider_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut project = catalog_test_project(&root);
    let mut source = RuntimeExtensionRegistry::default();
    let source_owner = source
        .intern_plugin_module("catalog_scene.runtime")
        .unwrap();
    source
        .register_component_for_owner(
            source_owner,
            ComponentTypeDescriptor::new(
                CATALOG_TYPE_ID,
                CATALOG_PROVIDER_ID,
                "Catalog scene component",
            ),
        )
        .unwrap();
    source
        .register_scene_component_codec_for_owner(source_owner, catalog_codec())
        .unwrap();
    let source_callbacks = Arc::new(Mutex::new(Vec::new()));
    let source_callbacks_for_listener = Arc::clone(&source_callbacks);
    source.register_owner_revocation_listener(source_owner, move |owner| {
        source_callbacks_for_listener.lock().unwrap().push(owner);
    });

    let mut merged = RuntimeExtensionRegistry::default();
    merged
        .intern_plugin_module("catalog_scene.host.runtime")
        .unwrap();
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    merge_extension_registry_contributions(
        &source,
        &mut merged,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(fatal_diagnostics.is_empty(), "{fatal_diagnostics:?}");
    let target_owner = merged
        .intern_plugin_module("catalog_scene.runtime")
        .unwrap();
    assert_ne!(source_owner.raw(), target_owner.raw());

    let before_types = project
        .scene_component_serializer_registry()
        .registered_type_ids();
    merged
        .apply_scene_component_codecs_to_project_manager(&project)
        .unwrap();
    assert!(project
        .scene_component_serializer_registry()
        .registered_type_ids()
        .contains(&CATALOG_TYPE_ID));

    let mut world = World::empty_for_project(&project).unwrap();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let payload = serde_json::json!({"source_owner": source_owner.raw(), "enabled": true});
    world
        .set_dynamic_component(entity, CATALOG_TYPE_ID, payload.clone())
        .unwrap();
    let scene = world.to_scene_asset(&project).unwrap();
    let reopened = World::from_scene_asset(&project, &scene).unwrap();
    assert_eq!(
        reopened.dynamic_component(entity, CATALOG_TYPE_ID),
        Some(&payload)
    );

    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    world.save_scene_to_project(&project, &scene_uri).unwrap();
    project.scan_and_import().unwrap();
    let source_path = project
        .existing_or_primary_project_source_path_for_uri(&scene_uri)
        .unwrap();
    let last_good_bytes = fs::read(&source_path).unwrap();
    let malformed_in_memory = {
        let mut malformed = scene.clone();
        malformed.entities[0].components[0].schema_version = 2;
        malformed
    };
    assert!(World::from_scene_asset(&project, &malformed_in_memory).is_err());
    let good_document = String::from_utf8(last_good_bytes.clone()).unwrap();
    let malformed_document = good_document.replacen("schema_version = 1", "schema_version = 2", 1);
    assert_ne!(
        malformed_document, good_document,
        "fixture must persist the component schema version"
    );
    fs::write(&source_path, malformed_document.as_bytes()).unwrap();
    project.scan_and_import().unwrap();
    assert_eq!(
        fs::read(&source_path).unwrap(),
        malformed_document.as_bytes()
    );
    assert!(World::load_scene_from_uri(&project, &scene_uri).is_err());
    fs::write(&source_path, &last_good_bytes).unwrap();
    project.scan_and_import().unwrap();

    let first = merged.revoke_owner_registrations(target_owner);
    assert!(first.components.len() >= 1);
    assert_eq!(
        source_callbacks.lock().unwrap().as_slice(),
        &[source_owner],
        "catalog-projected source callbacks retain source ownership while PM cleanup uses target ownership"
    );
    assert_eq!(
        project
            .scene_component_serializer_registry()
            .registered_type_ids(),
        before_types
    );
    assert!(merged.owner_revocation_failures().is_empty());
    assert!(world.save_scene_to_project(&project, &scene_uri).is_err());
    assert_eq!(fs::read(&source_path).unwrap(), last_good_bytes);
    drop(project);

    let mut reopened_project = ProjectManager::open(&root).unwrap();
    reopened_project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    reopened_project.scan_and_import().unwrap();
    assert!(World::load_scene_from_uri(&reopened_project, &scene_uri).is_err());

    let mut retry = RuntimeExtensionRegistry::default();
    let retry_owner = retry.intern_plugin_module("catalog_scene.runtime").unwrap();
    retry
        .register_component_for_owner(
            retry_owner,
            ComponentTypeDescriptor::new(
                CATALOG_TYPE_ID,
                CATALOG_PROVIDER_ID,
                "Catalog scene component",
            ),
        )
        .unwrap();
    retry
        .register_scene_component_codec_for_owner(retry_owner, catalog_codec())
        .unwrap();
    let mut retry_merged = RuntimeExtensionRegistry::default();
    let mut retry_diagnostics = Vec::new();
    let mut retry_fatal_diagnostics = Vec::new();
    merge_extension_registry_contributions(
        &retry,
        &mut retry_merged,
        &mut retry_diagnostics,
        &mut retry_fatal_diagnostics,
    );
    let retry_target_owner = retry_merged
        .intern_plugin_module("catalog_scene.runtime")
        .unwrap();
    retry_merged
        .apply_scene_component_codecs_to_project_manager(&reopened_project)
        .unwrap();
    assert_eq!(retry_target_owner.raw(), retry_owner.raw());
    assert_eq!(
        World::load_scene_from_uri(&reopened_project, &scene_uri)
            .unwrap()
            .dynamic_component(entity, CATALOG_TYPE_ID),
        Some(&payload)
    );
    drop(reopened_project);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn linked_catalog_panic_listener_retries_after_target_pm_cleanup() {
    let root = std::env::temp_dir().join(format!(
        "zircon_catalog_linked_panic_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let project = catalog_test_project(&root);
    let mut source = RuntimeExtensionRegistry::default();
    let source_owner = source
        .intern_plugin_module("catalog_scene.panic.runtime")
        .unwrap();
    source
        .register_component_for_owner(
            source_owner,
            ComponentTypeDescriptor::new(
                "catalog_scene.panic.component",
                "catalog_scene.panic",
                "Catalog panic component",
            ),
        )
        .unwrap();
    source
        .register_scene_component_codec_for_owner(
            source_owner,
            SceneComponentSerializer {
                type_id: "catalog_scene.panic.component",
                schema_id: "catalog_scene.panic.component.v1",
                schema_version: 1,
                provider_id: "catalog_scene.panic",
                capture: merge_codec_capture,
                instantiate: merge_codec_instantiate,
            },
        )
        .unwrap();
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_listener = Arc::clone(&attempts);
    source.register_owner_revocation_listener(source_owner, move |owner| {
        if attempts_for_listener.fetch_add(1, Ordering::AcqRel) == 0 {
            panic!("catalog source cleanup must be retryable");
        }
        assert_eq!(owner, source_owner);
    });

    let mut merged = RuntimeExtensionRegistry::default();
    merged
        .intern_plugin_module("catalog_scene.host.runtime")
        .unwrap();
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    merge_extension_registry_contributions(
        &source,
        &mut merged,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );
    let target_owner = merged
        .intern_plugin_module("catalog_scene.panic.runtime")
        .unwrap();
    assert_ne!(source_owner.raw(), target_owner.raw());
    merged
        .apply_scene_component_codecs_to_project_manager(&project)
        .unwrap();
    assert!(project
        .scene_component_serializer_registry()
        .registered_type_ids()
        .contains(&"catalog_scene.panic.component"));

    merged.revoke_owner_registrations(target_owner);
    assert_eq!(merged.owner_revocation_failures(), &[(target_owner, 1)]);
    assert!(!project
        .scene_component_serializer_registry()
        .registered_type_ids()
        .contains(&"catalog_scene.panic.component"));
    merged.revoke_owner_registrations(target_owner);
    assert!(merged.owner_revocation_failures().is_empty());
    assert_eq!(attempts.load(Ordering::Acquire), 2);

    drop(project);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn linked_catalog_batch_failure_restores_complete_extension_and_project_snapshots() {
    let root = std::env::temp_dir().join(format!(
        "zircon_catalog_linked_rollback_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let project = catalog_test_project(&root);
    let mut source = RuntimeExtensionRegistry::default();
    let first_owner = source
        .intern_plugin_module("catalog_scene.rollback.first.runtime")
        .unwrap();
    source
        .register_component_for_owner(
            first_owner,
            ComponentTypeDescriptor::new(
                "catalog_scene.rollback.first",
                "catalog_scene.rollback",
                "First rollback component",
            ),
        )
        .unwrap();
    source
        .register_scene_component_codec_for_owner(
            first_owner,
            SceneComponentSerializer {
                type_id: "catalog_scene.rollback.first",
                schema_id: "catalog_scene.rollback.shared.v1",
                schema_version: 1,
                provider_id: "catalog_scene.rollback",
                capture: rollback_first_capture,
                instantiate: rollback_first_instantiate,
            },
        )
        .unwrap();
    source.register_owner_revocation_listener(first_owner, |_| {});
    let second_owner = source
        .intern_plugin_module("catalog_scene.rollback.second.runtime")
        .unwrap();
    source
        .register_component_for_owner(
            second_owner,
            ComponentTypeDescriptor::new(
                "catalog_scene.rollback.second",
                "catalog_scene.rollback",
                "Second rollback component",
            ),
        )
        .unwrap();
    source
        .register_scene_component_codec_for_owner(
            second_owner,
            SceneComponentSerializer {
                type_id: "catalog_scene.rollback.second",
                schema_id: "catalog_scene.rollback.shared.v1",
                schema_version: 1,
                provider_id: "catalog_scene.rollback",
                capture: rollback_second_capture,
                instantiate: rollback_second_instantiate,
            },
        )
        .unwrap();
    source.register_owner_revocation_listener(second_owner, |_| {});

    let mut merged = RuntimeExtensionRegistry::default();
    merged.intern_plugin_module("host.runtime").unwrap();
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    merge_extension_registry_contributions(
        &source,
        &mut merged,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(fatal_diagnostics.is_empty(), "{fatal_diagnostics:?}");
    let extension_before = merged
        .scene_component_codecs()
        .map(|(owner, codec)| {
            (
                owner.raw(),
                codec.type_id,
                codec.schema_id,
                codec.schema_version,
                codec.provider_id,
            )
        })
        .collect::<Vec<_>>();
    let listener_before = merged
        .owner_revocation_listener_owners()
        .map(|owner| owner.raw())
        .collect::<Vec<_>>();
    let ownership_before = merged
        .scene_component_codec_owners()
        .map(|owner| (owner.raw(), format!("{:?}", merged.ownership_for(owner))))
        .collect::<Vec<_>>();
    let project_before = format!(
        "{:?}",
        project.scene_component_serializer_registry().clone()
    );
    let error = merged
        .apply_scene_component_codecs_to_project_manager(&project)
        .expect_err("duplicate schema must reject the whole linked batch");
    assert!(error
        .to_string()
        .contains("duplicate scene component schema"));
    let extension_after = merged
        .scene_component_codecs()
        .map(|(owner, codec)| {
            (
                owner.raw(),
                codec.type_id,
                codec.schema_id,
                codec.schema_version,
                codec.provider_id,
            )
        })
        .collect::<Vec<_>>();
    let listener_after = merged
        .owner_revocation_listener_owners()
        .map(|owner| owner.raw())
        .collect::<Vec<_>>();
    let ownership_after = merged
        .scene_component_codec_owners()
        .map(|owner| (owner.raw(), format!("{:?}", merged.ownership_for(owner))))
        .collect::<Vec<_>>();
    assert_eq!(extension_after, extension_before);
    assert_eq!(listener_after, listener_before);
    assert_eq!(ownership_after, ownership_before);
    assert_eq!(
        format!(
            "{:?}",
            project.scene_component_serializer_registry().clone()
        ),
        project_before
    );

    drop(project);
    let _ = fs::remove_dir_all(root);
}
