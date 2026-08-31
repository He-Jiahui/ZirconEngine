from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TRANSITION = (
    ROOT
    / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection/transition_metadata"
)


class EditorInertTransitionProjectionContractTests(unittest.TestCase):
    def test_empty_kind_returns_before_state_timing_and_direction_projection(self) -> None:
        source = (TRANSITION / "mod.rs").read_text(encoding="utf-8")
        function = source.split("pub(super) fn projected_transition_metadata", 1)[1]

        kind = function.index("projected_transition_kind")
        empty_guard = function.index("if kind.is_empty()")
        early_return = function.index("return ProjectedTransitionMetadata::without_transition")
        transition_in = function.index("projected_transition_in")
        duration = function.index("projected_transition_duration_ms")
        direction = function.index("projected_transition_direction")

        self.assertLess(kind, empty_guard)
        self.assertLess(empty_guard, early_return)
        self.assertLess(early_return, transition_in)
        self.assertLess(early_return, duration)
        self.assertLess(early_return, direction)

    def test_no_transition_record_owns_no_timing_strings(self) -> None:
        source = (TRANSITION / "model.rs").read_text(encoding="utf-8")
        constructor = source.split("fn without_transition", 1)[1]

        self.assertIn("easing: String::new()", constructor)
        self.assertIn("direction: String::new()", constructor)
        self.assertIn("active: true", constructor)
        self.assertIn("entered: true", constructor)
        self.assertIn("progress: 1.0", constructor)
        self.assertIn("duration_ms: 0", constructor)


if __name__ == "__main__":
    unittest.main()
