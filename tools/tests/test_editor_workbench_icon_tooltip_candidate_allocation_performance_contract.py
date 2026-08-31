from pathlib import Path
import unittest

from tools.editor_workbench_icon_tooltip_candidate_pressure import run


ROOT = Path(__file__).resolve().parents[2]
TOOLTIP = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/icon_tooltip.rs"
)
TOOLTIP_METADATA = ROOT / "zircon_editor/src/ui/template_runtime/workbench_tooltip.rs"


class EditorWorkbenchIconTooltipCandidateAllocationPerformanceContractTests(
    unittest.TestCase
):
    def test_pointer_move_compares_a_borrowed_candidate_before_ownership(self) -> None:
        source = TOOLTIP.read_text(encoding="utf-8")
        update = source.split(
            "pub(crate) fn update_workbench_icon_tooltip_candidate", 1
        )[1].split("pub(crate) fn next_workbench_icon_tooltip_delay", 1)[0]

        self.assertIn("IconTooltipTargetRef<'_>", source)
        self.assertIn("candidate.matches_owned", update)
        self.assertIn("candidate.map(IconTooltipTargetRef::into_owned)", update)
        self.assertLess(
            update.index("candidate.matches_owned"),
            update.index("candidate.map(IconTooltipTargetRef::into_owned)"),
        )
        self.assertIn("self.icon_tooltip_input.candidate = candidate;", update)
        self.assertNotIn("candidate.clone()", update)

    def test_tooltip_metadata_text_is_borrowed(self) -> None:
        parser = TOOLTIP_METADATA.read_text(encoding="utf-8")

        self.assertIn("Option<&str>", parser)
        self.assertNotIn("to_string()", parser)

    def test_tooltip_ancestor_lookup_does_not_materialize_a_bubble_route(self) -> None:
        source = TOOLTIP.read_text(encoding="utf-8")
        lookup = source.split("fn icon_tooltip_target_at_node", 1)[1].split(
            "fn icon_tooltip_target", 1
        )[0]

        self.assertIn("let mut node_id = Some(surface_node_id)", lookup)
        self.assertIn("while let Some(current_id) = node_id", lookup)
        self.assertNotIn("hit_test(", lookup)
        self.assertNotIn("bubble_route(", lookup)

    def test_pressure_model_bounds_allocations_by_candidate_changes(self) -> None:
        result = run(
            pointer_move_count=65_536,
            candidate_change_count=64,
            mean_bubble_depth=6,
            timer_tick_count=8,
        )

        self.assertEqual(
            result["retired_owned_candidate_path"]["allocation_site_executions"],
            196_824,
        )
        self.assertEqual(
            result["borrowed_candidate_path"]["allocation_site_executions"],
            192,
        )
        self.assertEqual(
            result["delta"]["avoided_allocation_site_executions"],
            196_632,
        )
        self.assertEqual(
            result["delta"]["allocation_site_execution_reduction_ratio"],
            1_025.12,
        )
        self.assertEqual(
            result["delta"]["avoided_bubble_route_element_writes"],
            393_216,
        )

    def test_tooltip_intro_tick_borrows_the_candidate_without_formatting_id(self) -> None:
        source = TOOLTIP.read_text(encoding="utf-8")
        tick = source.split("pub(crate) fn tick_workbench_icon_tooltip", 1)[
            1
        ].split("pub(crate) fn dismiss_workbench_icon_tooltip", 1)[0]

        self.assertIn("self.icon_tooltip_input.candidate.take()", tick)
        self.assertIn("self.icon_tooltip_input.candidate = candidate;", tick)
        self.assertIn("strip_prefix(TOOLTIP_ID_PREFIX)", tick)
        self.assertNotIn("candidate.clone()", tick)
        self.assertNotIn("candidate.tooltip_id()", tick)


if __name__ == "__main__":
    unittest.main()
