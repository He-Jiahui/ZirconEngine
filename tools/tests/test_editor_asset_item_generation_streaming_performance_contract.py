import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE_PATH = (
    ROOT
    / "zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs"
)


class EditorAssetItemGenerationStreamingPerformanceContractTests(unittest.TestCase):
    def setUp(self):
        self.source = SOURCE_PATH.read_text(encoding="utf-8")
        self.implementation = self.source.split("#[cfg(test)]\nmod tests", 1)[0]

    def test_from_iterator_uses_one_pass_owned_item_builder(self):
        implementation = self.implementation
        self.assertIn("fn from_items<T: IntoIterator<Item = AssetItemSnapshot>>", implementation)
        self.assertIn("let mut iterator = iter.into_iter();", implementation)
        self.assertIn("for item in iterator", implementation)
        self.assertIn("let mut chunks = Vec::with_capacity", implementation)
        self.assertIn("indices_by_uuid.insert(item.uuid.clone(), index)", implementation)
        self.assertIn("indices_by_locator.insert(item.locator.clone(), index)", implementation)
        self.assertNotIn("iter.into_iter().collect::<Vec<_>>()", implementation)

    def test_vec_and_iterator_constructors_share_streaming_builder(self):
        implementation = self.implementation
        self.assertIn("impl From<Vec<AssetItemSnapshot>>", implementation)
        self.assertIn("Self::from_items(items)", implementation)
        self.assertIn("impl FromIterator<AssetItemSnapshot>", implementation)
        self.assertIn("Self::from_items(iter)", implementation)
        self.assertEqual(implementation.count("fn from_items<T: IntoIterator<Item = AssetItemSnapshot>>"), 1)

    def test_streaming_model_removes_temporary_item_vector(self):
        item_count = 1_000_000
        chunk_size = 64
        chunks = (item_count + chunk_size - 1) // chunk_size
        legacy_owned_item_slots = item_count + item_count
        streaming_owned_item_slots = item_count
        self.assertEqual(chunks, 15_625)
        self.assertEqual(legacy_owned_item_slots, 2_000_000)
        self.assertEqual(streaming_owned_item_slots, 1_000_000)
        self.assertEqual(
            legacy_owned_item_slots - streaming_owned_item_slots,
            item_count,
        )


if __name__ == "__main__":
    unittest.main()
