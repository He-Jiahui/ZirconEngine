from __future__ import annotations

import re
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

    def test_manager_services_use_versioned_handles_without_legacy_arc_holders(self) -> None:
        manager_root = REPO_ROOT / "zircon_runtime/src/core/manager"
        service_source = (manager_root / "service.rs").read_text(encoding="utf-8")
        resolver_source = (manager_root / "resolver.rs").read_text(encoding="utf-8")
        manager_mod_source = (manager_root / "mod.rs").read_text(encoding="utf-8")
        runtime_resolution_source = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/handle/resolution.rs"
        ).read_text(encoding="utf-8")
        runtime_handle_source = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/handle/core_handle.rs"
        ).read_text(encoding="utf-8")
        service_entry_source = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/state/service_entry.rs"
        ).read_text(encoding="utf-8")
        resolution_test_source = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/tests/resolution/behavior.rs"
        ).read_text(encoding="utf-8")
        manager_resolution_contract_source = (
            REPO_ROOT / "zircon_runtime/tests/frameworks05_manager_resolution_contract.rs"
        ).read_text(encoding="utf-8")
        scene_module_source = (
            REPO_ROOT / "zircon_runtime/src/scene/module/mod.rs"
        ).read_text(encoding="utf-8")
        error_source = (
            REPO_ROOT / "zircon_runtime/src/core/framework/error.rs"
        ).read_text(encoding="utf-8")
        content_download_state_source = (
            REPO_ROOT
            / "zircon_plugins/net/features/content_download/runtime/src/manager/state.rs"
        ).read_text(encoding="utf-8")

        for required in (
            "pub struct ManagerServiceHandle<T: ?Sized>",
            "pub index: u32",
            "pub generation: u32",
            "pub service: RegistryName",
            "pub struct RegisteredManagerService",
            "pub trait ManagerServiceResolver",
        ):
            self.assertIn(required, service_source)
        self.assertIn("StaleServiceHandle", error_source)
        self.assertIn("ServiceUnavailable", error_source)
        self.assertIn("manager_service_handle(core, $service_name)", resolver_source)
        self.assertIn("ensure_service_resolution_available", runtime_resolution_source)
        self.assertIn("wait_for_service_resolution_change", runtime_resolution_source)
        self.assertIn("notify_service_resolution_changed", runtime_resolution_source)
        self.assertIn("initialization_owner", service_entry_source)
        self.assertIn("try_register_service_resolution_wait", runtime_handle_source)
        self.assertIn("clear_service_resolution_wait", runtime_handle_source)
        self.assertIn("service_activation_reentries", runtime_handle_source)
        self.assertIn(
            "concurrent_cyclic_lazy_manager_dependencies_return_without_deadlock",
            resolution_test_source,
        )
        self.assertIn(
            "direct_immediate_service_resolution_reuses_module_activation_instance",
            manager_resolution_contract_source,
        )
        self.assertIn("core: Option<CoreWeak>", content_download_state_source)
        self.assertNotIn("core: Option<CoreHandle>", content_download_state_source)
        self.assertNotIn(
            "pub use crate::core::framework",
            manager_mod_source,
            "core::manager must not alias neutral framework trait owners",
        )
        self.assertNotIn("pub const LEVEL_MANAGER_NAME", scene_module_source)

        legacy_symbols = (
            "RenderingManagerHandle",
            "RenderFrameworkHandle",
            "LevelManagerHandle",
            "ResourceManagerHandle",
            "InputManagerHandle",
            "InputActionManagerHandle",
            "ConfigManagerHandle",
            "EventManagerHandle",
            "AiManagerHandle",
            "NetManagerHandle",
            "PhysicsManagerHandle",
            "AnimationManagerHandle",
            "SoundManagerHandle",
            "NavigationManagerHandle",
            "resolve_rendering_manager",
            "resolve_render_framework",
            "resolve_level_manager",
            "resolve_resource_manager",
            "resolve_input_manager",
            "resolve_input_action_manager",
            "resolve_config_manager",
            "resolve_event_manager",
            "resolve_ai_manager",
            "resolve_net_manager",
            "resolve_physics_manager",
            "resolve_animation_manager",
            "resolve_sound_manager",
            "resolve_navigation_manager",
        )
        for legacy in legacy_symbols:
            self.assertNotIn(legacy, manager_mod_source)
            self.assertNotIn(legacy, resolver_source)

        manager_trait = (
            "RenderingManager|RenderFramework|LevelManager|ResourceManager|InputManager|"
            "InputActionManager|ConfigManager|EventManager|AiManager|NetManager|"
            "PhysicsManager|AnimationManager|SoundManager|NavigationManager"
        )
        stored_arc = re.compile(
            rf"^\s*[A-Za-z_][A-Za-z0-9_]*\s*:\s*(?:Option<)?Arc<dyn (?:{manager_trait})>"
        )
        storage_paths = (
            "zircon_runtime/src/dynamic_api/session.rs",
            "zircon_runtime/src/dynamic_api/session/state.rs",
            "zircon_runtime/src/dynamic_api/runtime_loop.rs",
            "zircon_editor/src/ui/retained_host/app.rs",
            "zircon_editor/src/ui/retained_host/viewport/viewport_state.rs",
            "zircon_plugins/net/features/content_download/runtime/src/manager/state.rs",
        )
        for relative_path in storage_paths:
            lines = (REPO_ROOT / relative_path).read_text(encoding="utf-8").splitlines()
            for index, line in enumerate(lines):
                previous = lines[index - 1].strip() if index else ""
                if previous == "#[cfg(test)]":
                    continue
                self.assertIsNone(
                    stored_arc.search(line),
                    f"production manager Arc storage remains at {relative_path}:{index + 1}",
                )

    def test_asset_manager_consumers_use_versioned_handles_at_use_points(self) -> None:
        retired_owners = (
            REPO_ROOT
            / "zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager_handle.rs",
            REPO_ROOT
            / "zircon_runtime/src/asset/pipeline/manager/asset_manager/resolve_asset_manager.rs",
        )
        for retired_owner in retired_owners:
            self.assertFalse(retired_owner.exists())

        production_roots = (
            REPO_ROOT / "zircon_runtime/src",
            REPO_ROOT / "zircon_app/src",
            REPO_ROOT / "zircon_editor/src",
            REPO_ROOT / "zircon_plugins",
        )
        legacy_symbols = (
            "AssetManagerHandle",
            "IntoProjectAssetManagerAccess",
            "resolve_asset_manager",
            "resolve_manager::<ProjectAssetManager>",
        )
        violations = []
        for root in production_roots:
            for path in root.rglob("*.rs"):
                relative = path.relative_to(REPO_ROOT).as_posix()
                if "/tests/" in f"/{relative}/" or "/test_" in relative:
                    continue
                source = path.read_text(encoding="utf-8")
                for symbol in legacy_symbols:
                    if symbol in source:
                        violations.append(f"{relative}: {symbol}")
        self.assertEqual(
            violations,
            [],
            "asset manager legacy resolution remains:\n" + "\n".join(violations),
        )

        project_access_source = (
            REPO_ROOT
            / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "pub struct ProjectAssetManagerAccess {\n    core: CoreWeak",
            project_access_source,
        )
        self.assertNotIn(
            "pub struct ProjectAssetManagerAccess {\n    core: CoreHandle",
            project_access_source,
        )
        self.assertNotIn("fn standalone(", project_access_source)

        manager_resolver_source = (
            REPO_ROOT / "zircon_runtime/src/core/manager/resolver.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "pub struct ManagerResolver {\n    core: CoreWeak", manager_resolver_source
        )
        self.assertNotIn(
            "pub struct ManagerResolver {\n    core: CoreHandle", manager_resolver_source
        )

        concrete_level_manager_violations = []
        concrete_level_manager_pattern = re.compile(
            r"(?:use\s+[^;]*\bDefaultLevelManager\b|"
            r"resolve_manager::<DefaultLevelManager>|"
            r"(?:Arc|Weak)<DefaultLevelManager>)"
        )
        for root in production_roots:
            for path in root.rglob("*.rs"):
                relative = path.relative_to(REPO_ROOT).as_posix()
                if relative.startswith("zircon_runtime/src/scene/"):
                    continue
                if (
                    "/tests/" in f"/{relative}/"
                    or "/test_support/" in f"/{relative}/"
                    or "/test_sources/" in f"/{relative}/"
                    or relative.endswith("_tests.rs")
                    or relative.endswith("/tests.rs")
                ):
                    continue
                source = path.read_text(encoding="utf-8")
                if concrete_level_manager_pattern.search(source):
                    concrete_level_manager_violations.append(relative)
        self.assertEqual(
            concrete_level_manager_violations,
            [],
            "cross-domain concrete LevelManager consumers remain:\n"
            + "\n".join(concrete_level_manager_violations),
        )

        graphics_storage = re.compile(
            r"^\s*[A-Za-z_][A-Za-z0-9_]*\s*:\s*(?:Option<)?Arc<ProjectAssetManager>"
        )
        graphics_violations = []
        for path in (REPO_ROOT / "zircon_runtime/src/graphics").rglob("*.rs"):
            relative = path.relative_to(REPO_ROOT).as_posix()
            if (
                "/tests/" in f"/{relative}/"
                or "/test_support/" in f"/{relative}/"
                or "/test_sources/" in f"/{relative}/"
                or relative.endswith("_tests.rs")
            ):
                continue
            lines = path.read_text(encoding="utf-8").splitlines()
            for index, line in enumerate(lines):
                if graphics_storage.search(line):
                    preceding = "\n".join(lines[max(0, index - 5) : index])
                    if "#[cfg(test)]" in preceding:
                        continue
                    graphics_violations.append(f"{relative}:{index + 1} {line.strip()}")
        self.assertEqual(
            graphics_violations,
            [],
            "graphics stores long-lived concrete asset managers:\n"
            + "\n".join(graphics_violations),
        )

    def test_editor_manager_consumers_keep_explicit_versioned_resolution(self) -> None:
        retained_assets = (
            REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("use super::RetainedEditorHost;", retained_assets)
        self.assertIn(
            "EditorAssetManager as EditorAssetManagerContract", retained_assets
        )
        self.assertIn("impl RetainedEditorHost", retained_assets)
        self.assertIn(".resolve(self.asset_manager.clone())", retained_assets)
        self.assertIn(".resolve(self.editor_asset_manager.clone())", retained_assets)
        self.assertIn(".resolve(self.resource_manager.clone())", retained_assets)

        editor_render_fixture = (
            REPO_ROOT / "zircon_editor/src/tests/editing/state.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("ProjectAssetManagerAccess::new(core, handle)", editor_render_fixture)
        self.assertIn("manager_service_handle(&core, SERVICE_NAME)", editor_render_fixture)
        self.assertIn("RegisteredManagerService::new", editor_render_fixture)
        self.assertNotIn(
            "WgpuRenderFramework::new(asset_manager)", editor_render_fixture
        )

    def test_editor_creates_levels_through_scene_owner_without_concrete_manager(self) -> None:
        project_access = (
            REPO_ROOT / "zircon_editor/src/ui/host/project_access.rs"
        ).read_text(encoding="utf-8")
        self.assertNotIn("DefaultLevelManager", project_access)
        self.assertNotIn("resolve_manager::<", project_access)
        self.assertIn("zircon_runtime::scene::create_level", project_access)


if __name__ == "__main__":
    unittest.main()
