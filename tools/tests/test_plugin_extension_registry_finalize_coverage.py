import re
import unittest
from pathlib import Path


def _function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    open_brace = source.index("{", start)
    depth = 0
    for index in range(open_brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[open_brace + 1 : index]
    raise AssertionError(f"unterminated function body: {signature}")


class PluginExtensionRegistryFinalizeCoverageTests(unittest.TestCase):
    def test_all_typed_extension_points_participate_in_finalize_state(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        source = (
            repo_root
            / "zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs"
        ).read_text(encoding="utf-8")

        struct_body = source[
            source.index("pub struct RuntimeExtensionRegistry") : source.index(
                "impl RuntimeExtensionRegistry"
            )
        ]
        typed_fields = set(
            re.findall(
                r"pub\(super\)\s+(\w+)\s*:\s*TypedExtensionPoint",
                struct_body,
            )
        )
        finalize_fields = set(
            re.findall(
                r"self\.(\w+)\.freeze\(\)",
                _function_body(source, "pub fn finalize(&mut self)"),
            )
        )
        finalized_state_fields = set(
            re.findall(
                r"self\.(\w+)\.is_frozen\(\)",
                _function_body(source, "pub fn is_finalized(&self)"),
            )
        )

        self.assertEqual(20, len(typed_fields))
        self.assertEqual(typed_fields, finalize_fields)
        self.assertEqual(typed_fields, finalized_state_fields)

    def test_frozen_runtime_table_is_hash_free_and_owns_the_finalized_state(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        source = (
            repo_root
            / "zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs"
        ).read_text(encoding="utf-8")
        frozen_body = _function_body(source, "pub struct FrozenExtensionTable")

        self.assertNotIn("HashMap", frozen_body)
        self.assertIn("sorted_key_indices", frozen_body)
        self.assertIn("TypedExtensionState::Frozen(FrozenExtensionTable::from_staging", source)
        self.assertNotIn("frozen: bool", source)

    def test_stable_slot_does_not_expose_dense_index_compatibility_api(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        source = (
            repo_root
            / "zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs"
        ).read_text(encoding="utf-8")
        slot_impl = source[
            source.index("impl ExtensionSlot") : source.index(
                "#[derive(Clone, Debug)]\npub struct TypedExtensionPoint"
            )
        ]

        self.assertNotIn("pub fn index", slot_impl)

    def test_catalog_and_apply_boundaries_finalize_before_runtime_reads(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        finalize_boundaries = (
            (
                "zircon_runtime/src/plugin/extension_registry/apply_to_asset_manager.rs",
                "pub fn apply_asset_importers_to_project_asset_manager",
                "for importer in",
            ),
            (
                "zircon_runtime/src/plugin/extension_registry/apply_to_module/runtime_core.rs",
                "pub fn apply_to_module",
                "descriptor.managers.extend",
            ),
            (
                "zircon_runtime/src/plugin/extension_registry/apply_to_ui/component.rs",
                "pub fn apply_ui_components_to_registry",
                "for component in",
            ),
            (
                "zircon_runtime/src/plugin/extension_registry/apply_to_world.rs",
                "pub fn apply_to_world",
                "self.world_runtime_extension_plan()?",
            ),
            (
                "zircon_runtime/src/plugin/extension_registry/apply_to_world/component.rs",
                "pub fn apply_component_types_to_world",
                "self.apply_finalized_component_types_to_world",
            ),
        )
        catalog_boundaries = (
            (
                "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/runtime.rs",
                "fn runtime_extension_report",
                "RuntimeExtensionCatalogReport",
            ),
            (
                "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report.rs",
                "fn runtime_extension_report_for_project",
                "RuntimeExtensionCatalogReport",
            ),
        )

        for path, signature, first_read in finalize_boundaries:
            source = (repo_root / path).read_text(encoding="utf-8")
            body = _function_body(source, signature)
            self.assertLess(body.index("self.finalize();"), body.index(first_read), path)
        for path, signature, report_constructor in catalog_boundaries:
            source = (repo_root / path).read_text(encoding="utf-8")
            body = _function_body(source, signature)
            self.assertLess(
                body.index("registry.finalize();"),
                body.rindex(report_constructor),
                path,
            )

        registry_world_source = (
            repo_root
            / "zircon_runtime/src/plugin/extension_registry/apply_to_world.rs"
        ).read_text(encoding="utf-8")
        plan_body = _function_body(
            registry_world_source, "pub fn world_runtime_extension_plan"
        )
        for runtime_read in (
            "self.components()",
            "self.plugin_resources()",
            "self.plugin_events()",
            "self.plugin_systems()",
            "self.plugin_runtime_systems()",
        ):
            self.assertIn(runtime_read, plan_body)
        self.assertIn("WorldRuntimeExtensionPlan::from_registrations", plan_body)

        world_driver_path = repo_root / "zircon_runtime/src/scene/module/world_driver.rs"
        world_driver_source = world_driver_path.read_text(encoding="utf-8")
        install_body = _function_body(
            world_driver_source, "pub fn install_world_runtime_extension_plan"
        )
        apply_body = _function_body(
            world_driver_source, "pub fn apply_world_runtime_extensions"
        )
        snapshot_body = _function_body(
            world_driver_source, "fn runtime_extension_plan_snapshot"
        )
        self.assertLess(
            install_body.index("let candidate = plan.as_ref().try_merge(contribution)"),
            install_body.index("*plan = Arc::new(candidate);"),
        )
        self.assertLess(
            apply_body.index("let plan = self.runtime_extension_plan_snapshot();"),
            apply_body.index("plan.apply_to_world(world)"),
        )
        self.assertIn(
            "Arc::clone(&lock_poison_recovered(&self.runtime_extensions))",
            snapshot_body,
        )
        self.assertNotIn("lock_poison_recovered", apply_body)


if __name__ == "__main__":
    unittest.main()
