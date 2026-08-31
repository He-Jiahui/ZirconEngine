from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/selection_defaults/catalog_selections.rs"
)


class PreallocatedCatalogSelectionCompletionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_selection_id_index_uses_complete_upper_bound(self) -> None:
        self.assertIn("let selection_capacity = completed", self.source)
        self.assertIn(".selections\n        .len()", self.source)
        self.assertIn(".saturating_add(registrations.len())", self.source)
        self.assertIn(
            "let mut selected_package_ids = HashSet::with_capacity(selection_capacity);",
            self.source,
        )
        self.assertNotIn("collect::<HashSet<_>>()", self.source)

    def test_completed_selection_vector_reserves_registration_bound(self) -> None:
        self.assertIn("completed.selections.reserve(registrations.len());", self.source)
        self.assertIn("for selection in &completed.selections", self.source)
        self.assertIn("selected_package_ids.insert(selection.id.clone());", self.source)

    def test_rust_guard_preserves_order_and_disabled_default_contract(self) -> None:
        self.assertIn(
            "preallocated_catalog_selection_completion_preserves_behavior_contract",
            self.source,
        )
        self.assertIn("for registration in registrations", self.source)
        self.assertIn("selection.enabled = false;", self.source)
        self.assertIn("completed.selections.push(selection);", self.source)


if __name__ == "__main__":
    unittest.main()
