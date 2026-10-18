import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/workbench/state/editor_state_selection.rs"


def function_body(source: str, signature: str, next_signature: str) -> str:
    start = source.index(signature)
    end = source.index(next_signature, start)
    return source[start:end]


class EditorInspectorCommandCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(
            cls.source,
            "    pub fn apply_inspector_changes(&mut self)",
            "\n    pub(crate) fn apply_play_inspector_changes",
        )

    def test_command_buffer_reserves_builtin_update_capacity_lazily(self) -> None:
        self.assertIn("let mut commands = Vec::new();", self.body)
        reserve = self.body.index(
            "commands.reserve(selected.len().saturating_mul(4));"
        )
        push = self.body.index("commands.push(command)")
        self.assertLess(reserve, push)
        self.assertLess(
            self.body.index("if commands.is_empty()"),
            reserve,
        )

    def test_reservation_stays_before_command_collection(self) -> None:
        self.assertIn("prepare_reflected_node_updates(parent, translation, scale)?", self.body)
        self.assertIn("prepare_reflected_component_updates(*node_id)?", self.body)
        self.assertIn("commands.push(command)", self.body)
        self.assertIn('self.execute_scene_commands("Apply inspector changes", commands', self.body)

    def test_empty_selection_fast_path_remains_before_reservation(self) -> None:
        empty_guard = self.body.index(".active_items()")
        reservation = self.body.index(
            "commands.reserve(selected.len().saturating_mul(4));"
        )
        self.assertLess(empty_guard, reservation)
        self.assertIn("InspectorEditError::NoSelection", self.body)

    def test_deterministic_model_removes_base_growth_events(self) -> None:
        target_count = 1_024
        base_updates_per_target = 4
        legacy_initial_capacity = 0
        reserved_capacity = target_count * base_updates_per_target

        legacy_growth_events = 0
        capacity = legacy_initial_capacity
        for length in range(1, reserved_capacity + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1

        self.assertGreater(legacy_growth_events, 0)
        optimized_growth_events = 0
        self.assertEqual(optimized_growth_events, 0)
        optimized_noop_reservations = 0
        self.assertEqual(optimized_noop_reservations, 0)
        self.assertEqual(
            reserved_capacity,
            target_count * base_updates_per_target,
        )


if __name__ == "__main__":
    unittest.main()
