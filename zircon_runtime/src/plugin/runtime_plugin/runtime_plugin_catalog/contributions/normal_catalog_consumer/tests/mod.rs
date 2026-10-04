//! Normal catalog consumer coverage for the linked scene-component lifecycle.
//!
//! This module intentionally enters through RuntimePluginCatalog::from_plugins,
//! project-manifest selection, and CompiledProjectPluginPlan before applying the
//! resulting extension registry to a real ProjectManager.  It keeps the source
//! callback owner separate from the target ProjectManager cleanup owner.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::json;

use crate::{
    asset::{
        project::{ProjectManager, ProjectManifest, ProjectPaths},
        AssetUri,
    },
    builtin::RuntimePluginId,
    core::{
        framework::{platform::RuntimeTargetMode, scene::ComponentTypeDescriptor},
        ModuleDependencySpec,
    },
    plugin::{
        PluginModuleId, RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginCatalog,
        RuntimePluginDescriptor,
    },
    scene::{
        components::{NodeKind, NodeRecord},
        world::{SceneComponentSerializer, World},
    },
};

const TYPE_ID: &str = "catalog_normal.scene.component";
const SCHEMA_ID: &str = "catalog_normal.scene.component.v1";
const PROVIDER_ID: &str = "catalog_normal_scene";
const BASE_PACKAGE_ID: &str = "catalog_normal_base";
const SCENE_PACKAGE_ID: &str = "catalog_normal_scene";
const SCENE_URI: &str = "res://scenes/main.scene.toml";

fn temp_project_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("zircon-{label}-{stamp}"))
}

fn scene_component_serializer() -> SceneComponentSerializer {
    SceneComponentSerializer {
        type_id: TYPE_ID,
        schema_id: SCHEMA_ID,
        schema_version: 1,
        provider_id: PROVIDER_ID,
        capture: normal_codec_capture,
        instantiate: normal_codec_instantiate,
    }
}

fn normal_codec_capture(
    _project: &ProjectManager,
    world: &World,
    record: &NodeRecord,
) -> Result<
    Option<crate::asset::assets::SceneComponentAssetRecord>,
    crate::scene::world::SceneProjectError,
> {
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
    .map_err(crate::scene::world::SceneProjectError::SceneAsset)
}

fn normal_codec_instantiate(
    _project: &ProjectManager,
    record: &crate::asset::assets::SceneComponentAssetRecord,
    _node: &mut NodeRecord,
) -> Result<Option<(String, serde_json::Value)>, crate::scene::world::SceneProjectError> {
    Ok(Some((
        TYPE_ID.to_owned(),
        record
            .decode_typed()
            .map_err(crate::scene::world::SceneProjectError::SceneAsset)?,
    )))
}

struct BaseCatalogPlugin {
    descriptor: RuntimePluginDescriptor,
}

impl BaseCatalogPlugin {
    fn new() -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                BASE_PACKAGE_ID,
                "Normal catalog base",
                RuntimePluginId::new(BASE_PACKAGE_ID),
                "zircon_fixture_catalog_normal_base",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .build(),
        }
    }
}

impl RuntimePlugin for BaseCatalogPlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }
}

struct SceneCatalogPlugin {
    descriptor: RuntimePluginDescriptor,
    source_owner: Arc<Mutex<Option<PluginModuleId>>>,
    source_callbacks: Arc<Mutex<Vec<PluginModuleId>>>,
}

impl SceneCatalogPlugin {
    fn new(
        source_owner: Arc<Mutex<Option<PluginModuleId>>>,
        source_callbacks: Arc<Mutex<Vec<PluginModuleId>>>,
    ) -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                SCENE_PACKAGE_ID,
                "Normal catalog scene",
                RuntimePluginId::new(SCENE_PACKAGE_ID),
                "zircon_fixture_catalog_normal_scene",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_module_dependency(ModuleDependencySpec::named("catalog_normal_base.runtime"))
            .build(),
            source_owner,
            source_callbacks,
        }
    }
}

impl RuntimePlugin for SceneCatalogPlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), crate::plugin::RuntimeExtensionRegistryError> {
        let owner = registry.intern_plugin_module("catalog_normal_scene.runtime")?;
        *self
            .source_owner
            .lock()
            .expect("source owner lock poisoned") = Some(owner);
        registry.register_component_for_owner(
            owner,
            ComponentTypeDescriptor::new(TYPE_ID, PROVIDER_ID, "Normal catalog scene component"),
        )?;
        registry.register_scene_component_codec_for_owner(owner, scene_component_serializer())?;
        let callbacks = Arc::clone(&self.source_callbacks);
        registry.register_owner_revocation_listener(owner, move |revoked| {
            callbacks
                .lock()
                .expect("source callback lock poisoned")
                .push(revoked);
        });
        Ok(())
    }
}

fn write_project_manifest(
    root: &Path,
    plugins: crate::core::framework::project::ProjectPluginManifest,
) {
    let paths = ProjectPaths::from_root(root).expect("project paths");
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .expect("project layout");
    let mut manifest = ProjectManifest::new(
        "normal-catalog-consumer",
        AssetUri::parse(SCENE_URI).expect("project URI"),
        1,
    );
    manifest.plugins = plugins;
    manifest
        .save(paths.manifest_path())
        .expect("selected manifest save");
    let asset_root =
        paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    let scene_path = asset_root.join("scenes/main.scene.toml");
    if !scene_path.exists() {
        fs::create_dir_all(asset_root.join("scenes")).expect("scene directory");
        fs::write(
            scene_path,
            crate::asset::assets::SceneAsset {
                entities: Vec::new(),
            }
            .to_toml_string()
            .expect("empty scene"),
        )
        .expect("empty scene write");
    }
}

fn open_project(
    root: &Path,
    plugins: crate::core::framework::project::ProjectPluginManifest,
) -> ProjectManager {
    write_project_manifest(root, plugins);
    let mut project = ProjectManager::open(root).expect("project open");
    project
        .register_first_wave_plugin_fixture_importers_for_test()
        .expect("fixture importers");
    project.scan_and_import().expect("initial asset scan");
    project
}

#[test]
fn normal_catalog_compiled_plan_consumes_scene_codec_after_project_drop_and_retry() {
    let root = temp_project_root("normal-catalog");
    let source_owner = Arc::new(Mutex::new(None));
    let source_callbacks = Arc::new(Mutex::new(Vec::new()));
    let base = BaseCatalogPlugin::new();
    let scene = SceneCatalogPlugin::new(Arc::clone(&source_owner), Arc::clone(&source_callbacks));

    // This is the production catalog ingress; it internally creates each
    // RuntimePluginRegistrationReport through the public from_plugin path.
    let catalog = RuntimePluginCatalog::from_plugins([
        &base as &dyn RuntimePlugin,
        &scene as &dyn RuntimePlugin,
    ]);
    assert!(catalog.is_success(), "{:?}", catalog.diagnostics());
    let selected_manifest = catalog.project_manifest();
    assert!(selected_manifest
        .selections
        .iter()
        .any(|selection| selection.id == SCENE_PACKAGE_ID));

    let mut project = open_project(&root, selected_manifest.clone());
    let selected = project.manifest().plugins.clone();
    let plan = catalog.compiled_project_plan(&selected, RuntimeTargetMode::ClientRuntime);
    assert!(plan
        .linked_provider_package_ids()
        .iter()
        .any(|package_id| package_id == SCENE_PACKAGE_ID));
    assert!(
        plan.runtime_extensions().fatal_diagnostics.is_empty(),
        "{:?}",
        plan.runtime_extensions().fatal_diagnostics
    );

    let source_owner_id = source_owner
        .lock()
        .expect("source owner lock poisoned")
        .expect("catalog source owner");
    let mut extensions = plan.runtime_extensions().registry.clone();
    let target_owner = extensions
        .scene_component_codec_owners()
        .next()
        .expect("compiled plan target owner");
    assert_ne!(
        source_owner_id.raw(),
        target_owner.raw(),
        "catalog merge must keep source callback and target PM owners distinct"
    );

    let before_type_ids = project
        .scene_component_serializer_registry()
        .registered_type_ids();
    extensions
        .apply_scene_component_codecs_to_project_manager(&project)
        .expect("compiled plan codec application");
    let after_type_ids = project
        .scene_component_serializer_registry()
        .registered_type_ids();
    assert!(after_type_ids.len() > before_type_ids.len());
    assert!(after_type_ids.contains(&TYPE_ID));

    let mut world = World::empty_for_project(&project).expect("empty world");
    let entity = world.spawn_node(NodeKind::Empty).expect("empty node");
    let payload = json!({ "source_owner": source_owner_id.raw(), "enabled": true });
    world
        .set_dynamic_component(entity, TYPE_ID, payload.clone())
        .expect("dynamic component");
    let scene_asset = world.to_scene_asset(&project).expect("scene capture");
    let restored = World::from_scene_asset(&project, &scene_asset).expect("scene restore");
    assert_eq!(restored.dynamic_component(entity, TYPE_ID), Some(&payload));
    let scene_uri = AssetUri::parse(SCENE_URI).expect("scene URI");
    world
        .save_scene_to_project(&project, &scene_uri)
        .expect("scene save");
    project.scan_and_import().expect("scene import");
    let loaded = World::load_scene_from_uri(&project, &scene_uri).expect("scene load");
    assert_eq!(loaded.dynamic_component(entity, TYPE_ID), Some(&payload));

    let project_registry_weak = project.scene_component_registry_weak();
    drop(project);
    assert!(
        project_registry_weak.upgrade().is_none(),
        "ProjectManager drop must release target registry"
    );

    let removed = extensions.revoke_owner_registrations(target_owner);
    assert!(
        !removed.components.is_empty(),
        "target owner revocation must remove the compiled component"
    );
    assert_eq!(
        source_callbacks
            .lock()
            .expect("source callback lock poisoned")
            .as_slice(),
        &[source_owner_id],
        "source callback must retain its source owner after PM drop"
    );
    assert!(extensions.owner_revocation_failures().is_empty());

    // Reopening through the same selected project manifest proves that the
    // persisted scene remains intact while the target PM registry is absent.
    let reopened = open_project(&root, selected_manifest.clone());
    assert!(
        World::load_scene_from_uri(&reopened, &scene_uri).is_err(),
        "revoked target codec must not decode before the normal retry"
    );
    let retry_selected = reopened.manifest().plugins.clone();
    let retry_plan =
        catalog.compiled_project_plan(&retry_selected, RuntimeTargetMode::ClientRuntime);
    let mut retry_extensions = retry_plan.runtime_extensions().registry.clone();
    let retry_target_owner = retry_extensions
        .scene_component_codec_owners()
        .next()
        .expect("retry target owner");
    retry_extensions
        .apply_scene_component_codecs_to_project_manager(&reopened)
        .expect("retry codec application");
    let retry_loaded = World::load_scene_from_uri(&reopened, &scene_uri).expect("retry scene load");
    assert_eq!(
        retry_loaded.dynamic_component(entity, TYPE_ID),
        Some(&payload)
    );

    let retry_registry_weak = reopened.scene_component_registry_weak();
    drop(reopened);
    assert!(retry_registry_weak.upgrade().is_none());
    let retry_removed = retry_extensions.revoke_owner_registrations(retry_target_owner);
    assert!(
        !retry_removed.components.is_empty(),
        "retry target owner revocation must remove its component"
    );
    assert_eq!(
        source_callbacks
            .lock()
            .expect("source callback lock poisoned")
            .as_slice(),
        &[source_owner_id, source_owner_id],
        "source callback identity must survive both PM drops"
    );
    assert!(retry_extensions.owner_revocation_failures().is_empty());

    fs::remove_dir_all(&root).expect("temporary project cleanup");
}
