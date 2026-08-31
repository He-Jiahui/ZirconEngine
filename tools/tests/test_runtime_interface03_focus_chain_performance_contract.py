import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FOCUS = REPO_ROOT / "zircon_runtime_interface" / "src" / "ui" / "focus.rs"


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


class FocusChainPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.focus = FOCUS.read_text(encoding="utf-8")

    def test_focus_chain_partitions_default_and_indexed_candidates(self) -> None:
        body = function_body(self.focus, "focus_chain")
        self.assertIn("let mut default_candidates = Vec::new()", body)
        self.assertIn("let mut indexed_candidates = Vec::new()", body)
        self.assertIn("&mut default_candidates", body)
        self.assertIn("&mut indexed_candidates", body)
        self.assertNotIn("let mut candidates = Vec::new()", body)

    def test_only_explicit_tab_indices_are_sorted(self) -> None:
        body = function_body(self.focus, "finish_focus_chain")
        self.assertIn("indexed_candidates.sort_by_key", body)
        self.assertIn("default_candidates", body)
        self.assertNotIn("default_candidates.sort", body)
        collect = function_body(self.focus, "collect_focus_candidates")
        self.assertIn("Some(tab_index)", collect)
        self.assertIn("indexed_candidates.push", collect)
        self.assertIn("default_candidates.push(node_id)", collect)

    def test_release_evidence_tracks_sorted_candidate_reduction(self) -> None:
        self.assertIn(
            "PERF_RESULT runtime_interface03_focus_chain_partition",
            self.focus,
        )
        self.assertIn("legacy_sorted_candidates=10000", self.focus)
        self.assertIn("optimized_sorted_candidates=0", self.focus)


if __name__ == "__main__":
    unittest.main()
