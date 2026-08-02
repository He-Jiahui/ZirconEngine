use std::sync::{Arc, Mutex};

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, NodeKind, VmTypeBacking, World};
use zircon_runtime::script::{
    with_script_runtime_test_context, CapabilitySet, HotReloadCoordinator,
    ScriptRuntimeTestContext, VmBackend, VmError, VmPluginHostContext, VmPluginInstance,
    VmPluginManager, VmPluginManifest, VmPluginPackage, VmPluginPackageSource,
    VmPluginSlotLifecycle, VmPluginSlotRecord, VmStateSchema, VmStateTypeSchema,
};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectFieldInfo, ReflectScriptVisibility, ReflectSerializationStrategy,
    ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration, ReflectedValue,
};

use super::{ReflectionHostError, ReflectionHostModule};
use crate::{CallSiteError, CompiledCallSite, ScriptCallTable};

#[test]
fn reflection_host_resolves_once_then_reads_and_writes_by_numeric_token() {
    let schema = state_schema(ReflectScriptVisibility::Public);
    let mut world = World::empty();
    world
        .register_vm_type(
            schema.types[0].registration.clone(),
            VmTypeBacking::DynamicComponent,
        )
        .expect("test world should install the public VM type");
    let host = reflection_host();
    host.install_type_registry(world.type_registry())
        .expect("public registry should compile into the reflection host table");
    let token = host
        .resolve("gameplay.Component.Health", "current")
        .expect("public VM field should resolve during package loading");

    let entity = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            entity,
            "gameplay.Component.Health",
            serde_json::json!({ "current": 25.0 }),
        )
        .expect("VM dynamic component should attach");
    let levels = DefaultLevelManager::default();
    let level = levels.create_level(world, LevelMetadata::default());
    let runtime = CoreRuntime::new();
    let context = ScriptRuntimeTestContext::new(runtime.handle().downgrade(), level, entity, 0.0);

    with_script_runtime_test_context(context, || {
        assert_eq!(
            host.read(token, entity)
                .expect("numeric reflection read should succeed"),
            ReflectedValue::Scalar(25.0)
        );
        assert!(host
            .write(token, entity, ReflectedValue::Scalar(40.0))
            .expect("numeric reflection write should succeed"));
        assert_eq!(
            host.read(token, entity)
                .expect("numeric reflection read should observe the write"),
            ReflectedValue::Scalar(40.0)
        );
    });
    assert_eq!(
        host.resolution_count(),
        1,
        "runtime reflection calls must not repeat string resolution"
    );
}

#[test]
fn reflection_host_does_not_expose_private_vm_types() {
    let schema = state_schema(ReflectScriptVisibility::Private);
    let mut world = World::empty();
    world
        .register_vm_type(
            schema.types[0].registration.clone(),
            VmTypeBacking::DynamicComponent,
        )
        .expect("private VM type should still register in the host registry");
    let host = reflection_host();
    host.install_type_registry(world.type_registry())
        .expect("private types should be filtered rather than rejected");

    let error = host
        .resolve("gameplay.Component.Health", "current")
        .expect_err("private VM type must not be callable from script reflection");

    assert!(matches!(
        error,
        ReflectionHostError::CallSite(CallSiteError::UnknownMember { .. })
    ));
}

#[test]
fn reflection_host_exports_public_builtin_component_fields() {
    let world = World::empty();
    let host = reflection_host();
    host.install_type_registry(world.type_registry())
        .expect("canonical host registry should compile");

    host.resolve("zircon_runtime::scene::components::Name", "value")
        .expect("public builtin Name field should be exported to VM reflection");
}

#[test]
fn prepared_tables_resolve_before_commit_but_dispatch_only_after_exact_commit() {
    let manager = VmPluginManager::mock();
    let schema = state_schema(ReflectScriptVisibility::Public);
    let table = Arc::new(Mutex::new(None));
    let site = Arc::new(Mutex::new(None));
    let host = test_host_context(&manager, "prepared-dispatch");
    let installed_table = Arc::clone(&table);
    host.reflection_schema_installer
        .register(move |snapshot| {
            *installed_table.lock().unwrap() = Some(
                ScriptCallTable::compile_snapshot(snapshot)
                    .map_err(|error| VmError::Operation(error.to_string()))?,
            );
            Ok(())
        })
        .expect("test installer should accept the prepared snapshot");
    let backend = PreparedDispatchBackend {
        schema: schema.clone(),
        table: Arc::clone(&table),
        site: Arc::clone(&site),
    };

    HotReloadCoordinator::new()
        .load_package("prepared-dispatch", &backend, gameplay_package(), &host)
        .expect("the exact prepared generation should commit after activation");

    let table = table
        .lock()
        .unwrap()
        .clone()
        .expect("activation should retain the prepared table");
    assert!(table.is_catalog_current());
    let site = site
        .lock()
        .unwrap()
        .clone()
        .expect("activation should resolve the prepared call site");
    let (world, entity) = world_with_health(&schema);
    assert_eq!(
        table
            .read(&site, &world, entity)
            .expect("the same token should dispatch after exact commit"),
        ReflectedValue::Scalar(25.0)
    );
}

#[test]
fn abandoned_prepared_table_cannot_resolve_after_another_candidate_commits() {
    let manager = VmPluginManager::mock();
    let schema = state_schema(ReflectScriptVisibility::Public);
    let installed_table = Arc::new(Mutex::new(None));
    let installed_for_callback = Arc::clone(&installed_table);
    let host = test_host_context(&manager, "abandoned-resolution");
    host.reflection_schema_installer
        .register(move |snapshot| {
            *installed_for_callback.lock().unwrap() = Some(
                ScriptCallTable::compile_snapshot(snapshot)
                    .map_err(|error| VmError::Operation(error.to_string()))?,
            );
            Ok(())
        })
        .expect("test installer should accept prepared snapshots");
    let coordinator = HotReloadCoordinator::new();

    coordinator
        .load_package(
            "abandoned-resolution",
            &FailingActivationBackend {
                schema: schema.clone(),
            },
            gameplay_package(),
            &host,
        )
        .expect_err("first prepared generation should be abandoned");
    let abandoned = installed_table
        .lock()
        .unwrap()
        .clone()
        .expect("failed activation should leave the captured table available to the test");
    assert!(abandoned
        .resolve("gameplay.Component.Health", "current")
        .is_ok());

    let committed_table = Arc::clone(&installed_table);
    coordinator
        .load_package(
            "abandoned-resolution",
            &PreparedDispatchBackend {
                schema,
                table: committed_table,
                site: Arc::new(Mutex::new(None)),
            },
            gameplay_package(),
            &host,
        )
        .expect("second prepared generation should commit");

    assert!(matches!(
        abandoned
            .resolve("gameplay.Component.Health", "current")
            .expect_err("an abandoned table must lose name-resolution capability"),
        CallSiteError::StaleCatalogRevision { .. }
    ));
}

#[derive(Debug)]
struct FailingActivationBackend {
    schema: VmStateSchema,
}

impl VmBackend for FailingActivationBackend {
    fn backend_name(&self) -> &str {
        "abandoned-resolution"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Ok(Box::new(FailingActivationInstance {
            manifest: package.manifest.clone(),
            schema: self.schema.clone(),
        }))
    }
}

#[derive(Debug)]
struct FailingActivationInstance {
    manifest: VmPluginManifest,
    schema: VmStateSchema,
}

impl VmPluginInstance for FailingActivationInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn state_schema(&mut self) -> Result<Option<VmStateSchema>, VmError> {
        Ok(Some(self.schema.clone()))
    }

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        Err(VmError::Operation(
            "intentional activation failure".to_string(),
        ))
    }
}

#[derive(Debug)]
struct PreparedDispatchBackend {
    schema: VmStateSchema,
    table: Arc<Mutex<Option<ScriptCallTable>>>,
    site: Arc<Mutex<Option<CompiledCallSite>>>,
}

impl VmBackend for PreparedDispatchBackend {
    fn backend_name(&self) -> &str {
        "prepared-dispatch"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Ok(Box::new(PreparedDispatchInstance {
            manifest: package.manifest.clone(),
            schema: self.schema.clone(),
            table: Arc::clone(&self.table),
            site: Arc::clone(&self.site),
        }))
    }
}

#[derive(Debug)]
struct PreparedDispatchInstance {
    manifest: VmPluginManifest,
    schema: VmStateSchema,
    table: Arc<Mutex<Option<ScriptCallTable>>>,
    site: Arc<Mutex<Option<CompiledCallSite>>>,
}

impl VmPluginInstance for PreparedDispatchInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn state_schema(&mut self) -> Result<Option<VmStateSchema>, VmError> {
        Ok(Some(self.schema.clone()))
    }

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        let table =
            self.table.lock().unwrap().clone().ok_or_else(|| {
                VmError::Operation("prepared table was not installed".to_string())
            })?;
        let resolved = table
            .resolve("gameplay.Component.Health", "current")
            .map_err(|error| VmError::Operation(error.to_string()))?;
        let (world, entity) = world_with_health(&self.schema);
        if !matches!(
            table.read(&resolved, &world, entity),
            Err(CallSiteError::StaleCatalogRevision { .. })
        ) {
            return Err(VmError::Operation(
                "prepared call table dispatched before catalog commit".to_string(),
            ));
        }
        *self.site.lock().unwrap() = Some(resolved);
        Ok(())
    }
}

fn world_with_health(schema: &VmStateSchema) -> (World, u64) {
    let mut world = World::empty();
    world
        .register_vm_type(
            schema.types[0].registration.clone(),
            VmTypeBacking::DynamicComponent,
        )
        .expect("test World should install the package schema");
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            entity,
            "gameplay.Component.Health",
            serde_json::json!({ "current": 25.0 }),
        )
        .expect("test health component should attach");
    (world, entity)
}

fn gameplay_package() -> VmPluginPackage {
    VmPluginPackage {
        manifest: VmPluginManifest {
            name: "gameplay".to_string(),
            version: "1.0.0".to_string(),
            entry: "main".to_string(),
            capabilities: CapabilitySet::default(),
            management: Default::default(),
        },
        zr_vm_project: None,
        bytecode: Vec::new(),
    }
}

fn reflection_host() -> ReflectionHostModule {
    let manager = VmPluginManager::mock();
    let host_context = test_host_context(&manager, "reflection-host");
    ReflectionHostModule::new(host_context.reflection_world_access())
}

fn test_host_context(
    manager: &Arc<VmPluginManager>,
    backend_selector: &str,
) -> VmPluginHostContext {
    VmPluginHostContext::new_for_tests(
        manager.base_plugin_context().clone(),
        CapabilitySet::default(),
        backend_selector.to_string(),
        VmPluginPackageSource::default(),
        manager.host_registry(),
        manager.host_exports(),
        manager.host_interfaces(),
        manager.reflection_catalog(),
        Default::default(),
        Arc::new(NoopSlotLifecycle),
    )
}

struct NoopSlotLifecycle;

impl VmPluginSlotLifecycle for NoopSlotLifecycle {
    fn load_package(
        &self,
        _backend_selector: &str,
        _package: VmPluginPackage,
    ) -> Result<zircon_runtime::script::PluginSlotId, VmError> {
        Err(VmError::Operation("unused test lifecycle".to_string()))
    }

    fn hot_reload_slot(
        &self,
        _slot: zircon_runtime::script::PluginSlotId,
        _package: VmPluginPackage,
    ) -> Result<(), VmError> {
        Err(VmError::Operation("unused test lifecycle".to_string()))
    }

    fn unload_slot(&self, _slot: zircon_runtime::script::PluginSlotId) -> Result<(), VmError> {
        Err(VmError::Operation("unused test lifecycle".to_string()))
    }

    fn slot(
        &self,
        slot: zircon_runtime::script::PluginSlotId,
    ) -> Result<VmPluginSlotRecord, VmError> {
        Err(VmError::MissingSlot(slot.get()))
    }

    fn list_slots(&self) -> Vec<VmPluginSlotRecord> {
        Vec::new()
    }
}

fn state_schema(visibility: ReflectScriptVisibility) -> VmStateSchema {
    VmStateSchema {
        schema_version: 2,
        types: vec![VmStateTypeSchema {
            registration: ReflectTypeRegistration::new(
                ReflectTypePath::new("gameplay.Component.Health", "Health")
                    .expect("test reflection path should be valid")
                    .with_plugin_id("gameplay"),
                "Health",
                ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
                    "current",
                    "Scalar",
                    ReflectEditorHint::Scalar,
                )]),
                ReflectSerializationStrategy::Value,
            )
            .as_component()
            .with_plugin_owned(true)
            .with_plugin_id("gameplay")
            .with_script_visibility(visibility),
            type_hash: 1,
            renames: Vec::new(),
        }],
    }
}
