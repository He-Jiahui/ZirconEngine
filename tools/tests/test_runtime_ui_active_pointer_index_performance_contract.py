import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class RuntimeUiActivePointerIndexPerformanceContractTests(unittest.TestCase):
    def test_active_pointer_table_uses_an_indexed_pointer_lookup(self):
        source = (
            ROOT / "zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("use std::collections::HashMap;", source)
        self.assertIn(
            "indices_by_pointer_id: HashMap<UiPointerId, usize>",
            source,
        )
        self.assertIn(
            "self.indices_by_pointer_id.get(&pointer_id).copied()",
            source,
        )
        self.assertNotIn(".iter().find(|entry| entry.pointer_id == pointer_id)", source)
        self.assertNotIn(
            ".iter_mut().find(|entry| entry.pointer_id == pointer_id)",
            source,
        )

    def test_active_pointer_index_is_updated_for_insert_remove_and_clear(self):
        source = (
            ROOT / "zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs"
        ).read_text(encoding="utf-8")
        implementation = source.split("#[cfg(test)]", 1)[0]
        self.assertIn("self.indices_by_pointer_id.insert(pointer_id, index);", implementation)
        self.assertIn("self.indices_by_pointer_id.remove(&pointer_id)?", implementation)
        self.assertIn("self.indices_by_pointer_id.clear();", implementation)
        self.assertIn(
            ".insert(moved_entry.pointer_id, moved_index)",
            implementation,
        )

    def test_pointer_entries_keep_stable_order_when_an_entry_is_removed(self):
        source = (
            ROOT / "zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs"
        ).read_text(encoding="utf-8")
        implementation = source.split("#[cfg(test)]", 1)[0]
        self.assertIn("self.entries.remove(index)", implementation)
        self.assertNotIn("self.entries.swap_remove(index)", implementation)


if __name__ == "__main__":
    unittest.main()
