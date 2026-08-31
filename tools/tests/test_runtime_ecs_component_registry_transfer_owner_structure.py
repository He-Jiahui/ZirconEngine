import unittest
from pathlib import Path


class RuntimeEcsComponentRegistryTransferOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_08_15_component_registry_transfer_owner_split_"
        "static_passed_cargo_deferred"
    )

    def test_transfer_transaction_is_child_owned_without_algorithm_drift(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        registry_path = (
            repo_root / "zircon_runtime/src/scene/ecs/component/registry.rs"
        )
        registry = registry_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(registry.splitlines()), 230)
        self.assertIn("mod transferred;", registry)
        self.assertIn("PreflightedTransferredDescriptorImports", registry)
        self.assertIn("TransferredComponentDescriptor", registry)
        self.assertIn('#[path = "registry/tests.rs"]', registry)
        for moved_anchor in (
            "struct PendingTransferredDescriptor",
            "fn preflight_transferred_descriptor_import",
            "fn preflight_dynamic_descriptor_import",
            "fn publish_preflighted_transferred_descriptor_imports",
            "fn descriptor_matches_transfer",
        ):
            self.assertNotIn(moved_anchor, registry)

        owner_dir = registry_path.with_suffix("")
        transferred = (owner_dir / "transferred.rs").read_text(encoding="utf-8")
        tests = (owner_dir / "tests.rs").read_text(encoding="utf-8")
        self.assertLessEqual(len(transferred.splitlines()), 310)
        self.assertLessEqual(len(tests.splitlines()), 220)

        self._assert_anchors_are_ordered(
            transferred,
            (
                "pub(crate) struct TransferredComponentDescriptor",
                "pub(crate) struct PreflightedTransferredDescriptorImports",
                "struct PendingTransferredDescriptor",
                "impl PreflightedTransferredDescriptorImports",
                "impl ComponentRegistry",
                "fn descriptor_matches_transfer",
            ),
        )
        for invariant in (
            "base_descriptor_count + self.pending.len()",
            "debug_assert_eq!(component_id.index(), self.descriptors.len())",
            "self.descriptors.reserve(imports.pending.len())",
            "existing.type_name == transferred.type_name",
            "existing.storage_type == transferred.storage_type",
            "existing.source == transferred.source",
        ):
            self.assertIn(invariant, transferred)

        for test_anchor in (
            "rust_table_components_receive_their_registered_dense_column_layout",
            "table_column_layout_batches_preserve_the_signature_component_order",
            "transferred_rust_table_descriptor_imports_its_column_layout_once",
            "dynamic_descriptor_preflight_reserves_one_unpublished_target_local_id",
            "transferred_descriptor_preflight_defers_import_until_publication",
            "transferred_descriptor_preflight_rejects_conflicts_without_mutating_the_base",
        ):
            self.assertIn(f"fn {test_anchor}()", tests)

    def test_runtime_08_inventory_and_status_mirrors_cover_the_new_owner(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        inventory_path = repo_root / (
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/"
            "scripts/runtime_structure_audits/ecs_kernel_data_source_inventory.py"
        )
        inventory = inventory_path.read_text(encoding="utf-8")
        self.assertIn("EXPECTED_SOURCE_FILE_COUNT = 77", inventory)
        self.assertIn(
            '"zircon_runtime/src/scene/ecs/component/registry/transferred.rs"',
            inventory,
        )

        runtime_inventory = (
            repo_root
            / "zircon_runtime/src/tests/runtime_absorption/ecs_kernel_data/inventory.rs"
        ).read_text(encoding="utf-8")
        self.assertIn('"src/scene/ecs/component/registry/transferred.rs"', runtime_inventory)
        self.assertIn("EXPECTED_RUNTIME_08_SOURCE_FILES.len(), 77", runtime_inventory)

        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        structure_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/scene/ecs/component/registry.rs",
            "zircon_runtime/src/scene/ecs/component/registry/transferred.rs",
            "zircon_runtime/src/scene/ecs/component/registry/tests.rs",
            "tools/tests/test_runtime_ecs_component_registry_transfer_owner_structure.py",
        ):
            self.assertIn(current_path, structure_plan)

    def _assert_anchors_are_ordered(self, source: str, anchors: tuple[str, ...]) -> None:
        positions = [source.index(anchor) for anchor in anchors]
        self.assertEqual(positions, sorted(positions))


if __name__ == "__main__":
    unittest.main()
