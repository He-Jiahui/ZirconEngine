from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
PRIMITIVES = ROOT / "zircon_editor/src/ui/retained_host/primitives.rs"
PATCH_MAP = ROOT / "zircon_editor/src/ui/retained_host/persistent_row_patch_map.rs"


class ModelRcOrderedPatchCursorPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.primitives = PRIMITIVES.read_text(encoding="utf-8")
        cls.patch_map = PATCH_MAP.read_text(encoding="utf-8")

    def test_patch_map_exposes_forward_and_reverse_ordered_cursors(self):
        self.assertIn("fn forward_cursor(&self)", self.patch_map)
        self.assertIn("fn reverse_cursor(&self)", self.patch_map)
        self.assertIn("struct PersistentRowPatchCursor", self.patch_map)

    def test_cursor_frontier_uses_fixed_stack_storage(self):
        self.assertIn("struct PendingPatchStack", self.patch_map)
        cursor = self.patch_map[
            self.patch_map.index("struct PendingPatchStack") :
            self.patch_map.index("impl<T> PersistentRowPatchMap")
        ]
        self.assertIn(
            "[Option<&'a PersistentRowPatchNode<T>>; PATCH_CURSOR_CAPACITY]", cursor
        )
        self.assertIn("occupied: usize", cursor)
        stack = cursor[: cursor.index("pub(super) struct PersistentRowPatchCursor")]
        self.assertNotRegex(stack, re.compile(r"\b(?:Vec|Box|Rc|Arc)<"))

    def test_overlay_iterators_own_ordered_cursors(self):
        model_iter = self.primitives[
            self.primitives.index("enum ModelIter") :
            self.primitives.index("impl<'a, T> Iterator for ModelIter")
        ]
        self.assertGreaterEqual(
            model_iter.count("PersistentRowPatchCursor<'a, T>"),
            4,
            "both overlay variants need independent forward and reverse cursors",
        )

    def test_model_iterator_concrete_type_does_not_leak_private_cursor(self):
        self.assertNotIn("pub(crate) enum ModelIter", self.primitives)
        self.assertIn(
            "pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator",
            self.primitives,
        )

    def test_overlay_next_paths_do_not_restart_trie_lookup_for_each_row(self):
        iterator_impl = self.primitives[
            self.primitives.index("impl<'a, T> Iterator for ModelIter") :
            self.primitives.index("impl<T> DoubleEndedIterator for ModelIter")
        ]
        reverse_impl = self.primitives[
            self.primitives.index("impl<T> DoubleEndedIterator for ModelIter") :
            self.primitives.index("impl<T> ExactSizeIterator for ModelIter")
        ]
        self.assertNotRegex(iterator_impl, re.compile(r"patches\s*\.get\(row\)"))
        self.assertNotRegex(reverse_impl, re.compile(r"patches\s*\.get\(\*back\)"))
        self.assertGreaterEqual(iterator_impl.count("value_at(row)"), 2)
        self.assertGreaterEqual(reverse_impl.count("value_at(*back)"), 2)

    def test_model_equality_short_circuits_shared_storage_before_row_iteration(self):
        equality = self.primitives[
            self.primitives.index("impl<T: PartialEq> PartialEq for ModelRc<T>") :
            self.primitives.index("impl<T: Clone> From<Rc<VecModel<T>>> for ModelRc<T>")
        ]
        shared_identity = equality.index("self.shares_values_with(other)")
        deep_compare = equality.index("self.iter().eq(other.iter())")
        self.assertLess(shared_identity, deep_compare)


if __name__ == "__main__":
    unittest.main()
