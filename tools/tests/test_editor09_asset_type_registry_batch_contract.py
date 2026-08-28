from __future__ import annotations

import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
REGISTRY = REPO_ROOT / "zircon_editor/src/core/asset/type_registry/registry.rs"
BATCH = REPO_ROOT / "zircon_editor/src/core/asset/type_registry/registry/batch.rs"
EXTENSION_MATERIALIZATION = (
    REPO_ROOT / "zircon_editor/src/core/plugin/extension_materialization.rs"
)
MATERIALIZATION_TESTS = (
    REPO_ROOT / "zircon_editor/src/tests/editor_asset_type_registry/materialization.rs"
)


class Editor09AssetTypeRegistryBatchContractTests(unittest.TestCase):
    def test_registry_routes_single_and_catalog_contributions_through_batch_core(self) -> None:
        registry = REGISTRY.read_text(encoding="utf-8")
        batch = BATCH.read_text(encoding="utf-8")

        self.assertIn("mod batch;", registry)
        self.assertIn("pub(crate) fn apply_contributions", registry)
        self.assertIn("apply_contributions([(owner, contribution)])", registry)
        self.assertIn("struct PendingEntryDelta", batch)
        self.assertIn("fn finalize_pending_entries", batch)
        self.assertIn(
            "context_command_entry_count += entry.definition.context_commands.len()",
            batch,
        )
        self.assertIn(
            "creation_template_entry_count += entry.definition.creation_templates.len()",
            batch,
        )
        self.assertNotIn("binary_search_by", batch)

    def test_catalog_materializes_one_batch_and_restores_traversal_diagnostic_order(self) -> None:
        materialization = EXTENSION_MATERIALIZATION.read_text(encoding="utf-8")

        self.assertIn("asset_type_contributions", materialization)
        self.assertIn("asset_types.apply_contributions", materialization)
        self.assertIn("diagnostics.sort_by_key", materialization)
        self.assertNotIn("asset_types.apply_contribution(", materialization)

    def test_scale_contract_rejects_the_old_per_contribution_generation_expectation(self) -> None:
        tests = MATERIALIZATION_TESTS.read_text(encoding="utf-8")

        self.assertIn("for contribution_count in [1, 100, 10_000, 100_000]", tests)
        self.assertIn("context_command_sort_count()", tests)
        self.assertIn("context_command_entry_count()", tests)
        self.assertNotIn("before_generation + 1_000", tests)


if __name__ == "__main__":
    unittest.main()
