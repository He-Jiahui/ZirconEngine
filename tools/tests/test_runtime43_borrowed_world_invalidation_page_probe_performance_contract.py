import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/dynamic_api/session/world_sync.rs"


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


class BorrowedWorldInvalidationPageProbePerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_size_probes_serialize_borrowed_pending_batches(self) -> None:
        search = rust_function_body(
            self.source, "fn build_largest_world_invalidation_page("
        )

        self.assertIn("BorrowedWorldInvalidationPage", self.source)
        self.assertIn("impl Serialize for BorrowedWorldInvalidationPage", self.source)
        self.assertIn("encode_borrowed_world_invalidation_page_at", search)

    def test_owned_page_materializes_once_after_search(self) -> None:
        search = rust_function_body(
            self.source, "fn build_largest_world_invalidation_page("
        )

        self.assertEqual(search.count("build_world_invalidation_page(pending,"), 1)
        loop = rust_function_body(search, "while low <= high")
        self.assertNotIn("build_world_invalidation_page", loop)

    def test_borrowed_probe_preserves_tail_queue_wire_order(self) -> None:
        serializer = self.source[
            self.source.index("Serialize for BorrowedWorldInvalidationItems") :
        ]

        self.assertIn("self.items.iter().rev().take(self.len)", serializer)
        self.assertIn("serialize_element(item)?", serializer)


if __name__ == "__main__":
    unittest.main()
