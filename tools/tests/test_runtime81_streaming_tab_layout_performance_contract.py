import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE_PATH = REPO_ROOT / "zircon_runtime/src/text/layout/tab.rs"


def production_source() -> str:
    return SOURCE_PATH.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]


def function_body(source: str, signature: str, next_signature: str) -> str:
    body = source.split(signature, 1)[1]
    return body.split(next_signature, 1)[0]


class StreamingTabLayoutPerformanceContract(unittest.TestCase):
    def test_grapheme_validation_does_not_materialize_a_temporary_vector(self) -> None:
        source = production_source()
        self.assertIn("fn has_matching_tab_graphemes(", source)
        self.assertRegex(source, r"text\s*\.graphemes\(true\)\s*\.fold\(")
        self.assertNotIn("collect::<Vec<_>>()", source)
        self.assertNotIn("let graphemes", source)

    def test_advance_path_only_allocates_the_final_output_vector(self) -> None:
        source = production_source()
        body = function_body(
            source,
            "pub(crate) fn tab_aligned_advances(",
            "pub(crate) fn tab_aligned_width(",
        )
        self.assertIn("Vec::with_capacity(advances.len())", body)
        self.assertIn("text.graphemes(true).zip(advances.iter().copied())", body)
        self.assertNotIn("collect::<", body)

    def test_width_path_accumulates_directly_without_an_output_vector(self) -> None:
        source = production_source()
        body = function_body(
            source,
            "pub(crate) fn tab_aligned_width(",
            "pub(crate) fn tab_interval_width(",
        )
        self.assertIn("for (grapheme, advance) in", body)
        self.assertIn("cursor += resolved_advance;", body)
        self.assertIn("cursor", body)
        self.assertNotIn("tab_aligned_advances", body)
        self.assertNotIn("Vec::", body)
        self.assertNotIn("collect::<", body)


if __name__ == "__main__":
    unittest.main()
