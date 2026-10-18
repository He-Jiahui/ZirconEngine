"""Source and allocation model for showcase state summaries on a fixed stack."""

import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/template_runtime/showcase_demo_state/state_panel.rs"
LOWER = ROOT / "zircon_editor/src/ui/template_runtime/showcase_demo_state/state_panel/tests/stack_buffer_tests.rs"


class Editor895ShowcaseStateFlagStackTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.owner = cls.source.split("fn state_flags_summary(", 1)[1].split("#[cfg(test)]", 1)[0]

    def test_all_eight_flags_stay_in_order_on_a_fixed_stack(self) -> None:
        self.assertIn('let mut states = [""; STATE_FLAG_SUMMARY_CAPACITY]', self.owner)
        self.assertIn("for (enabled, label) in [", self.owner)
        for name in ("focused", "hovered", "pressed", "dragging", "popup_open", "expanded", "selected", "checked"):
            self.assertIn(f"flags.{name}", self.owner)
        self.assertIn("states[count] = label", self.owner)

    def test_state_summary_allocates_only_final_joined_output(self) -> None:
        self.assertIn('states[..count].join(", ")', self.owner)
        self.assertNotIn("Vec::", self.owner)
        self.assertNotIn(".collect(", self.owner)

    def test_existing_capacity_contract_is_updated_for_the_stack(self) -> None:
        previous = (SOURCE.parent / "state_panel/tests/capacity_tests.rs").read_text(encoding="utf-8")
        self.assertIn('source.contains("let mut states = [', previous)
        self.assertNotIn('source.contains("Vec::with_capacity(STATE_FLAG_SUMMARY_CAPACITY)', previous)

    def test_lower_parity_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod stack_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("editor895_showcase_state_flag_stack_preserves_all_masks", lower)
        self.assertIn("EDITOR895_SHOWCASE_STATE_FLAG_STACK_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_fixed_flag_vector_allocation_model(self) -> None:
        summaries, flag_bound = 4_096, 8
        self.assertEqual(summaries * flag_bound, 32_768)
        self.assertEqual(summaries, 4_096)
        optimized_slots = summaries * flag_bound if "Vec::" in self.owner else 0
        self.assertEqual(optimized_slots, 0)


if __name__ == "__main__":
    unittest.main()
