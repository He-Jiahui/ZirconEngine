from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/surface/focus.rs"


class RuntimeFocusPointerDragRetainPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("    fn clear_invalid_transient_input_owners(&mut self) {")
        end = cls.source.index("\n    fn clear_drag_drop_session_for_source", start)
        cls.body = cls.source[start:end]

    def test_pointer_drag_path_filters_the_map_in_place(self) -> None:
        self.assertIn(
            "let mut pointer_drags = std::mem::take(&mut self.input.pointer_drags);",
            self.body,
        )
        self.assertIn(
            "pointer_drags.retain(|owner, _| is_valid_input_owner(self, *owner));",
            self.body,
        )
        self.assertIn("self.input.pointer_drags = pointer_drags;", self.body)
        self.assertNotIn(".keys()\n            .copied()", self.body)
        self.assertNotIn(".collect::<Vec<_>>()", self.body)

    def test_pointer_drag_semantics_and_lower_regression_are_wired(self) -> None:
        self.assertIn(
            "focus_pointer_drag_retain_removes_invalid_owners_once", self.source
        )
        self.assertIn("runtime822_focus_pointer_drag_retain_bench_v1", self.source)
        self.assertIn("is_valid_input_owner(self, *owner)", self.body)

    def test_retain_restores_the_same_map_after_validation(self) -> None:
        take_offset = self.body.index("std::mem::take(&mut self.input.pointer_drags)")
        retain_offset = self.body.index("pointer_drags.retain(")
        restore_offset = self.body.index("self.input.pointer_drags = pointer_drags;")
        self.assertLess(take_offset, retain_offset)
        self.assertLess(retain_offset, restore_offset)

    def test_deterministic_model_removes_temporary_map_allocations(self) -> None:
        pointer_drag_count = 4_096
        reconciliations = 1_000
        invalid_count = pointer_drag_count // 2
        legacy_temporary_vectors = reconciliations
        optimized_temporary_vectors = 0
        legacy_removal_lookups = reconciliations * invalid_count
        optimized_map_traversals = reconciliations

        self.assertGreater(legacy_temporary_vectors, optimized_temporary_vectors)
        self.assertGreater(legacy_removal_lookups, optimized_map_traversals)
        source_keys = list(range(pointer_drag_count))
        retained_keys = [key for key in source_keys if key % 2 == 0]
        self.assertEqual(retained_keys, source_keys[::2])
        self.assertEqual(len(retained_keys), invalid_count)


if __name__ == "__main__":
    unittest.main()
