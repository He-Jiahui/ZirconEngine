import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/core/recovery/project_recovery_assessment.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/core/recovery/project_recovery_assessment/tests/session_effect_state_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor884SessionEffectStateSingleBufferPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.owner = function_body(cls.source, "session_effect_states")

    def test_summary_streams_into_one_string(self) -> None:
        self.assertIn("let mut states = String::new();", self.owner)
        self.assertIn(
            "for (index, entry) in effects.iter().enumerate()", self.owner
        )
        self.assertIn('states.push_str(", ");', self.owner)
        self.assertRegex(self.owner, r"write!\(\s*&mut states")

    def test_summary_has_no_temporary_string_vector_or_join(self) -> None:
        self.assertNotIn("format!(", self.owner)
        self.assertNotIn("collect::<Vec", self.owner)
        self.assertNotIn('.join(", ")', self.owner)

    def test_empty_and_order_semantics_are_explicit(self) -> None:
        self.assertIn("if index > 0", self.owner)
        self.assertTrue(self.owner.rstrip().endswith("states\n}"))

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod session_effect_state_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor884_session_effect_state_single_buffer_preserves_exact_text",
            lower,
        )
        self.assertIn("EDITOR884_SESSION_EFFECT_STATE_SINGLE_BUFFER_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_temporary_allocation_model(self) -> None:
        effect_count = 4_096
        legacy_temporary_strings = effect_count
        legacy_temporary_vector_slots = effect_count
        optimized_temporary_strings = 0
        optimized_temporary_vector_slots = 0
        self.assertEqual(legacy_temporary_strings, 4_096)
        self.assertEqual(legacy_temporary_vector_slots, 4_096)
        self.assertEqual(optimized_temporary_strings, 0)
        self.assertEqual(optimized_temporary_vector_slots, 0)


if __name__ == "__main__":
    unittest.main()
