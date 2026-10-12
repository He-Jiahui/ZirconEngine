//! Core-owner mount fragment for the normal catalog -> RuntimePreparedProject ingress.
//!
//! This file is intentionally unmounted in the Asset-owned candidate.  The Core owner should
//! mount it from dynamic_api/session/tests/mod.rs so pub(super) RuntimeDynamicSession fields
//! remain private and no production visibility changes are needed.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::json;

use crate::{
    asset::{
        assets::SceneAsset,
        project::{ProjectManager, ProjectManifest, ProjectPaths},
        project_asset_manager_handle, AssetUri,
    },
    builtin::RuntimePluginId,
    core::{
        framework::{
            platform::RuntimeTargetMode, project::ProjectPluginManifest,
            scene::ComponentTypeDescriptor,
        },
        manager::resolve_manager_service,
    },
    plugin::{
        PluginModuleId, RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginCatalog,
        RuntimePluginDescriptor, RuntimePluginRegistrationReport,
    },
    scene::{
        components::{NodeKind, NodeRecord},
        world::{SceneComponentSerializer, SceneProjectError, World},
    },
};
use zircon_runtime_interface::project::RelPath;

use super::super::{RuntimeDynamicSession, RuntimeDynamicSessionProfile, RuntimeProjectConfig};

const TYPE_ID: &str = "catalog_prepared.scene.component";
const SCHEMA_ID: &str = "catalog_prepared.scene.component.v1";
const PROVIDER_ID: &str = "catalog_prepared_scene";
const BASE_PACKAGE_ID: &str = "catalog_prepared_base";
const SCENE_PACKAGE_ID: &str = "catalog_prepared_scene";
const SCENE_URI: &str = "res://scenes/main.scene.toml";

fn temporary_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("zircon-prepared-catalog-{stamp}"))
}

fn serializer() -> SceneComponentSerializer {
    SceneComponentSerializer {
        type_id: TYPE_ID,
        schema_id: SCHEMA_ID,
        schema_version: 1,
        provider_id: PROVIDER_ID,
        capture,
        instantiate,
    }
}

fn capture(
    _project: &ProjectManager,
    world: &World,
    record: &NodeRecord,
) -> Result<Option<crate::asset::assets::SceneComponentAssetRecord>, SceneProjectError> {
    let Some(payload) = world.dynamic_component(record.id, TYPE_ID) else {
        return Ok(None);
    };
    crate::asset::assets::SceneComponentAssetRecord::from_typed(
        TYPE_ID,
        SCHEMA_ID,
        1,
        PROVIDER_ID,
        payload,
    )
    .map(Some)
    .map_err(SceneProjectError::SceneAsset)
}

fn instantiate(
    _project: &ProjectManager,
    row: &crate::asset::assets::SceneComponentAssetRecord,
    _record: &mut NodeRecord,
) -> Result<Option<(String, serde_json::Value)>, SceneProjectError> {
    Ok(Some((
        TYPE_ID.to_owned(),
        row.decode_typed().map_err(SceneProjectError::SceneAsset)?,
    )))
}

struct BasePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl BasePlugin {
    fn new() -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                BASE_PACKAGE_ID,
                "Prepared catalog base",
                RuntimePluginId::new(BASE_PACKAGE_ID),
                "zircon_fixture_prepared_catalog_base",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .build(),
        }
    }
}

impl RuntimePlugin for BasePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }
}

struct ScenePlugin {
    descriptor: RuntimePluginDescriptor,
    source_owner: Arc<Mutex<Option<PluginModuleId>>>,
    callbacks: Arc<Mutex<Vec<PluginModuleId>>>,
}

impl ScenePlugin {
    fn new(
        source_owner: Arc<Mutex<Option<PluginModuleId>>>,
        callbacks: Arc<Mutex<Vec<PluginModuleId>>>,
    ) -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                SCENE_PACKAGE_ID,
                "Prepared catalog scene",
                RuntimePluginId::new(SCENE_PACKAGE_ID),
                "zircon_fixture_prepared_catalog_scene",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_module_dependency(crate::core::ModuleDependencySpec::named(
                "catalog_prepared_base.runtime",
            ))
            .build(),
            source_owner,
            callbacks,
        }
    }
}

impl RuntimePlugin for ScenePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), crate::plugin::RuntimeExtensionRegistryError> {
        let owner = registry.intern_plugin_module("catalog_prepared_scene.runtime")?;
        *self.source_owner.lock().expect("owner lock") = Some(owner);
        registry.register_component_for_owner(
            owner,
            ComponentTypeDescriptor::new(TYPE_ID, PROVIDER_ID, "Prepared catalog scene component"),
        )?;
        registry.register_scene_component_codec_for_owner(owner, serializer())?;
        let callbacks = Arc::clone(&self.callbacks);
        registry.register_owner_revocation_listener(owner, move |revoked| {
            callbacks.lock().expect("callback lock").push(revoked);
        });
        Ok(())
    }
}

fn write_project(root: &Path, plugins: ProjectPluginManifest) {
    let paths = ProjectPaths::from_root(root).expect("project paths");
    paths
        .ensure_layout(&[RelPath::project_assets()])
        .expect("project layout");
    let mut manifest = ProjectManifest::new(
        "prepared-catalog-consumer",
        AssetUri::parse(SCENE_URI).expect("scene URI"),
        1,
    );
    manifest.plugins = plugins;
    manifest.save(paths.manifest_path()).expect("manifest save");
    let scene_path = paths
        .asset_root(&RelPath::project_assets())
        .join("scenes/main.scene.toml");
    fs::create_dir_all(scene_path.parent().expect("scene parent")).expect("scene directory");
    fs::write(
        scene_path,
        SceneAsset {
            entities: Vec::new(),
        }
        .to_toml_string()
        .expect("empty scene"),
    )
    .expect("scene write");
}

#[test]
fn linked_catalog_uses_runtime_prepared_project_before_pm_drop_and_revoke() {
    let root = temporary_root();
    let source_owner = Arc::new(Mutex::new(None));
    let callbacks = Arc::new(Mutex::new(Vec::new()));
    let base = BasePlugin::new();
    let scene = ScenePlugin::new(Arc::clone(&source_owner), Arc::clone(&callbacks));
    let catalog = RuntimePluginCatalog::from_plugins([
        &base as &dyn RuntimePlugin,
        &scene as &dyn RuntimePlugin,
    ]);
    assert!(catalog.is_success(), "{:?}", catalog.diagnostics());
    write_project(&root, catalog.project_manifest());

    let config = RuntimeProjectConfig::from_root(&root).expect("project config");
    let reports = vec![
        RuntimePluginRegistrationReport::from_plugin(&base),
        RuntimePluginRegistrationReport::from_plugin(&scene),
    ];
    let mut session = RuntimeDynamicSession::new_with_linked_plugins(
        RuntimeDynamicSessionProfile::Minimal,
        Some(config),
        reports,
    )
    .expect("minimal linked session");

    let source_owner_id = source_owner
        .lock()
        .expect("owner lock")
        .expect("source owner");
    let mut extensions = session._runtime_extension_registry.clone();
    let target_owner = extensions
        .scene_component_codec_owners()
        .next()
        .expect("prepared target owner");
    assert_ne!(source_owner_id.raw(), target_owner.raw());

    let core = session.runtime.handle();
    let asset_handle = project_asset_manager_handle(&core).expect("asset manager handle");
    let asset_manager = resolve_manager_service(&core, asset_handle).expect("asset manager");
    let mut project = asset_manager
        .current_project_manager()
        .expect("prepared ProjectManager");
    assert!(project
        .scene_component_serializer_registry()
        .registered_type_ids()
        .contains(&TYPE_ID));

    let mut world = World::empty_for_project(&project).expect("empty world");
    let entity = world.spawn_node(NodeKind::Empty).expect("empty node");
    let payload = json!({ "source_owner": source_owner_id.raw(), "enabled": true });
    world
        .set_dynamic_component(entity, TYPE_ID, payload.clone())
        .expect("dynamic component");
    let scene = world.to_scene_asset(&project).expect("scene capture");
    assert_eq!(
        World::from_scene_asset(&project, &scene)
            .expect("scene restore")
            .dynamic_component(entity, TYPE_ID),
        Some(&payload)
    );
    let scene_uri = AssetUri::parse(SCENE_URI).expect("scene URI");
    world
        .save_scene_to_project(&project, &scene_uri)
        .expect("prepared scene save");
    project.scan_and_import().expect("prepared scene import");
    assert_eq!(
        World::load_scene_from_uri(&project, &scene_uri)
            .expect("prepared scene load")
            .dynamic_component(entity, TYPE_ID),
        Some(&payload)
    );

    let registry_weak = project.scene_component_registry_weak();
    drop(project);
    drop(asset_manager);
    drop(core);
    drop(session);
    assert!(registry_weak.upgrade().is_none(), "prepared PM must drop");

    let removed = extensions.revoke_owner_registrations(target_owner);
    assert!(!removed.components.is_empty());
    assert_eq!(
        callbacks.lock().expect("callback lock").as_slice(),
        &[source_owner_id]
    );
    assert!(extensions.owner_revocation_failures().is_empty());

    fs::remove_dir_all(root).expect("fixture cleanup");
}
