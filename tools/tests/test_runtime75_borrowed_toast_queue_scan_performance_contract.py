import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/toast.rs"


def rust_function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        character = source[index]
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated Rust function: {signature}")


class BorrowedToastQueueScanPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_sync_materializes_only_the_selected_toast(self) -> None:
        sync = rust_function_body(self.source, "fn sync_toast_state(")

        self.assertIn("scan_toast_queue", sync)
        self.assertNotIn("toast_entries", sync)
        self.assertIn("struct BorrowedToastEntry<'a>", self.source)
        self.assertIn("fn into_display_entry(self)", self.source)

    def test_nested_queue_scan_uses_a_borrowed_visitor(self) -> None:
        visitor = rust_function_body(self.source, "fn visit_toast_entries")

        self.assertIn("visit_toast_entries(value, visitor)", visitor)
        self.assertNotIn(".flat_map(", visitor)
        self.assertNotIn(".collect", visitor)
        self.assertNotIn("fn toast_entry_list(", self.source)

    def test_expiry_clones_only_retained_raw_values(self) -> None:
        expiry = rust_function_body(self.source, "fn expire_toast(")

        self.assertIn("visit_toast_entries", expiry)
        self.assertEqual(expiry.count("entry.raw.to_owned_value()"), 1)
        self.assertNotIn("toast_entries(state, descriptor)", expiry)


if __name__ == "__main__":
    unittest.main()
