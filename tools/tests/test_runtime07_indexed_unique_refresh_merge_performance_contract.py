from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/work.rs"
)
SIGNATURE = "pub(super) fn merge(&mut self, later: Self)"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class IndexedUniqueRefreshMergePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(cls.source, SIGNATURE)

    def test_refresh_uses_action_index_before_scanning_notification_order(self) -> None:
        indexed_guard = "if current_actions.remove(&path).is_some() {"
        duplicate_scan = "current_order.retain(|current| current != &path);"
        self.assertEqual(self.body.count(indexed_guard), 1)
        self.assertEqual(self.body.count(duplicate_scan), 1)
        self.assertLess(self.body.index(indexed_guard), self.body.index(duplicate_scan))
        self.assertNotIn("current_actions.remove(&path);", self.body)

    def test_merged_path_still_has_one_order_owner_and_one_action_owner(self) -> None:
        self.assertIn("current_order.push(path.clone());", self.body)
        self.assertIn("current_actions.insert(path, action);", self.body)

    def test_rust_guards_cover_unique_append_and_duplicate_reordering(self) -> None:
        self.assertIn(
            "unique_refreshes_append_without_rescanning_existing_order",
            self.source,
        )
        self.assertIn(
            "duplicate_refresh_moves_path_to_latest_notification_position",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
