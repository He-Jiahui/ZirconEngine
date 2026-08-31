import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
QUERY_ACCESS = (
    REPO_ROOT / "zircon_runtime" / "src" / "scene" / "ecs" / "query" / "query_access.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", source)
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class SingleWriteConflictProbePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = QUERY_ACCESS.read_text(encoding="utf-8")

    def test_add_write_reuses_the_read_conflict_probe_as_its_insertion_position(self) -> None:
        body = function_body(self.source, "add_write")
        self.assertEqual(body.count("binary_search("), 1)
        self.assertIn("self.reads.binary_search(&component_id)", body)
        self.assertIn("Err(index) => index", body)
        self.assertIn("self.reads.insert(read_index, component_id);", body)
        self.assertNotIn("contains_id(", body)

    def test_add_write_preserves_the_write_subset_of_reads_invariant(self) -> None:
        body = function_body(self.source, "add_write")
        self.assertIn("self.reads.insert(read_index, component_id);", body)
        self.assertIn("insert_id(&mut self.writes, component_id);", body)

    def test_rust_regressions_cover_direct_and_merged_access_sets(self) -> None:
        self.assertIn(
            "fn runtime60_batch_writes_remain_mirrored_in_reads()", self.source
        )
        self.assertIn(
            "fn runtime60_batch_merged_writes_remain_mirrored_in_reads()", self.source
        )
        self.assertIn(
            "fn runtime60_batch_repeated_write_keeps_the_conflict_diagnostic()",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
