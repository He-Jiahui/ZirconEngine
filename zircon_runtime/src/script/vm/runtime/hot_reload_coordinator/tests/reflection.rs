use std::sync::{Arc, Mutex};

use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectFieldInfo, ReflectScriptVisibility, ReflectSerializationStrategy,
    ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
};

use super::*;
use crate::scene::World;
use crate::script::{VmStateSchema, VmStateTypeSchema};

#[derive(Debug)]
struct ReflectionSchemaBackend {
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl VmBackend for ReflectionSchemaBackend {
    fn backend_name(&self) -> &str {
        "reflection-schema"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        Ok(Box::new(ReflectionSchemaInstance {
            manifest: package.manifest.clone(),
            events: Arc::clone(&self.events),
        }))
    }
}

#[derive(Debug)]
struct ReflectionSchemaInstance {
    manifest: VmPluginManifest,
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl VmPluginInstance for ReflectionSchemaInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn state_schema(&mut self) -> Result<Option<VmStateSchema>, VmError> {
        self.events.lock().unwrap().push("state_schema");
        Ok(Some(vm_state_schema()))
    }

    fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError> {
        self.events.lock().unwrap().push("activate");
        Ok(())
    }
}

#[test]
fn load_publishes_reflection_schema_before_slot_becomes_active() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let backend = ReflectionSchemaBackend {
        events: Arc::clone(&events),
    };
    let host = test_host_context();
    let install_events = Arc::clone(&events);
    host.reflection_schema_installer
        .register(move |snapshot| {
            assert!(snapshot.can_resolve_names());
            assert!(!snapshot.is_current());
            install_events.lock().unwrap().push("install_schema");
            Ok(())
        })
        .expect("test reflection installer should register");
    let coordinator = HotReloadCoordinator::new();

    let mut package = test_package("0.1.0");
    package.manifest.name = "gameplay".to_string();
    coordinator
        .load_package("reflection-schema", &backend, package, &host)
        .expect("valid reflected VM package should load");

    assert_eq!(
        events.lock().unwrap().as_slice(),
        &["state_schema", "install_schema", "activate"]
    );
    let mut world = World::empty();
    host.reflection_catalog
        .apply_to_world(&mut world)
        .expect("published schema should be available to future worlds");
    assert!(world
        .type_registry()
        .contains_type_path("gameplay.Component.Health"));
}

fn vm_state_schema() -> VmStateSchema {
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
            .with_script_visibility(ReflectScriptVisibility::Public),
            type_hash: 1,
            renames: Vec::new(),
        }],
    }
}
