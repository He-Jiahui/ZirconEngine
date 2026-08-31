from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
VALUE_SYNC = ROOT / (
    "zircon_runtime/src/asset/assets/material/material_asset/value_sync.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime09CSingleEntryTextureSlotSyncPerformanceContractTests(
    unittest.TestCase
):
    def test_some_texture_uses_one_btree_entry_lookup(self) -> None:
        source = VALUE_SYNC.read_text(encoding="utf-8")
        sync = function_region(
            source,
            "pub(super) fn sync_texture_slot(",
            "pub(super) fn sync_f32_override(",
        )

        self.assertIn("use std::collections::btree_map::Entry;", source)
        self.assertEqual(sync.count("slots.entry(slot.to_string())"), 1)
        self.assertIn("Entry::Occupied(mut entry)", sync)
        self.assertIn("Entry::Vacant(entry)", sync)
        self.assertNotIn("slots.get(slot)", sync)
        self.assertNotIn("slots.insert(slot.to_string(), value)", sync)

    def test_occupied_entry_reads_metadata_once_then_replaces_the_value(self) -> None:
        source = VALUE_SYNC.read_text(encoding="utf-8")
        sync = function_region(
            source,
            "pub(super) fn sync_texture_slot(",
            "pub(super) fn sync_f32_override(",
        )

        self.assertEqual(sync.count("entry.get()"), 1)
        self.assertIn("previous.fallback.clone()", sync)
        self.assertIn("previous.transform", sync)
        self.assertIn("previous.texture_uv_channel()", sync)
        self.assertIn("entry.insert(value);", sync)

    def test_entry_paths_are_covered_by_rust(self) -> None:
        source = VALUE_SYNC.read_text(encoding="utf-8")

        self.assertIn(
            "fn texture_slot_sync_preserves_existing_slot_metadata()",
            source,
        )
        self.assertIn("fn texture_slot_sync_inserts_a_new_slot()", source)


if __name__ == "__main__":
    unittest.main()
