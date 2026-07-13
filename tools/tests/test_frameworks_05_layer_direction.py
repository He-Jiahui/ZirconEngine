from __future__ import annotations

import unittest
from pathlib import Path

from tools.runtime_domain_dependency_audit import audit_runtime_domain_dependencies


REPO_ROOT = Path(__file__).resolve().parents[2]


class Frameworks05LayerDirectionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        report = audit_runtime_domain_dependencies(REPO_ROOT)
        cls.references = report["references"]

    def assert_edges_are_empty(self, pairs: set[tuple[str, str]]) -> None:
        violations = [
            reference
            for reference in self.references
            if (reference["source_domain"], reference["target_domain"]) in pairs
        ]
        self.assertEqual(
            violations,
            [],
            "production dependency direction violations remain:\n"
            + "\n".join(
                f"{item['source_domain']}->{item['target_domain']} "
                f"{item['path']}:{item['line']} {item['source']}"
                for item in violations
            ),
        )

    def test_core_contracts_do_not_depend_on_upper_runtime_domains(self) -> None:
        self.assert_edges_are_empty(
            {
                ("core", "asset"),
                ("core", "graphics"),
                ("core", "scene"),
                ("core", "plugin"),
            }
        )

    def test_internal_domains_do_not_depend_on_runtime_facades(self) -> None:
        self.assert_edges_are_empty(
            {
                ("animation", "plugin"),
                ("asset", "plugin"),
                ("platform", "builtin"),
                ("scene", "plugin"),
                ("script", "plugin"),
            }
        )

    def test_animation_timeline_contract_does_not_project_asset_models(self) -> None:
        manager_source = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/animation/manager.rs"
        ).read_text(encoding="utf-8")
        timeline_source = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/animation/timeline.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("crate::asset", timeline_source)
        self.assertNotIn("from_sequence", timeline_source)
        self.assertNotIn("from_clip", timeline_source)
        self.assertNotIn("sequence_timeline_descriptor", manager_source)
        self.assertNotIn("clip_timeline_descriptor", manager_source)
        self.assertNotIn("sequence_track_paths", manager_source)

    def test_animation_manager_contract_does_not_mutate_scene_world(self) -> None:
        manager_source = (
            REPO_ROOT / "zircon_runtime/src/core/framework/animation/manager.rs"
        ).read_text(encoding="utf-8")
        runtime_sequence_source = (
            REPO_ROOT / "zircon_runtime/src/animation/scene_hook/sequences.rs"
        ).read_text(encoding="utf-8")
        plugin_sequence_source = (
            REPO_ROOT
            / "zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("crate::scene", manager_source)
        self.assertNotIn("apply_sequence_to_world", manager_source)
        self.assertIn("crate::animation::sequence::apply_sequence_to_world", runtime_sequence_source)
        self.assertIn("crate::sequence::apply_sequence_to_world", plugin_sequence_source)

    def test_navigation_gizmo_contract_does_not_project_nav_mesh_assets(self) -> None:
        gizmo_source = (
            REPO_ROOT / "zircon_runtime/src/core/framework/navigation/gizmo.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("crate::asset", gizmo_source)
        self.assertNotIn("from_nav_mesh_asset", gizmo_source)

    def test_render_framework_contract_does_not_accept_graphics_pipeline_assets(self) -> None:
        framework_source = (
            REPO_ROOT / "zircon_runtime/src/core/framework/render/framework.rs"
        ).read_text(encoding="utf-8")
        graphics_registration_source = (
            REPO_ROOT
            / "zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/mod.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("crate::graphics", framework_source)
        self.assertNotIn("register_pipeline_asset", framework_source)
        self.assertIn("impl WgpuRenderFramework", graphics_registration_source)
        self.assertIn("pub fn register_pipeline_asset", graphics_registration_source)

    def test_scene_component_descriptors_have_one_neutral_owner(self) -> None:
        neutral_owner = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/scene/component_type_descriptor/mod.rs"
        )
        retired_owner = (
            REPO_ROOT / "zircon_runtime/src/plugin/component_type_descriptor/mod.rs"
        )
        self.assertTrue(neutral_owner.is_file())
        self.assertFalse(retired_owner.exists())

        for relative_path in (
            "zircon_runtime/src/scene/dynamic_scene/scene/capture.rs",
            "zircon_runtime/src/scene/dynamic_scene/scene/mod.rs",
            "zircon_runtime/src/scene/dynamic_scene/scene/validation.rs",
            "zircon_runtime/src/scene/reflect/dynamic_component.rs",
            "zircon_runtime/src/scene/world/component_type_registry.rs",
            "zircon_runtime/src/scene/world/dynamic_components.rs",
        ):
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertNotIn("crate::plugin::ComponentTypeDescriptor", source)
            self.assertNotIn("crate::plugin::{ComponentPropertyDescriptor", source)

    def test_navigation_asset_schema_has_one_neutral_owner(self) -> None:
        neutral_owner = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/navigation/asset/mod.rs"
        )
        retired_owner = REPO_ROOT / "zircon_runtime/src/asset/assets/navigation.rs"
        self.assertTrue(neutral_owner.is_file())
        self.assertFalse(retired_owner.exists())

        for relative_path in (
            "zircon_runtime/src/core/framework/navigation/bake.rs",
            "zircon_runtime/src/core/framework/navigation/manager.rs",
        ):
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertNotIn("crate::asset", source)

    def test_runtime_target_mode_has_one_neutral_owner(self) -> None:
        neutral_owner = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/platform/runtime_target_mode.rs"
        )
        retired_owner = (
            REPO_ROOT
            / "zircon_runtime/src/builtin/runtime_modules/ids/target_mode.rs"
        )
        builtin_facade = (
            REPO_ROOT / "zircon_runtime/src/builtin/mod.rs"
        ).read_text(encoding="utf-8")
        plugin_sdk_prelude = (
            REPO_ROOT / "zircon_plugins/plugin_sdk/src/prelude.rs"
        ).read_text(encoding="utf-8")
        self.assertTrue(neutral_owner.is_file())
        self.assertFalse(retired_owner.exists())
        self.assertNotIn("RuntimeTargetMode", builtin_facade)
        self.assertNotIn("RuntimeTargetMode", plugin_sdk_prelude)

        for path in (REPO_ROOT / "zircon_runtime/src/platform").rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            self.assertNotIn("crate::builtin::RuntimeTargetMode", source)
            self.assertNotIn("crate::builtin::{RuntimeTargetMode", source)

    def test_animation_asset_schema_has_one_neutral_owner(self) -> None:
        neutral_owner = (
            REPO_ROOT / "zircon_runtime/src/core/framework/animation/asset/mod.rs"
        )
        retired_owner = REPO_ROOT / "zircon_runtime/src/asset/assets/animation"
        manager_source = (
            REPO_ROOT / "zircon_runtime/src/core/framework/animation/manager.rs"
        ).read_text(encoding="utf-8")
        self.assertTrue(neutral_owner.is_file())
        self.assertFalse(retired_owner.exists())
        self.assertNotIn("crate::asset", manager_source)

    def test_asset_native_importer_uses_neutral_command_host(self) -> None:
        source = (
            REPO_ROOT / "zircon_runtime/src/asset/importer/native.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("crate::plugin", source)
        self.assertNotIn("LoadedNativePlugin", source)
        self.assertNotIn("NativePluginBehaviorCallReport", source)
        self.assertIn("NativeAssetImportCommandHost", source)
        self.assertIn("NativeAssetImportCommandReport", source)

    def test_asset_package_roots_do_not_accept_plugin_manifests(self) -> None:
        manager_source = (
            REPO_ROOT / "zircon_runtime/src/asset/project/manager/package_assets.rs"
        ).read_text(encoding="utf-8")
        registry_source = (
            REPO_ROOT / "zircon_runtime/src/asset/project/package_asset_registry.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("crate::plugin", manager_source)
        self.assertNotIn("crate::plugin", registry_source)
        self.assertNotIn("PluginPackageManifest", manager_source)
        self.assertNotIn("PluginPackageManifest", registry_source)
        self.assertNotIn("register_package_manifest_asset_roots", manager_source)
        self.assertNotIn("register_manifest_roots", registry_source)
        self.assertIn("register_package_asset_roots", manager_source)
        self.assertIn("register_package_roots", registry_source)

    def test_scene_runtime_hooks_are_owned_and_stored_by_scene(self) -> None:
        scene_owner = REPO_ROOT / "zircon_runtime/src/scene/runtime_hook/mod.rs"
        retired_owner = REPO_ROOT / "zircon_runtime/src/plugin/scene_hook/mod.rs"
        world_driver = (
            REPO_ROOT / "zircon_runtime/src/scene/module/world_driver.rs"
        ).read_text(encoding="utf-8")
        self.assertTrue(scene_owner.is_file())
        self.assertFalse(retired_owner.exists())
        self.assertIn("SceneRuntimeHookSet", world_driver)

        for path in (REPO_ROOT / "zircon_runtime/src/core/runtime").rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            self.assertNotIn("SceneRuntimeHookRegistration", source)
            self.assertNotIn("SceneRuntimeHookSet", source)

    def test_script_bridge_host_does_not_depend_on_plugin_manifests(self) -> None:
        source = (
            REPO_ROOT / "zircon_runtime/src/script/vm/host/bridge_host_module.rs"
        ).read_text(encoding="utf-8")
        neutral_owner = REPO_ROOT / "zircon_runtime/src/core/framework/bridge"
        plugin_bridge_root = (
            REPO_ROOT / "zircon_runtime/src/plugin/bridge.rs"
        ).read_text(encoding="utf-8")
        self.assertNotIn("crate::plugin", source)
        self.assertNotIn("PluginPackageManifest", source)
        self.assertNotIn("PluginInterfaceMethodManifest", source)
        self.assertNotIn("register_bridge_host_module_from_manifest", source)
        self.assertIn("BridgeInvocationTable", source)
        self.assertTrue((neutral_owner / "interface_slot.rs").is_file())
        self.assertFalse((neutral_owner / "weak.rs").exists())
        self.assertTrue(
            (REPO_ROOT / "zircon_runtime/src/plugin/bridge/weak.rs").is_file()
        )
        for retired_path in (
            "zircon_runtime/src/plugin/bridge/interface_id.rs",
            "zircon_runtime/src/plugin/bridge/diagnostics.rs",
            "zircon_runtime/src/plugin/bridge/strong.rs",
        ):
            self.assertFalse((REPO_ROOT / retired_path).exists())
        self.assertNotIn("BridgeInterfaceStatus", plugin_bridge_root)
        self.assertNotIn("BridgeOwnerTransitionMode", plugin_bridge_root)
        for path in neutral_owner.rglob("*.rs"):
            neutral_source = path.read_text(encoding="utf-8")
            self.assertNotIn("crate::plugin", neutral_source)

    def test_core_runtime_observes_modules_without_owning_plugin_lifecycle(self) -> None:
        observer_owner = (
            REPO_ROOT
            / "zircon_runtime/src/core/runtime/module_lifecycle_observer.rs"
        )
        plugin_adapter = (
            REPO_ROOT
            / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs"
        ).read_text(encoding="utf-8")
        self.assertTrue(observer_owner.is_file())
        self.assertIn("impl RuntimeModuleLifecycleObserver", plugin_adapter)

        for relative_path in (
            "zircon_runtime/src/core/runtime/runtime.rs",
            "zircon_runtime/src/core/runtime/handle/core_handle.rs",
            "zircon_runtime/src/core/runtime/handle/runtime_extensions.rs",
            "zircon_runtime/src/core/runtime/state/core_runtime_state.rs",
        ):
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertNotIn("RuntimePluginBridgeLifecycle", source)
            self.assertNotIn("plugin_bridge_lifecycle", source)

    def test_world_runtime_extensions_are_planned_and_stored_by_scene(self) -> None:
        scene_plan_owner = (
            REPO_ROOT / "zircon_runtime/src/scene/runtime_extension/mod.rs"
        )
        world_driver = (
            REPO_ROOT / "zircon_runtime/src/scene/module/world_driver.rs"
        ).read_text(encoding="utf-8")
        plugin_projection = (
            REPO_ROOT
            / "zircon_runtime/src/plugin/extension_registry/apply_to_world.rs"
        ).read_text(encoding="utf-8")
        self.assertTrue(scene_plan_owner.is_file())
        self.assertIn("WorldRuntimeExtensionPlan", world_driver)
        self.assertIn("world_runtime_extension_plan", plugin_projection)
        self.assertFalse(
            (
                REPO_ROOT
                / "zircon_runtime/src/core/runtime/state/world_runtime_extensions.rs"
            ).exists()
        )

        for relative_path in (
            "zircon_runtime/src/core/runtime/runtime.rs",
            "zircon_runtime/src/core/runtime/handle/core_handle.rs",
            "zircon_runtime/src/core/runtime/handle/runtime_extensions.rs",
            "zircon_runtime/src/core/runtime/state/core_runtime_state.rs",
        ):
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertNotIn("WorldRuntimeExtensionSet", source)
            self.assertNotIn("world_extensions", source)
            self.assertNotIn("install_world_runtime_extensions", source)

    def test_core_devtools_catalog_is_injected_as_data(self) -> None:
        devtools_source = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/diagnostics/devtools.rs"
        ).read_text(encoding="utf-8")
        app_source = (
            REPO_ROOT / "zircon_app/src/entry/engine_entry.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("crate::plugin", devtools_source)
        self.assertNotIn("RuntimePluginDescriptor", devtools_source)
        self.assertIn("devtools_plugin_catalog_entries", devtools_source)
        self.assertIn("replace_devtools_plugin_catalog_entries", app_source)
        self.assertIn("RuntimePluginDescriptor::builtin_catalog", app_source)

    def test_scene_execution_is_not_part_of_neutral_navigation_or_physics_managers(self) -> None:
        navigation_manager = (
            REPO_ROOT / "zircon_runtime/src/core/framework/navigation/manager.rs"
        ).read_text(encoding="utf-8")
        physics_manager = (
            REPO_ROOT / "zircon_runtime/src/core/framework/physics/manager.rs"
        ).read_text(encoding="utf-8")
        scene_navigation = (
            REPO_ROOT / "zircon_runtime/src/scene/navigation.rs"
        ).read_text(encoding="utf-8")
        plugin_navigation = (
            REPO_ROOT / "zircon_plugins/navigation/runtime/src/manager.rs"
        ).read_text(encoding="utf-8")
        plugin_physics = (
            REPO_ROOT / "zircon_plugins/physics/runtime/src/manager/service.rs"
        ).read_text(encoding="utf-8")

        for manager_source in (navigation_manager, physics_manager):
            self.assertNotIn("crate::scene", manager_source)
            self.assertNotIn("&World", manager_source)
            self.assertNotIn("&mut World", manager_source)
        self.assertNotIn("bake_surface", navigation_manager)
        self.assertNotIn("tick_world_agent", navigation_manager)
        self.assertNotIn("tick_scene_world", physics_manager)
        self.assertIn("trait SceneNavigationRuntime", scene_navigation)
        self.assertIn("impl SceneNavigationRuntime for DefaultNavigationManager", plugin_navigation)
        self.assertIn("pub(crate) fn tick_scene_world", plugin_physics)

    def test_project_export_and_plugin_selection_schema_has_one_neutral_owner(self) -> None:
        neutral_owner = (
            REPO_ROOT / "zircon_runtime/src/core/framework/project/mod.rs"
        )
        plugin_root = (
            REPO_ROOT / "zircon_runtime/src/plugin/mod.rs"
        ).read_text(encoding="utf-8")
        self.assertTrue(neutral_owner.is_file())
        self.assertFalse(
            (REPO_ROOT / "zircon_runtime/src/plugin/export_profile.rs").exists()
        )
        self.assertFalse(
            any(
                (
                    REPO_ROOT / "zircon_runtime/src/plugin/project_plugin_manifest"
                ).rglob("*.rs")
            )
        )
        for retired_export in (
            "ExportProfile",
            "ExportPackagingStrategy",
            "ProjectPluginManifest",
            "ProjectPluginSelection",
            "RuntimeProfileId",
        ):
            self.assertNotIn(retired_export, plugin_root)

        for relative_path in (
            "zircon_runtime/src/asset/project/manifest/export_profiles.rs",
            "zircon_runtime/src/asset/project/manifest/project_manifest.rs",
        ):
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertNotIn("crate::plugin", source)
            self.assertIn("core::framework::project", source)


if __name__ == "__main__":
    unittest.main()
