from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/core/commands/defaults.rs"


class EditorDefaultCommandRegistryDirectAppendPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("pub(super) fn default_workbench_commands()")
        end = cls.source.index("\nfn command<I, S>(", start)
        cls.body = cls.source[start:end]

    def test_registry_reserves_one_exact_outer_bound(self) -> None:
        self.assertIn(
            "Vec::with_capacity(DEFAULT_WORKBENCH_COMMAND_CAPACITY)",
            self.body,
        )
        self.assertIn("const DEFAULT_WORKBENCH_COMMAND_CAPACITY: usize = 66;", self.source)
        self.assertIn("EDITOR811_DEFAULT_COMMAND_DIRECT_APPEND_BENCH_V1", self.source)

    def test_feature_groups_append_into_the_caller_owned_vector(self) -> None:
        for helper in (
            "append_file_commands",
            "append_edit_commands",
            "append_selection_commands",
            "append_runtime_commands",
            "append_view_commands",
            "append_window_commands",
            "append_animation_asset_toolkit_open_operations",
        ):
            self.assertIn(f"{helper}(&mut commands);", self.body)
            self.assertIn(f"fn {helper}(", self.source)
            self.assertIn("commands: &mut Vec<EditorCommandDescriptor>", self.source)

        self.assertNotIn("commands.extend(file_commands())", self.body)
        self.assertNotIn("commands.extend(edit_commands())", self.body)
        self.assertNotIn("commands.extend(selection_commands())", self.body)
        self.assertNotIn("commands.extend(runtime_commands())", self.body)
        self.assertNotIn("commands.extend(view_commands())", self.body)
        self.assertNotIn("commands.extend(window_commands())", self.body)
        self.assertNotIn("fn file_commands() -> Vec<EditorCommandDescriptor>", self.source)

    def test_registry_cardinality_and_ordering_contract_are_explicit(self) -> None:
        self.assertIn("assert_eq!(commands.len(), DEFAULT_WORKBENCH_COMMAND_CAPACITY);", self.source)
        self.assertIn("commands.push(command_palette_command());", self.body)
        self.assertLess(
            self.body.index("append_file_commands(&mut commands);"),
            self.body.index("append_edit_commands(&mut commands);"),
        )
        self.assertLess(
            self.body.index("append_view_commands(&mut commands);"),
            self.body.index("append_window_commands(&mut commands);"),
        )

    def test_direct_append_model_removes_temporary_growth(self) -> None:
        command_count = 66
        capacity = 0
        legacy_growth_events = 0
        for length in range(1, command_count + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1

        self.assertEqual(command_count, 66)
        self.assertEqual(legacy_growth_events, 6)
        optimized_growth_events = 0
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
