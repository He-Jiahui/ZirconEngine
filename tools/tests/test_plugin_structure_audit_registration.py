import tempfile
import unittest
from pathlib import Path

from tools.plugin_structure_audits.registration import (
    audit_plugin_registration_conformance,
    audit_runtime_registration_builder,
    audit_runtime_plugin_descriptor_single_source,
)


class PluginStructureAuditRegistrationTests(unittest.TestCase):
    def test_runtime_plugin_descriptor_single_source_accepts_embedded_owner(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            runtime_src = repo_root / "zircon_plugins" / "sample" / "runtime" / "src"
            (runtime_src / "tests").mkdir(parents=True)
            (runtime_src / "plugin.rs").write_text(
                """
impl RuntimePlugin for SampleRuntimePlugin {}

fn descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder().with_module_descriptor(module_descriptor())
}
""",
                encoding="utf-8",
            )
            (runtime_src / "tests" / "registration.rs").write_text(
                "runtime.register_module(module_descriptor()).unwrap();\n",
                encoding="utf-8",
            )
            audited_roots: list[str] = []
            violations: list[str] = []

            audit_runtime_plugin_descriptor_single_source(
                repo_root,
                runtime_src,
                audited_roots,
                violations,
            )

        self.assertEqual(
            ["zircon_plugins/sample/runtime/src"],
            audited_roots,
        )
        self.assertEqual([], violations)

    def test_runtime_plugin_descriptor_single_source_accepts_split_descriptor_owner(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            runtime_src = repo_root / "zircon_plugins" / "sample" / "runtime" / "src"
            descriptor_root = runtime_src / "runtime_plugin"
            descriptor_root.mkdir(parents=True)
            (runtime_src / "plugin.rs").write_text(
                "impl RuntimePlugin for SampleRuntimePlugin {}\n",
                encoding="utf-8",
            )
            (descriptor_root / "descriptor.rs").write_text(
                "RuntimePluginDescriptor::builder()\n"
                "    .with_module_descriptor(module_descriptor())\n",
                encoding="utf-8",
            )
            audited_roots: list[str] = []
            violations: list[str] = []

            audit_runtime_plugin_descriptor_single_source(
                repo_root,
                runtime_src,
                audited_roots,
                violations,
            )

        self.assertEqual(
            ["zircon_plugins/sample/runtime/src"],
            audited_roots,
        )
        self.assertEqual([], violations)

    def test_runtime_plugin_descriptor_single_source_rejects_parallel_registration(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            runtime_src = repo_root / "zircon_plugins" / "sample" / "runtime" / "src"
            runtime_src.mkdir(parents=True)
            (runtime_src / "plugin.rs").write_text(
                """
impl RuntimePlugin for SampleRuntimePlugin {}

fn descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder()
        .with_module_descriptor(first_module_descriptor())
        .with_module_descriptor(second_module_descriptor())
}

fn register(registry: &mut RuntimeExtensionRegistry) {
    registry.register_module(first_module_descriptor()).unwrap();
}
""",
                encoding="utf-8",
            )
            audited_roots: list[str] = []
            violations: list[str] = []

            audit_runtime_plugin_descriptor_single_source(
                repo_root,
                runtime_src,
                audited_roots,
                violations,
            )

        self.assertEqual(
            [
                "zircon_plugins/sample/runtime/src:multiple:.with_module_descriptor(...):2",
                "zircon_plugins/sample/runtime/src/plugin.rs:11:stale:register_module(...)",
            ],
            violations,
        )

    def test_runtime_plugin_descriptor_single_source_rejects_missing_descriptor(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            runtime_src = repo_root / "zircon_plugins" / "sample" / "runtime" / "src"
            runtime_src.mkdir(parents=True)
            (runtime_src / "plugin.rs").write_text(
                "impl RuntimePlugin for SampleRuntimePlugin {}\n",
                encoding="utf-8",
            )
            audited_roots: list[str] = []
            violations: list[str] = []

            audit_runtime_plugin_descriptor_single_source(
                repo_root,
                runtime_src,
                audited_roots,
                violations,
            )

        self.assertEqual(
            ["zircon_plugins/sample/runtime/src:missing:.with_module_descriptor(...)"],
            violations,
        )

    def test_global_registration_audit_reports_root_compatibility_tracks(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            plugin_workspace = repo_root / "zircon_plugins"
            plugin_workspace.mkdir()
            (plugin_workspace / "Cargo.toml").write_text(
                """
[workspace]
members = ["sample/runtime", "sample/editor"]
""",
                encoding="utf-8",
            )
            runtime_src = plugin_workspace / "sample" / "runtime" / "src"
            runtime_src.mkdir(parents=True)
            (runtime_src / "registration.rs").write_text("", encoding="utf-8")
            (runtime_src / "lib.rs").write_text(
                "pub use registration::register;\n",
                encoding="utf-8",
            )
            editor_src = plugin_workspace / "sample" / "editor" / "src"
            (editor_src / "registration").mkdir(parents=True)
            (editor_src / "registration" / "mod.rs").write_text(
                "",
                encoding="utf-8",
            )
            (editor_src / "lib.rs").write_text(
                "mod registration;\n",
                encoding="utf-8",
            )

            report = audit_plugin_registration_conformance(repo_root).to_json()

        self.assertEqual(4, report["registration_compatibility_shim_sites"])
        self.assertEqual(
            [
                "zircon_plugins/sample/editor/src/lib.rs:1",
                "zircon_plugins/sample/editor/src/registration/mod.rs:root-owner",
                "zircon_plugins/sample/runtime/src/lib.rs:1",
                "zircon_plugins/sample/runtime/src/registration.rs:root-owner",
            ],
            report["registration_compatibility_shim_site_details"],
        )

    def test_global_registration_audit_finds_public_register_outside_importers(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            plugin_workspace = repo_root / "zircon_plugins"
            plugin_workspace.mkdir()
            (plugin_workspace / "Cargo.toml").write_text(
                """
[workspace]
members = ["sample/runtime", "plugin_sdk"]
""",
                encoding="utf-8",
            )
            runtime_src = plugin_workspace / "sample" / "runtime" / "src"
            runtime_src.mkdir(parents=True)
            (runtime_src / "plugin.rs").write_text(
                """
pub fn register(registry: &mut RuntimeExtensionRegistry) {
    registry.register_module();
}
""",
                encoding="utf-8",
            )
            sdk_src = plugin_workspace / "plugin_sdk" / "src"
            sdk_src.mkdir(parents=True)
            (sdk_src / "registration.rs").write_text(
                "pub fn register(self) {}\n",
                encoding="utf-8",
            )

            report = audit_plugin_registration_conformance(repo_root).to_json()

        self.assertEqual(1, report["free_function_registration_sites"])
        self.assertEqual(
            ["zircon_plugins/sample/runtime/src/plugin.rs:2"],
            report["free_function_registration_site_details"],
        )

    def test_global_registration_audit_includes_nested_feature_runtime_members(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            plugin_workspace = repo_root / "zircon_plugins"
            plugin_workspace.mkdir()
            (plugin_workspace / "Cargo.toml").write_text(
                """
[workspace]
members = ["sound/runtime", "sound/features/spatial/runtime"]
""",
                encoding="utf-8",
            )
            feature_src = (
                plugin_workspace
                / "sound"
                / "features"
                / "spatial"
                / "runtime"
                / "src"
            )
            feature_src.mkdir(parents=True)
            (feature_src / "plugin.rs").write_text(
                "pub fn register(registry: &mut RuntimeExtensionRegistry) {}\n",
                encoding="utf-8",
            )

            report = audit_plugin_registration_conformance(repo_root).to_json()

        self.assertEqual(1, report["free_function_registration_sites"])
        self.assertEqual(
            [
                "zircon_plugins/sound/features/spatial/runtime/src/plugin.rs:1"
            ],
            report["free_function_registration_site_details"],
        )

    def test_runtime_registration_builder_accepts_descriptor_owned_by_plugin_report(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            runtime_src = repo_root / "zircon_plugins" / "physics" / "runtime" / "src"
            runtime_src.mkdir(parents=True)
            (runtime_src / "plugin.rs").write_text(
                """
fn register(registry: &mut RuntimeExtensionRegistry) -> Result<(), RuntimeExtensionRegistryError> {
    let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
        .module(PLUGIN_RUNTIME_MODULE_NAME)?;
    module.export_interface::<dyn PhysicsQueryInterface>(self.manager.clone())?;
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

    def test_runtime_registration_builder_rejects_wrong_owner_name(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            runtime_src = repo_root / "zircon_plugins" / "physics" / "runtime" / "src"
            runtime_src.mkdir(parents=True)
            (runtime_src / "plugin.rs").write_text(
                """
const PLUGIN_RUNTIME_MODULE_NAME: &str = "physics";
const OTHER_MODULE_NAME: &str = "physics.shadow";

fn register(registry: &mut RuntimeExtensionRegistry) -> Result<(), RuntimeExtensionRegistryError> {
    let _expected_owner = PLUGIN_RUNTIME_MODULE_NAME;
    let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
        .module(OTHER_MODULE_NAME)?;
    module.export_interface::<dyn PhysicsQueryInterface>(self.manager.clone())?;
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
                "zircon_plugins/physics/runtime/src/plugin.rs:missing:.module(PLUGIN_RUNTIME_MODULE_NAME)"
            ],
            violations,
        )

    def test_runtime_registration_builder_rejects_retired_descriptor_argument(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            runtime_src = repo_root / "zircon_plugins" / "physics" / "runtime" / "src"
            runtime_src.mkdir(parents=True)
            (runtime_src / "plugin.rs").write_text(
                """
fn register(registry: &mut RuntimeExtensionRegistry) -> Result<(), RuntimeExtensionRegistryError> {
    let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
        .module(PLUGIN_RUNTIME_MODULE_NAME, module_descriptor())?;
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
                "zircon_plugins/physics/runtime/src/plugin.rs:missing:.module(PLUGIN_RUNTIME_MODULE_NAME)",
                "zircon_plugins/physics/runtime/src/plugin.rs:stale:module builder descriptor argument",
            ],
            violations,
        )


if __name__ == "__main__":
    unittest.main()
