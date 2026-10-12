use super::*;

use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectFieldId, ReflectFieldInfo, ReflectSerializationStrategy,
    ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration, ReflectedValue,
};

use crate::script::{
    VmStateBlob, VmStateFieldValue, VmStateObject, VmStateSchema, VmStateTypeIdentity,
    VmStateTypeSchema,
};

fn health_field_id() -> ReflectFieldId {
    ReflectFieldId::from_stable_keys("tests.hot-reload-state", "health")
}

#[derive(Debug)]
struct MigrationRollbackBackend {
    events: Arc<Mutex<Vec<String>>>,
}

impl VmBackend for MigrationRollbackBackend {
    fn backend_name(&self) -> &str {
        "migration-rollback"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        _host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        self.events
            .lock()
            .unwrap()
            .push(format!("{}.load", package.manifest.version));
        Ok(Box::new(MigrationRollbackInstance {
            manifest: package.manifest.clone(),
            events: Arc::clone(&self.events),
        }))
    }
}

#[derive(Debug)]
struct MigrationRollbackInstance {
    manifest: VmPluginManifest,
    events: Arc<Mutex<Vec<String>>>,
}

impl VmPluginInstance for MigrationRollbackInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, host: &VmPluginHostContext) -> Result<(), VmError> {
        let expected_capability = format!("rollback.{}", self.manifest.version);
        if !host.capabilities.contains(&expected_capability) {
            return Err(VmError::Operation(format!(
                "{} activation received the wrong capability set",
                self.manifest.version
            )));
        }
        let expected_root = PathBuf::from(format!("{}-package", self.manifest.version));
        if host.plugin.package_root.as_ref() != Some(&expected_root) {
            return Err(VmError::Operation(format!(
                "{} activation received the wrong package root",
                self.manifest.version
            )));
        }
        self.events
            .lock()
            .unwrap()
            .push(format!("{}.activate", self.manifest.version));
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), VmError> {
        self.events
            .lock()
            .unwrap()
            .push(format!("{}.deactivate", self.manifest.version));
        Ok(())
    }

    fn save_state(&mut self) -> Result<VmStateBlob, VmError> {
        self.events
            .lock()
            .unwrap()
            .push(format!("{}.save", self.manifest.version));
        let type_path = ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap();
        VmStateBlob::from_reflected_objects(
            1,
            vec![VmStateTypeIdentity {
                type_path: type_path.clone(),
                type_hash: 1,
            }],
            &[VmStateObject {
                type_path,
                fields: vec![VmStateFieldValue::new(
                    health_field_id(),
                    ReflectedValue::Scalar(75.0),
                )],
            }],
        )
        .map_err(VmError::from)
    }

    fn state_schema(&mut self) -> Result<Option<VmStateSchema>, VmError> {
        let type_path = ReflectTypePath::new("game.PlayerState", "PlayerState").unwrap();
        Ok(Some(if self.manifest.version == "old" {
            VmStateSchema {
                schema_version: 1,
                types: vec![VmStateTypeSchema {
                    registration: state_registration(
                        type_path,
                        ReflectFieldInfo::from_stable_keys(
                            "tests.hot-reload-state",
                            "health",
                            "old_health",
                            "f64",
                            ReflectEditorHint::Scalar,
                        ),
                    ),
                    type_hash: 1,
                }],
            }
        } else {
            VmStateSchema {
                schema_version: 2,
                types: vec![VmStateTypeSchema {
                    registration: state_registration(
                        type_path,
                        ReflectFieldInfo::from_stable_keys(
                            "tests.hot-reload-state",
                            "health",
                            "health",
                            "f64",
                            ReflectEditorHint::Scalar,
                        ),
                    ),
                    type_hash: 2,
                }],
            }
        }))
    }

    fn restore_state(&mut self, state: &VmStateBlob) -> Result<(), VmError> {
        let objects = state.reflected_objects()?;
        let restored_old_health = objects.iter().any(|object| {
            object.fields.iter().any(|field| {
                field.field_id == health_field_id() && field.value == ReflectedValue::Scalar(75.0)
            })
        });
        if !restored_old_health {
            return Err(VmError::Operation(
                "rollback did not restore the old reflected state".to_string(),
            ));
        }
        self.events
            .lock()
            .unwrap()
            .push(format!("{}.restore", self.manifest.version));
        Ok(())
    }
}

fn state_registration(
    type_path: ReflectTypePath,
    field: ReflectFieldInfo,
) -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        type_path,
        "Player State",
        ReflectTypeInfo::struct_with_fields(vec![field]),
        ReflectSerializationStrategy::Value,
    )
}

#[test]
fn migration_failure_rolls_back_old_module() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let backend = MigrationRollbackBackend {
        events: Arc::clone(&events),
    };
    let coordinator = HotReloadCoordinator::new();
    let mut old_host = test_host_context();
    old_host.capabilities = CapabilitySet::default().with("rollback.old");
    old_host.plugin.package_root = Some(PathBuf::from("old-package"));
    old_host.plugin.source_root = Some(PathBuf::from("old-package"));
    old_host.plugin.data_root = Some(PathBuf::from("old-package/data"));
    let slot = coordinator
        .load_package(
            "migration-rollback",
            &backend,
            test_package("old"),
            &old_host,
        )
        .unwrap();
    let mut new_host = test_host_context();
    new_host.capabilities = CapabilitySet::default().with("rollback.new");
    new_host.plugin.package_root = Some(PathBuf::from("new-package"));
    new_host.plugin.source_root = Some(PathBuf::from("new-package"));
    new_host.plugin.data_root = Some(PathBuf::from("new-package/data"));

    let error = coordinator
        .hot_reload(
            slot,
            "migration-rollback",
            &backend,
            test_package("new"),
            &new_host,
        )
        .unwrap_err();

    assert!(matches!(error, VmError::StateMigration(_)));
    let record = coordinator.slot(slot).unwrap();
    assert_eq!(record.state, VmPluginSlotState::Active);
    assert_eq!(record.generation, 1);
    assert_eq!(record.manifest.version, "old");
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "old.load",
            "old.activate",
            "old.save",
            "old.deactivate",
            "new.load",
            "new.activate",
            "new.deactivate",
            "old.activate",
            "old.restore",
        ]
    );
}
