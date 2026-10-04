use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    NodeRecord, ProjectManager, SceneComponentAssetRecord, SceneComponentSerializer,
    SceneComponentSerializerRegistry, SceneProjectError, World,
};
use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::{AssetReference, AssetUri, ImportedAsset, SceneAsset};
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::core::resource::ResourceLocator;
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::components::NodeKind;
use serde_json::Value;
use zircon_runtime_interface::project::RelPath;

const THIRD_TYPE_ID: &str = "tests.scene.provider.component_probe";
const THIRD_SCHEMA_ID: &str = "tests.scene.provider.component_probe.v1";
const THIRD_PROVIDER_ID: &str = "tests.scene.provider";

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_scene_{label}_{unique}"))
}

fn create_test_project(root: &Path) -> ProjectManager {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "SceneGenericProviderSandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let assets = paths.asset_root(&RelPath::project_assets());
    fs::create_dir_all(assets.join("models")).unwrap();
    fs::create_dir_all(assets.join("scenes")).unwrap();
    fs::write(
        assets.join("models/triangle.obj"),
        "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nvt 0.0 0.0\nvt 1.0 0.0\nvt 0.0 1.0\nvn 0.0 0.0 1.0\nf 1/1/1 2/2/1 3/3/1\n",
    )
    .unwrap();
    fs::write(
        assets.join("scenes/main.scene.toml"),
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

fn capture_third_component(
    _project: &ProjectManager,
    world: &World,
    record: &NodeRecord,
) -> Result<Option<SceneComponentAssetRecord>, SceneProjectError> {
    let Some(payload) = world.dynamic_component(record.id, THIRD_TYPE_ID) else {
        return Ok(None);
    };
    SceneComponentAssetRecord::from_typed(
        THIRD_TYPE_ID,
        THIRD_SCHEMA_ID,
        1,
        THIRD_PROVIDER_ID,
        payload,
    )
    .map(Some)
    .map_err(SceneProjectError::SceneAsset)
}

fn instantiate_third_component(
    _project: &ProjectManager,
    row: &SceneComponentAssetRecord,
    _record: &mut NodeRecord,
) -> Result<Option<(String, Value)>, SceneProjectError> {
    let payload = row.decode_typed().map_err(SceneProjectError::SceneAsset)?;
    Ok(Some((THIRD_TYPE_ID.to_owned(), payload)))
}

fn third_component_serializer() -> SceneComponentSerializer {
    SceneComponentSerializer {
        type_id: THIRD_TYPE_ID,
        schema_id: THIRD_SCHEMA_ID,
        schema_version: 1,
        provider_id: THIRD_PROVIDER_ID,
        capture: capture_third_component,
        instantiate: instantiate_third_component,
    }
}

fn register_third_provider(
    project: &ProjectManager,
) -> Result<
    (RuntimeExtensionRegistry, crate::plugin::PluginModuleId),
    crate::plugin::RuntimeExtensionRegistryError,
> {
    let mut extensions = RuntimeExtensionRegistry::default();
    let owner = extensions.intern_plugin_module("tests.scene.provider.runtime")?;
    extensions.register_component_for_owner(
        owner,
        ComponentTypeDescriptor::new(THIRD_TYPE_ID, THIRD_PROVIDER_ID, "Third probe"),
    )?;
    extensions.register_scene_component_codec_for_owner(owner, third_component_serializer())?;
    extensions.apply_scene_component_codecs_to_project_manager(project)?;
    Ok((extensions, owner))
}

#[test]
fn builtin_scene_component_registry_routes_rows_by_stable_identity() {
    assert_eq!(
        SceneComponentSerializerRegistry::builtin().registered_type_ids(),
        vec!["zircon.render2d.sprite", "zircon.render2d.mesh"]
    );
}

#[test]
fn registry_rejects_unknown_and_mismatched_component_rows_before_decode() {
    let registry = SceneComponentSerializerRegistry::builtin();
    let unknown = SceneComponentAssetRecord {
        type_id: "tests.scene.third_component".into(),
        schema_id: "tests.scene.third_component.v1".into(),
        schema_version: 1,
        provider_id: "tests.scene.registry".into(),
        payload: serde_json::json!({"enabled": true}),
        references: Vec::new(),
    };
    assert!(registry.validate_rows(&[unknown]).is_err());

    let mismatched = SceneComponentAssetRecord {
        type_id: "zircon.render2d.sprite".into(),
        schema_id: "zircon.render2d.sprite.v2".into(),
        schema_version: 2,
        provider_id: "foreign.provider".into(),
        payload: serde_json::json!({}),
        references: Vec::new(),
    };
    assert!(registry.validate_rows(&[mismatched]).is_err());
}

#[test]
fn registry_roundtrips_a_third_provider_row_without_changing_dispatch_code() {
    let mut registry = SceneComponentSerializerRegistry::builtin();
    registry
        .register_provider(
            SceneComponentSerializer {
                type_id: THIRD_TYPE_ID,
                schema_id: THIRD_SCHEMA_ID,
                schema_version: 1,
                provider_id: THIRD_PROVIDER_ID,
                capture: capture_third_component,
                instantiate: instantiate_third_component,
            },
            ComponentTypeDescriptor::new(THIRD_TYPE_ID, THIRD_PROVIDER_ID, "Third probe"),
        )
        .unwrap();
    let row = SceneComponentAssetRecord::from_typed(
        THIRD_TYPE_ID,
        THIRD_SCHEMA_ID,
        1,
        THIRD_PROVIDER_ID,
        &serde_json::json!({"enabled": true, "weight": 0.75}),
    )
    .unwrap();
    registry.validate_rows(std::slice::from_ref(&row)).unwrap();
    assert_eq!(
        registry.registered_type_ids(),
        vec![
            "zircon.render2d.sprite",
            "zircon.render2d.mesh",
            THIRD_TYPE_ID
        ]
    );
    assert_eq!(
        row.decode_typed::<Value>().unwrap(),
        serde_json::json!({"enabled": true, "weight": 0.75})
    );
}

#[test]
fn third_provider_roundtrips_through_world_save_and_reopen_with_references() {
    let root = unique_temp_project_root("scene_generic_provider_reopen");
    let mut project = create_test_project(&root);
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let model_uri = AssetUri::parse("res://models/triangle.obj").unwrap();
    let model_entry = project
        .asset_registry()
        .entry_by_path(&model_uri)
        .expect("fixture model should be registered");
    let model_reference = AssetReference::new(model_entry.uuid(), model_uri.clone());

    let (mut extensions, owner) = register_third_provider(&project).unwrap();

    let mut world = World::empty_for_project(&project).unwrap();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let original_payload = serde_json::json!({
        "enabled": true,
        "weight": 0.75,
        "asset": model_reference.clone(),
    });
    world
        .set_dynamic_component(entity, THIRD_TYPE_ID, original_payload.clone())
        .unwrap();

    world.save_scene_to_project(&project, &scene_uri).unwrap();
    project.scan_and_import().unwrap();
    let saved = project
        .load_artifact(&scene_uri)
        .expect("saved scene artifact should load");
    let ImportedAsset::Scene(saved_scene) = saved else {
        panic!("saved document should remain a scene");
    };
    let saved_row = saved_scene
        .entities
        .iter()
        .flat_map(|entity| entity.components.iter())
        .find(|row| row.type_id == THIRD_TYPE_ID)
        .expect("third provider row should be persisted");
    assert_eq!(saved_row.provider_id, THIRD_PROVIDER_ID);
    assert_eq!(saved_row.schema_id, THIRD_SCHEMA_ID);
    assert_eq!(saved_row.references, vec![model_reference.clone()]);

    extensions.revoke_owner_registrations(owner);
    assert!(World::load_scene_from_uri(&project, &scene_uri).is_err());

    drop(project);

    let mut reopened_project = ProjectManager::open(&root).unwrap();
    reopened_project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    reopened_project.scan_and_import().unwrap();
    assert!(World::load_scene_from_uri(&reopened_project, &scene_uri).is_err());
    let (_reopened_extensions, _reopened_owner) =
        register_third_provider(&reopened_project).unwrap();
    let reopened = World::load_scene_from_uri(&reopened_project, &scene_uri).unwrap();
    assert_eq!(
        reopened.dynamic_component(entity, THIRD_TYPE_ID),
        Some(&original_payload)
    );

    drop(reopened_project);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn mixed_provider_and_render2d_rows_roundtrip_with_parent_identity_after_reopen() {
    let root = unique_temp_project_root("scene_mixed_provider_render2d_reopen");
    let mut project = create_test_project(&root);
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let (extensions, _owner) = register_third_provider(&project).unwrap();
    let mut world = World::empty_for_project(&project).unwrap();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let child_entity = world.spawn_node(NodeKind::Empty).unwrap();
    let payload = serde_json::json!({"enabled": true, "identity": "parent"});
    world
        .set_dynamic_component(parent, THIRD_TYPE_ID, payload.clone())
        .unwrap();
    let mut scene = world.to_scene_asset(&project).unwrap();
    let child_index = scene
        .entities
        .iter()
        .position(|entity| entity.entity == child_entity)
        .expect("the second World entity must be represented in the scene asset");
    let mut child = scene.entities.remove(child_index);
    child.name = "render2d-child".to_owned();
    child.parent = Some(parent);
    child.components = vec![SceneComponentAssetRecord::from_typed(
        super::MESH_TYPE_ID,
        super::MESH_SCHEMA_ID,
        super::SCHEMA_VERSION,
        super::BUILTIN_PROVIDER_ID,
        &crate::asset::SceneMesh2dAsset {
            model: AssetReference::from_locator(ResourceLocator::parse("builtin://quad").unwrap()),
            material: AssetReference::from_locator(
                ResourceLocator::parse("builtin://material/default").unwrap(),
            ),
            color: [0.25, 0.5, 1.0, 1.0],
            z_order: 7,
            material_alpha_mode: crate::core::framework::render::RenderMaterialAlphaMode::Opaque,
        },
    )
    .unwrap()];
    scene.entities.push(child);

    let mixed_world = World::from_scene_asset(&project, &scene).unwrap();
    mixed_world
        .save_scene_to_project(&project, &scene_uri)
        .unwrap();
    project.scan_and_import().unwrap();
    drop(extensions);
    drop(project);

    let mut reopened_project = ProjectManager::open(&root).unwrap();
    reopened_project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    reopened_project.scan_and_import().unwrap();
    let (_reopened_extensions, _reopened_owner) =
        register_third_provider(&reopened_project).unwrap();
    let reopened = World::load_scene_from_uri(&reopened_project, &scene_uri).unwrap();
    assert_eq!(
        reopened.dynamic_component(parent, THIRD_TYPE_ID),
        Some(&payload)
    );
    let reopened_scene = reopened.to_scene_asset(&reopened_project).unwrap();
    let child = reopened_scene
        .entities
        .iter()
        .find(|entity| entity.entity == child_entity)
        .expect("mixed scene child identity must survive reopen");
    assert_eq!(child.parent, Some(parent));
    assert_eq!(
        child.components[0].type_id,
        super::MESH_TYPE_ID,
        "the built-in 2D codec must remain alongside the generic provider row"
    );

    drop(reopened_project);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn world_load_rejects_unknown_duplicate_and_mismatched_rows_before_mutation() {
    let root = unique_temp_project_root("scene_generic_provider_rejection");
    let project = create_test_project(&root);
    let (_extensions, _owner) = register_third_provider(&project).unwrap();
    let mut world = World::empty_for_project(&project).unwrap();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let mut scene = world.to_scene_asset(&project).unwrap();
    let unknown = SceneComponentAssetRecord {
        type_id: "tests.unknown".into(),
        schema_id: "tests.unknown.v1".into(),
        schema_version: 1,
        provider_id: "tests.provider".into(),
        payload: serde_json::json!({}),
        references: Vec::new(),
    };
    scene.entities[0].components.push(unknown);
    assert!(World::from_scene_asset(&project, &scene).is_err());

    scene.entities[0].components.clear();
    let row = SceneComponentAssetRecord::from_typed(
        THIRD_TYPE_ID,
        THIRD_SCHEMA_ID,
        1,
        THIRD_PROVIDER_ID,
        &serde_json::json!({"enabled": true}),
    )
    .unwrap();
    scene.entities[0].components = vec![row.clone(), row];
    assert!(World::from_scene_asset(&project, &scene).is_err());

    scene.entities[0].components = vec![SceneComponentAssetRecord {
        type_id: THIRD_TYPE_ID.into(),
        schema_id: "tests.scene.component.probe.v2".into(),
        schema_version: 2,
        provider_id: THIRD_PROVIDER_ID.into(),
        payload: serde_json::json!({}),
        references: Vec::new(),
    }];
    assert!(World::from_scene_asset(&project, &scene).is_err());
    assert!(world.contains_entity(entity));

    drop(project);
    let _ = std::fs::remove_dir_all(root);
}
#[test]
fn owner_qualified_codec_registration_rolls_back_both_registries() {
    let root = unique_temp_project_root("scene_provider_atomic_registration");
    let project = create_test_project(&root);
    let (mut first, first_owner) = register_third_provider(&project).unwrap();
    let project_registry_before = project
        .scene_component_serializer_registry()
        .registered_type_ids();

    let mut retry = RuntimeExtensionRegistry::default();
    let retry_owner = retry
        .intern_plugin_module("tests.scene.provider.retry.runtime")
        .unwrap();
    let retry_type = "tests.scene.provider.retry.component";
    let retry_provider = "tests.scene.provider.retry";
    let retry_descriptor = ComponentTypeDescriptor::new(retry_type, retry_provider, "Retry probe");
    let retry_serializer = SceneComponentSerializer {
        type_id: retry_type,
        schema_id: THIRD_SCHEMA_ID,
        schema_version: 1,
        provider_id: retry_provider,
        capture: capture_third_component,
        instantiate: instantiate_third_component,
    };

    let error = retry
        .register_scene_component_provider_for_project_manager(
            &project,
            retry_descriptor.clone(),
            retry_serializer,
        )
        .expect_err("duplicate PM schema must reject before publication");
    assert!(error
        .to_string()
        .contains("duplicate scene component schema"));
    assert!(retry.ownership_for(retry_owner).components.is_empty());
    assert_eq!(
        project
            .scene_component_serializer_registry()
            .registered_type_ids(),
        project_registry_before
    );

    first.revoke_owner_registrations(first_owner);
    retry
        .register_scene_component_provider_for_project_manager(
            &project,
            retry_descriptor,
            retry_serializer,
        )
        .expect("the same owner registration must succeed after the old schema is revoked");
    assert!(!retry.ownership_for(retry_owner).components.is_empty());

    drop(project);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn revoked_live_dynamic_rows_reject_save_without_changing_previous_bytes() {
    let root = unique_temp_project_root("scene_provider_revoked_save");
    let mut project = create_test_project(&root);
    let (mut extensions, owner) = register_third_provider(&project).unwrap();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let mut world = World::empty_for_project(&project).unwrap();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .set_dynamic_component(entity, THIRD_TYPE_ID, serde_json::json!({"enabled": true}))
        .unwrap();
    world.save_scene_to_project(&project, &scene_uri).unwrap();
    let path = project
        .existing_or_primary_project_source_path_for_uri(&scene_uri)
        .unwrap();
    let previous_bytes = std::fs::read(&path).unwrap();

    extensions.revoke_owner_registrations(owner);
    assert!(world.save_scene_to_project(&project, &scene_uri).is_err());
    assert_eq!(std::fs::read(path).unwrap(), previous_bytes);

    let (_retry_extensions, _retry_owner) = register_third_provider(&project).unwrap();
    world
        .save_scene_to_project(&project, &scene_uri)
        .expect("a re-admitted owner can retry the failed save");
    project.scan_and_import().unwrap();
    let reopened = World::load_scene_from_uri(&project, &scene_uri).unwrap();
    assert_eq!(
        reopened.dynamic_component(entity, THIRD_TYPE_ID),
        Some(&serde_json::json!({"enabled": true}))
    );

    drop(project);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn project_manager_drop_releases_registry_when_owner_listener_is_weak() {
    let root = unique_temp_project_root("scene_provider_weak_drop");
    let project = create_test_project(&root);
    let (extensions, owner) = register_third_provider(&project).unwrap();
    let weak = project.scene_component_registry_weak();
    assert!(weak.upgrade().is_some());
    drop(project);
    assert!(weak.upgrade().is_none());
    drop(extensions);
    let _ = owner;
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_revocation_panic_cleans_rows_and_retries_retained_listener() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let mut extensions = RuntimeExtensionRegistry::default();
    let owner = extensions
        .intern_plugin_module("tests.scene.provider.panic.runtime")
        .unwrap();
    extensions
        .register_component_for_owner(
            owner,
            ComponentTypeDescriptor::new(
                "tests.scene.provider.panic.component",
                "tests.scene.provider.panic",
                "Panic provider",
            ),
        )
        .unwrap();
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_listener = Arc::clone(&attempts);
    extensions.register_owner_revocation_listener(owner, move |_| {
        if attempts_for_listener.fetch_add(1, Ordering::AcqRel) == 0 {
            panic!("first owner cleanup attempt deliberately fails");
        }
    });

    let removed = extensions.revoke_owner_registrations(owner);
    assert_eq!(removed.components.len(), 1);
    assert_eq!(extensions.owner_revocation_failures(), &[(owner, 1)]);
    assert!(extensions.ownership_for(owner).is_empty());

    extensions.revoke_owner_registrations(owner);
    assert!(extensions.owner_revocation_failures().is_empty());
    assert_eq!(attempts.load(Ordering::Acquire), 2);
}

#[test]
#[ignore = "representative 20K generic-row budget oracle; execute in the product perf lane"]
fn generic_provider_20k_row_validation_stays_within_persistence_budget() {
    let mut registry = SceneComponentSerializerRegistry::builtin();
    registry
        .register_provider(
            third_component_serializer(),
            ComponentTypeDescriptor::new(THIRD_TYPE_ID, THIRD_PROVIDER_ID, "Third probe"),
        )
        .unwrap();
    let rows = (0..20_000)
        .map(|index| {
            SceneComponentAssetRecord::from_typed(
                THIRD_TYPE_ID,
                THIRD_SCHEMA_ID,
                1,
                THIRD_PROVIDER_ID,
                &serde_json::json!({"index": index, "enabled": true}),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let started = std::time::Instant::now();
    for row in &rows {
        registry.validate_rows(std::slice::from_ref(row)).unwrap();
    }
    assert!(
        started.elapsed() < std::time::Duration::from_secs(2),
        "20K generic rows exceeded the persistence validation budget"
    );
}
