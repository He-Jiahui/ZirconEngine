from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RUN_ALIGNMENT = ROOT / "zircon_runtime/src/text/rich/parser/run_alignment.rs"
PERFORMANCE_TESTS = (
    ROOT / "zircon_runtime/src/text/rich/parser/performance_tests.rs"
)


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime84GraphemeAlignmentPerformanceContractTests(unittest.TestCase):
    def test_ascii_alignment_reuses_canonical_runs(self) -> None:
        source = RUN_ALIGNMENT.read_text(encoding="utf-8")
        alignment = function_body(
            source,
            "fn align_runs_to_graphemes(",
            "fn ascii_runs_are_canonical(",
        )

        self.assertIn("if ascii_runs_are_canonical(text, runs) {", alignment)
        self.assertIn("return Ok(runs.to_vec());", alignment)
        self.assertIn("Vec::with_capacity(runs.len().min(max_runs))", alignment)
        self.assertIn("fn ascii_runs_are_canonical(", source)
        self.assertIn("!text.is_ascii()", source)

    def test_unicode_alignment_clones_metadata_only_for_new_output_runs(self) -> None:
        source = RUN_ALIGNMENT.read_text(encoding="utf-8")
        alignment = function_body(
            source,
            "fn align_runs_to_graphemes(",
            "fn ascii_runs_are_canonical(",
        )

        self.assertIn("source_metadata_matches(previous, source)", alignment)
        self.assertIn("clone_source_run(start, end, source)", alignment)
        self.assertNotIn(".cloned()", alignment)
        self.assertNotIn("push_or_merge_run(&mut aligned", alignment)
        self.assertIn("fn source_metadata_matches(", source)
        self.assertIn("fn clone_source_run(", source)

    def test_release_benchmark_tracks_metadata_clone_collapse(self) -> None:
        self.assertTrue(PERFORMANCE_TESTS.is_file())
        benchmark = PERFORMANCE_TESTS.read_text(encoding="utf-8")

        self.assertIn("RUNTIME84_GRAPHEME_ALIGNMENT_PERF", benchmark)
        self.assertIn("legacy_ascii_metadata_clones=250000", benchmark)
        self.assertIn("optimized_ascii_metadata_clones=128", benchmark)
        self.assertIn("legacy_unicode_metadata_clones=50000", benchmark)
        self.assertIn("optimized_unicode_metadata_clones=1", benchmark)


if __name__ == "__main__":
    unittest.main()
