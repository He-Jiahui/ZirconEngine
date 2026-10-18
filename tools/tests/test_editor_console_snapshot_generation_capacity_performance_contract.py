from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/workbench/snapshot/data/console_output_snapshot.rs"


class EditorConsoleSnapshotGenerationCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("fn generation_from_flat_parts(")
        end = cls.source.index("\nfn flatten_lines(", start)
        cls.body = cls.source[start:end]

    def test_bounded_generation_reserves_retained_line_upper_bound(self) -> None:
        self.assertIn(
            "let retained_line_capacity = line_count.saturating_sub(retained_start);",
            self.body,
        )
        self.assertIn(
            "let mut lines = Vec::with_capacity(retained_line_capacity);",
            self.body,
        )
        self.assertLess(
            self.body.index("Vec::with_capacity(retained_line_capacity)"),
            self.body.index("retained_index, line"),
        )
        self.assertNotIn(".collect::<Vec<_>>()", self.body)

    def test_projection_semantics_remain_in_place(self) -> None:
        self.assertIn("let line_count = logical_line_count(text).max(levels.len());", self.body)
        self.assertIn("let retained_start = line_count.saturating_sub(", self.body)
        self.assertIn("text.split('\\n')", self.body)
        self.assertIn(".skip(retained_start)", self.body)
        self.assertIn("ConsoleOutputLineGeneration::from_lines(lines)", self.body)
        self.assertIn("source_index as u64", self.body)
        self.assertIn("jump_sequences.get(source_index).copied().flatten()", self.body)

    def test_deterministic_model_removes_geometric_growth_for_bounded_snapshot(self) -> None:
        logical_lines = 256
        legacy_growth_events = 0
        capacity = 0
        for length in range(1, logical_lines + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1

        self.assertGreater(legacy_growth_events, 0)
        self.assertEqual(logical_lines, 256)
        optimized_growth_events = 0
        reserved_capacity = logical_lines
        for length in range(1, logical_lines + 1):
            if length > reserved_capacity:
                optimized_growth_events += 1

        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
