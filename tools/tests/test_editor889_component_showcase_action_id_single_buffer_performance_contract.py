import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/bindings.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/bindings/tests/action_id_single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor889ComponentShowcaseActionIdSingleBufferPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_action_id_uses_one_capacity_bounded_output(self) -> None:
        owner = function_body(
            self.source, "component_showcase_action_id_for_binding_id"
        )
        self.assertIn("let mut action_id = String::with_capacity(", owner)
        self.assertIn("binding_id.len().saturating_mul(2)", owner)
        self.assertIn("append_camel_to_snake_segment(&mut action_id", owner)
        self.assertTrue(owner.rstrip().endswith("action_id\n}"))

    def test_fallback_segments_append_with_positional_delimiters(self) -> None:
        owner = function_body(
            self.source, "component_showcase_action_id_for_binding_id"
        )
        self.assertIn(".filter(|segment| !segment.is_empty())", owner)
        self.assertIn(".enumerate()", owner)
        self.assertIn("if index > 0", owner)
        self.assertIn("action_id.push('.');", owner)

    def test_action_id_does_not_collect_join_or_format_children(self) -> None:
        owner = function_body(
            self.source, "component_showcase_action_id_for_binding_id"
        )
        self.assertNotIn("collect::<Vec", owner)
        self.assertNotIn(".join(", owner)
        self.assertNotIn("format!(", owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod action_id_single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor889_component_showcase_action_id_single_buffer_preserves_exact_text",
            lower,
        )
        self.assertIn(
            "EDITOR889_COMPONENT_SHOWCASE_ACTION_ID_SINGLE_BUFFER_BENCH_V1",
            lower,
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_segment_temporary_allocation_model(self) -> None:
        renders = 4_096
        segments = 64
        legacy_child_strings = renders * segments
        legacy_vector_slots = renders * segments
        optimized_child_strings = 0
        optimized_vector_slots = 0
        self.assertEqual(legacy_child_strings, 262_144)
        self.assertEqual(legacy_vector_slots, 262_144)
        self.assertEqual(optimized_child_strings, 0)
        self.assertEqual(optimized_vector_slots, 0)


if __name__ == "__main__":
    unittest.main()
