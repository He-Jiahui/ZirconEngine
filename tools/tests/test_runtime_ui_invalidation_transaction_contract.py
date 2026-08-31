from pathlib import Path
import unittest

from tools.runtime_ui_invalidation_transaction_pressure import run


ROOT = Path(__file__).resolve().parents[2]
INVALIDATION = ROOT / "zircon_runtime/src/ui/surface/invalidation.rs"
SURFACE = ROOT / "zircon_runtime/src/ui/surface/surface.rs"


class RuntimeUiInvalidationTransactionContractTests(unittest.TestCase):
    def test_surface_consumes_validated_transaction_without_clone_vector(self):
        invalidation = INVALIDATION.read_text(encoding="utf-8")
        surface = SURFACE.read_text(encoding="utf-8")
        apply_body = surface.split("pub fn apply_invalidation_transaction", 1)[1].split(
            "pub(crate) fn set_runtime_style_index", 1
        )[0]

        self.assertIn("fn into_changes", invalidation)
        self.assertIn("for change in transaction.into_changes()", apply_body)
        self.assertNotIn("cloned().collect::<Vec<_>>()", apply_body)

    def test_pressure_model_counts_transaction_owned_work_only(self):
        result = run(change_count=16_384, transaction_count=120)

        self.assertEqual(result["old_change_clones"], 1_966_080)
        self.assertEqual(result["new_change_clones"], 0)
        self.assertEqual(result["old_temporary_vectors"], 120)
        self.assertEqual(result["new_temporary_vectors"], 0)

    def test_model_rejects_empty_inputs(self):
        with self.assertRaises(ValueError):
            run(0, 1)
        with self.assertRaises(ValueError):
            run(1, 0)


if __name__ == "__main__":
    unittest.main()
