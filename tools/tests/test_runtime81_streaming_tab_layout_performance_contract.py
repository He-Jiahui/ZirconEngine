import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE_PATH = REPO_ROOT / "zircon_runtime/src/text/layout/tab.rs"
MEASURE_PATH = REPO_ROOT / "zircon_runtime/src/text/layout/measure.rs"


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
        self.assertIn("cursor.add(resolved_advance);", body)
        self.assertIn("cursor", body)
        self.assertNotIn("tab_aligned_advances", body)
        self.assertNotIn("Vec::", body)
        self.assertNotIn("collect::<", body)

    def test_public_width_fuses_tab_admission_with_the_streaming_accumulation(self) -> None:
        source = production_source()
        body = function_body(
            source,
            "pub(crate) fn tab_aligned_width(",
            "/// Streams a tabbed sequence whose grapheme count and tab presence were already validated.",
        )

        self.assertIn("let mut raw_cursor", body)
        self.assertIn("let mut tabbed_cursor", body)
        self.assertIn("let Some(advance) = advances.next()", body)
        self.assertIn("if advances.next().is_some()", body)
        self.assertNotIn("has_matching_tab_graphemes", body)
        self.assertNotIn("Vec::", body)

    def test_prevalidated_measurement_width_skips_a_second_grapheme_count_pass(self) -> None:
        source = production_source()
        measure = MEASURE_PATH.read_text(encoding="utf-8")
        body = function_body(
            source,
            "pub(super) fn tab_aligned_width_for_matching_graphemes(",
            "pub(crate) fn tab_interval_width(",
        )

        self.assertIn("tab_aligned_width_for_matching_graphemes", measure)
        self.assertNotRegex(measure, r"\btab_aligned_width\(")
        self.assertIn("for (grapheme, advance) in", body)
        self.assertNotIn("has_matching_tab_graphemes", body)
        self.assertNotIn("Vec::", body)

    def test_release_width_gate_keeps_allocation_and_latency_evidence_executable(self) -> None:
        source = SOURCE_PATH.read_text(encoding="utf-8")

        self.assertIn("optimization_batch_20260827bl_streaming_tab_layout_p95", source)
        self.assertIn("RUNTIME81_STREAMING_TAB_LAYOUT_BENCH_V1", source)
        self.assertIn("const RELEASE_GRAPHEME_COUNT: usize = 131_072;", source)
        self.assertIn("const RELEASE_SAMPLE_COUNT: usize = 31;", source)
        self.assertIn("const RELEASE_WARMUP_COUNT: usize = 5;", source)
        self.assertRegex(source, r"#\[ignore\s*=\s*\"release-only performance evidence\"\]")
        self.assertIn("optimized_p50 * 100 <= legacy_p50 * 30", source)
        self.assertIn("optimized_p95 * 2 <= legacy_p95", source)


if __name__ == "__main__":
    unittest.main()
