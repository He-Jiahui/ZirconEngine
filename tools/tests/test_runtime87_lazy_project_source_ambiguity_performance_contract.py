from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/asset/project/manager/source_path_for_uri.rs"


def rust_function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*[^{{]*{{", source, re.DOTALL)
    if match is None:
        raise AssertionError(f"missing Rust function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function {name}")
    return source[match.end() : index - 1]


class LazyProjectSourceAmbiguityPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        cls.body = rust_function_body(source, "source_operation_path_for_project_uri")

    def test_unique_project_source_returns_the_first_match_without_collection(self) -> None:
        self.assertIn("let mut existing = self", self.body)
        self.assertNotIn("collect::<Vec<_>>()", self.body)
        self.assertRegex(
            self.body,
            r"let Some\(first\)\s*=\s*existing\.next\(\)\s*else",
        )
        self.assertIn("let Some(second) = existing.next() else", self.body)
        self.assertIn("return Ok(first)", self.body)

    def test_ambiguity_vector_is_allocated_only_after_a_second_match(self) -> None:
        second_match = self.body.index("let Some(second) = existing.next() else")
        allocation = self.body.index("let mut ambiguous = Vec::with_capacity")
        self.assertLess(second_match, allocation)
        self.assertIn("ambiguous.push(first)", self.body)
        self.assertIn("ambiguous.push(second)", self.body)
        self.assertIn("ambiguous.extend(existing)", self.body)

    def test_missing_and_ambiguous_error_contracts_are_preserved(self) -> None:
        self.assertIn("AssetImportError::MissingProjectAssetUri", self.body)
        self.assertIn("AssetImportError::ambiguous_project_asset_uri", self.body)
        self.assertIn("uri.clone()", self.body)
        self.assertIn("ambiguous", self.body)


if __name__ == "__main__":
    unittest.main()
