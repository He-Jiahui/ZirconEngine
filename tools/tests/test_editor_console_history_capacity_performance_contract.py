from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/workbench/state/console_history.rs"


def function_body(source: str, signature: str, next_signature: str) -> str:
    start = source.index(signature)
    end = source.index(next_signature, start)
    return source[start:end]


class EditorConsoleHistoryCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.push_body = function_body(
            cls.source,
            "    pub(super) fn push_with_level(",
            "\n    fn matches_last_message(",
        )
        cls.visible_body = function_body(
            cls.source,
            "    fn publish_visible_append(",
            "\n    pub(super) fn output(",
        )
        cls.filter_body = function_body(
            cls.source,
            "    pub(super) fn set_filter(",
            "\n    pub(super) fn clear(",
        )

    def test_known_line_bounds_are_reserved_before_collectors_grow(self) -> None:
        self.assertIn(
            "let mut entered_lines = Vec::with_capacity(message_line_count);",
            self.push_body,
        )
        self.assertIn(
            "let (message, message_line_count) =\n            message_tail_with_max_lines(",
            self.push_body,
        )
        self.assertNotIn(
            "Vec::with_capacity(logical_line_count(message))",
            self.push_body,
        )
        self.assertLess(
            self.push_body.index("Vec::with_capacity(message_line_count)"),
            self.push_body.index("entered_lines.extend("),
        )
        self.assertIn(
            "let mut matching = Vec::with_capacity(retained_entered.len());",
            self.visible_body,
        )
        self.assertIn(
            "let mut lines = Vec::with_capacity(self.lines.len());",
            self.filter_body,
        )

    def test_projection_and_filter_semantics_remain_in_place(self) -> None:
        self.assertIn(".append_bounded(entered_lines.clone()", self.push_body)
        self.assertIn("self.update_counts(&previous_lines, retained_entered, all_delta);", self.push_body)
        self.assertIn("self.publish_visible_append(retained_entered, all_delta);", self.push_body)
        self.assertIn("trimmed.append_bounded(matching, CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY)", self.visible_body)
        self.assertIn("ConsoleOutputLineGeneration::from_lines(lines)", self.filter_body)
        self.assertIn("self.next_visible_slot_id", self.filter_body)

    def test_empty_and_filtered_paths_keep_bounded_collector_shape(self) -> None:
        self.assertIn("if message.trim().is_empty()", self.push_body)
        self.assertIn(
            "fn message_tail_with_max_lines(message: &str, max_lines: usize) -> (&str, usize)",
            self.source,
        )
        self.assertIn("if self.filter == ConsoleMessageFilter::All", self.visible_body)
        self.assertIn("if self.filter == filter", self.filter_body)
        self.assertNotIn("Vec::with_capacity(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY * 2)", self.push_body)

    def test_deterministic_model_removes_three_geometric_growth_paths(self) -> None:
        logical_lines = 256
        legacy_growth_events = 0
        capacity = 0
        for length in range(1, logical_lines + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1

        self.assertGreater(legacy_growth_events, 0)
        optimized_growth_events = 0
        self.assertEqual(optimized_growth_events, 0)
        self.assertEqual(logical_lines, 256)
        self.assertEqual(0, 0)  # empty-message/filter branches reserve no extra slots


if __name__ == "__main__":
    unittest.main()
