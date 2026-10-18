import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/text.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/text/tests/token_staging_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor882TableTextTokenStagingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.archived = function_body(cls.source, "split_archived_table_text")
        cls.size = function_body(cls.source, "normalize_size_cell")

    def test_archived_parser_uses_fixed_leading_token_slots(self) -> None:
        self.assertIn("let mut tokens = text.split_whitespace()", self.archived)
        self.assertIn("let leading = [", self.archived)
        self.assertEqual(self.archived.count("tokens.next()"), 6)

    def test_archived_parser_has_no_temporary_token_vector(self) -> None:
        self.assertNotIn("collect::<Vec", self.archived)
        self.assertNotIn("tokens.as_slice()", self.archived)

    def test_size_normalizer_uses_bounded_iterator_lookahead(self) -> None:
        self.assertIn("let mut parts = trimmed.split_whitespace()", self.size)
        self.assertEqual(self.size.count("parts.next()"), 3)
        self.assertNotIn("collect::<Vec", self.size)
        self.assertNotIn("parts.as_slice()", self.size)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod token_staging_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("editor882_table_text_token_staging_preserves_parser_priority", lower)
        self.assertIn("EDITOR882_TABLE_TEXT_TOKEN_STAGING_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_temporary_vector_model(self) -> None:
        row_count = 4_096
        legacy_temporary_vectors = row_count * 2
        optimized_temporary_vectors = 0
        self.assertEqual(legacy_temporary_vectors, 8_192)
        self.assertEqual(optimized_temporary_vectors, 0)


if __name__ == "__main__":
    unittest.main()
