from pathlib import Path
import unittest

from tools.editor_pointer_frame_cadence_refresh_pressure import (
    pressure_report,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]
POINTER_DISPATCH = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/workbench/pointer.rs"
)
POINTER_FEEDBACK = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/pointer_feedback.rs"
)
TEMPLATE_SURFACE = ROOT / (
    "zircon_editor/src/ui/workbench/reference/template_surface.rs"
)
EVENT_LOOP_REDRAW = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs"
)
UNREAL_INVALIDATION_ROOT = ROOT / (
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/"
    "SlateInvalidationRoot.cpp"
)


class EditorPointerFrameCadenceRefreshPressureTests(unittest.TestCase):
    def test_four_changed_events_per_frame_coalesce_refresh_pipeline(self):
        report = pressure_report(1_000, 4)

        current = report["current_event_owned_refresh"]
        target = report["frame_cadence_publication"]
        self.assertEqual(report["frame_count"], 250)
        self.assertEqual(current["surface_rebuild_dirty_count"], 1_000)
        self.assertEqual(current["expensive_refresh_stage_executions"], 2_000)
        self.assertEqual(target["surface_rebuild_dirty_count"], 250)
        self.assertEqual(target["expensive_refresh_stage_executions"], 500)
        self.assertEqual(target["coalesced_visual_write_count"], 750)
        self.assertEqual(report["comparison"]["refresh_stage_reduction_ratio"], 4.0)
        self.assertFalse(report["is_product_timing"])

    def test_non_divisible_event_count_uses_terminal_frame(self):
        report = pressure_report(1_000, 17)

        self.assertEqual(report["frame_count"], 59)
        self.assertEqual(
            report["frame_cadence_publication"]["published_visual_state_count"],
            59,
        )
        self.assertEqual(
            report["comparison"]["avoided_surface_refreshes"], 941
        )
        self.assertGreater(
            report["comparison"]["refresh_stage_reduction_ratio"], 16.9
        )

    def test_rejects_non_positive_inputs(self):
        for values in ((0, 1), (1, 0), (-1, 1), (1, -1)):
            with self.subTest(values=values):
                with self.assertRaises(ValueError):
                    pressure_report(*values)

    def test_artifact_output_rejects_the_system_drive(self):
        with self.assertRaises(ValueError):
            validate_output_path(r"C:\zircon-profiles\pointer-cadence.json")
        self.assertEqual(
            validate_output_path(r"E:\zircon-profiles\pointer-cadence.json").drive.upper(),
            "E:",
        )

    def test_model_is_bound_to_current_event_owned_refresh_before_redraw_merge(self):
        dispatch = POINTER_DISPATCH.read_text(encoding="utf-8")
        feedback = POINTER_FEEDBACK.read_text(encoding="utf-8")
        surface = TEMPLATE_SURFACE.read_text(encoding="utf-8")
        redraw = EVENT_LOOP_REDRAW.read_text(encoding="utf-8")

        route = dispatch.index("bridge.route_pointer_event(event)")
        refresh = dispatch.index("bridge.refresh_pointer_feedback")
        self.assertLess(route, refresh)
        self.assertIn("feedback_deferred_count", feedback)
        self.assertNotIn("refresh_after_state_change(self.runtime.as_ref())", feedback)
        self.assertIn("self.surface.rebuild_dirty(self.layout_size)?", surface)
        self.assertIn(
            "self.refresh_projection(runtime, &workset, report.layout_recomputed)",
            surface,
        )
        self.assertIn("frames_extract_skip_count", surface)
        self.assertIn("workbench_surface_extract_frames", surface)

        tooltip = (ROOT / "zircon_editor/src/ui/retained_host/app/workbench_tooltip.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("effects.request_workbench_projection();", tooltip)
        self.assertNotIn("and_then(|changed|", tooltip)

        feedback = (ROOT / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/pointer_feedback.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("feedback_deferred_count", feedback)
        self.assertNotIn("refresh_after_state_change(self.runtime.as_ref())", feedback)
        self.assertIn("let existing = std::mem::replace", redraw)
        self.assertIn("self.pending_redraw = existing.merge(redraw)", redraw)

    def test_unreal_queues_typed_widget_work_for_the_fast_path(self):
        unreal = UNREAL_INVALIDATION_ROOT.read_text(encoding="utf-8")
        self.assertIn("FSlateInvalidationRoot::InvalidateWidget", unreal)
        self.assertIn("PaintFastPath_AddUniqueSortedToFinalUpdateList", unreal)
        self.assertIn("FSlateInvalidationRoot::PaintFastPath", unreal)


if __name__ == "__main__":
    unittest.main()
