import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_PATH = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "scene"
    / "viewport"
    / "pointer"
    / "candidates"
    / "renderable_candidates.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*[<(]", source)
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


class EditorAdaptiveRenderableOwnerDedupPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE_PATH.read_text(encoding="utf-8")

    def test_candidate_projection_uses_adaptive_owner_admission(self) -> None:
        body = function_body(self.source, "renderable_candidates")
        self.assertIn("Vec::with_capacity(render_meshes.len())", body)
        self.assertIn("admit_renderable_owner", body)
        self.assertNotIn("HashSet::", body)
        self.assertIn("use std::collections::HashSet", self.source)

    def test_owner_index_is_created_only_after_order_decreases(self) -> None:
        body = function_body(self.source, "admit_renderable_owner")
        compact = re.sub(r"\s+", "", body)
        self.assertIn("seen_owners.is_none()", body)
        self.assertIn("owner<*previous", compact)
        self.assertIn("collect::<HashSet<_>>()", compact)
        self.assertIn("seen_owners.as_mut()", body)

    def test_grouped_owner_regression_keeps_the_fallback_unallocated(self) -> None:
        self.assertIn(
            "grouped_owner_sequence_keeps_lazy_index_unallocated",
            self.source,
        )
        self.assertIn("assert!(seen_owners.is_none())", self.source)

    def test_interleaved_owner_regression_collapses_non_adjacent_duplicates(self) -> None:
        self.assertIn(
            "interleaved_owner_sequence_collapses_non_adjacent_duplicates",
            self.source,
        )
        self.assertIn("assert!(seen_owners.is_some())", self.source)


if __name__ == "__main__":
    unittest.main()
