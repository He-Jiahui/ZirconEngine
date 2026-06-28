import tempfile
import unittest
from pathlib import Path

from tools.plugin_structure_audits.registration import audit_runtime_registration_builder


class PluginStructureAuditRegistrationTests(unittest.TestCase):
    def test_runtime_registration_builder_accepts_descriptor_factory_arguments(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            runtime_src = repo_root / "zircon_plugins" / "physics" / "runtime" / "src"
            runtime_src.mkdir(parents=True)
            (runtime_src / "plugin.rs").write_text(
                """
fn register(registry: &mut RuntimeExtensionRegistry) -> Result<(), RuntimeExtensionRegistryError> {
    let shared_manager = Arc::new(DefaultPhysicsManager::new(None));
    let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
        .module(
            PLUGIN_RUNTIME_MODULE_NAME,
            module_descriptor_with_manager(Some(shared_manager.clone())),
        )?;
    module.export_interface::<dyn PhysicsQueryInterface>(shared_manager)?;
    register_runtime_system(&mut module)
}
""",
                encoding="utf-8",
            )
            (runtime_src / "runtime_system.rs").write_text(
                """
fn register_runtime_system(module: &mut RuntimePluginModuleRegistration<'_>) {
    module
        .runtime_scene_system(PHYSICS_STEP_SYSTEM, SystemStage::FixedUpdate, |_| Ok(()))
        .register()
        .unwrap();
}
""",
                encoding="utf-8",
            )
            violations: list[str] = []

            audit_runtime_registration_builder(repo_root, runtime_src, violations)

        self.assertEqual([], violations)

    def test_runtime_registration_builder_rejects_descriptor_on_wrong_module(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            runtime_src = repo_root / "zircon_plugins" / "physics" / "runtime" / "src"
            runtime_src.mkdir(parents=True)
            (runtime_src / "plugin.rs").write_text(
                """
const PLUGIN_RUNTIME_MODULE_NAME: &str = "physics";
const OTHER_MODULE_NAME: &str = "physics.shadow";

fn register(registry: &mut RuntimeExtensionRegistry) -> Result<(), RuntimeExtensionRegistryError> {
    let shared_manager = Arc::new(DefaultPhysicsManager::new(None));
    let _expected_owner = PLUGIN_RUNTIME_MODULE_NAME;
    let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
        .module(
            OTHER_MODULE_NAME,
            module_descriptor_with_manager(Some(shared_manager.clone())),
        )?;
    module.export_interface::<dyn PhysicsQueryInterface>(shared_manager)?;
    register_runtime_system(&mut module)
}
""",
                encoding="utf-8",
            )
            (runtime_src / "runtime_system.rs").write_text(
                """
fn register_runtime_system(module: &mut RuntimePluginModuleRegistration<'_>) {
    module
        .runtime_scene_system(PHYSICS_STEP_SYSTEM, SystemStage::FixedUpdate, |_| Ok(()))
        .register()
        .unwrap();
}
""",
                encoding="utf-8",
            )
            violations: list[str] = []

            audit_runtime_registration_builder(repo_root, runtime_src, violations)

        self.assertEqual(
            [
                "zircon_plugins/physics/runtime/src/plugin.rs:missing:.module(PLUGIN_RUNTIME_MODULE_NAME, module_descriptor())"
            ],
            violations,
        )


if __name__ == "__main__":
    unittest.main()
