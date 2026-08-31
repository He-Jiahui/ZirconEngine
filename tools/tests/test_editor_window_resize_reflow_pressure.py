from pathlib import Path
import unittest

from tools.editor_window_resize_reflow_pressure import (
    pressure_report,
    pressure_suite,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]
RESIZE_EVENTS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/resize.rs"
)
EVENT_LOOP_LIFECYCLE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs"
)
EVENT_LOOP_REDRAW = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs"
)
EVENT_LOOP_STATE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs"
)
SHELL_METRICS = ROOT / (
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/shell_metrics.rs"
)
PRESENT_REDRAW = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs"
)
PROFILE_CAPTURE = ROOT / "tools/ui-profile-capture.ps1"
PROFILE_RESIZE = ROOT / "tools/ui-profile-native-resize.ps1"
UNREAL_SLATE_APPLICATION = ROOT / (
    "dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/"
    "SlateApplication.cpp"
)
UNREAL_WINDOW = ROOT / (
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp"
)


def function_body(source: str, signature: str, next_signature: str) -> str:
    return source.split(signature, 1)[1].split(next_signature, 1)[0]


class EditorWindowResizeReflowPressureTests(unittest.TestCase):
    def test_current_resize_path_commits_latest_geometry_at_the_redraw_boundary(self):
        resize = RESIZE_EVENTS.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
        lifecycle = EVENT_LOOP_LIFECYCLE.read_text(encoding="utf-8")
        redraw = EVENT_LOOP_REDRAW.read_text(encoding="utf-8")
        event_loop_state = EVENT_LOOP_STATE.read_text(encoding="utf-8")
        shell_metrics = SHELL_METRICS.read_text(encoding="utf-8").split(
            "#[cfg(test)]", 1
        )[0]
        present = PRESENT_REDRAW.read_text(encoding="utf-8")

        queue = function_body(resize, "fn queue_resize_frame", "pub(super) fn handle_window_moved")
        redraw_frame = function_body(
            redraw,
            "fn redraw_requested_impl",
            "fn take_pending_redraw",
        )

        self.assertNotIn("NATIVE_RESIZE_REFLOW_DEBOUNCE", resize)
        self.assertNotIn("pending_resize_reflow_deadline", event_loop_state)
        self.assertNotIn("schedule_due_resize_reflow", lifecycle)
        self.assertNotIn("defer_native_resize_reflow", queue)
        self.assertIn("self.pending_presenter_resize = Some", queue)
        self.assertIn("into_interactive_frame_update", queue)
        self.assertIn("pending_presenter_resize.is_some()", redraw_frame)
        configure = redraw_frame.index("self.apply_pending_presenter_resize(event_loop)")
        frame_update = redraw_frame.index("redraw.requires_frame_update()")
        present_call = redraw_frame.index("present_redraw(")
        self.assertLess(configure, frame_update)
        self.assertLess(frame_update, present_call)
        self.assertNotIn("native_resize_present", redraw_frame)
        self.assertNotIn("native_resize_reflow_pending", shell_metrics)
        self.assertNotIn("native_resize_reflow_pending", present)

    def test_rejected_debounce_exposes_a_sixty_three_frame_stale_geometry_window(self):
        report = pressure_report(
            resize_events=25,
            event_interval_ms=40.0,
            rejected_trailing_debounce_ms=80.0,
            frame_interval_ms=1000.0 / 60.0,
            semantic_nodes=10_000,
            total_layout_nodes=10_000,
            total_hit_entries=10_000,
            affected_layout_nodes=64,
            affected_hit_entries=64,
            damage_regions=8,
        )

        rejected = report["rejected_trailing_debounce"]
        self.assertEqual(rejected["mismatched_geometry_window_ms"], 1040.0)
        self.assertEqual(rejected["mismatched_geometry_frame_budgets"], 63)
        self.assertEqual(rejected["full_frame_redraw_requests"], 26)
        self.assertEqual(rejected["retained_geometry_commits"], 1)
        self.assertEqual(rejected["semantic_projection_visits"], 10_000)

    def test_frame_cadence_coalesces_metrics_without_semantic_projection(self):
        report = pressure_report(
            resize_events=25,
            event_interval_ms=40.0,
            rejected_trailing_debounce_ms=80.0,
            frame_interval_ms=1000.0 / 60.0,
            semantic_nodes=10_000,
            total_layout_nodes=10_000,
            total_hit_entries=10_000,
            affected_layout_nodes=64,
            affected_hit_entries=64,
            damage_regions=8,
        )

        target = report["frame_cadence_geometry_publication"]
        self.assertEqual(target["retained_geometry_commits"], 25)
        self.assertLessEqual(
            target["max_event_to_geometry_commit_ms"], 1000.0 / 60.0
        )
        self.assertEqual(target["semantic_projection_visits"], 0)
        self.assertEqual(target["geometry_operation_units"], 3_400)
        self.assertEqual(report["rejected_final_reflow"]["geometry_operation_units"], 30_008)
        self.assertFalse(report["is_product_timing"])

    def test_event_storm_bounds_geometry_commits_by_frame_cadence(self):
        report = pressure_report(
            resize_events=2_000,
            event_interval_ms=4.0,
            rejected_trailing_debounce_ms=80.0,
            frame_interval_ms=1000.0 / 60.0,
            semantic_nodes=10_000,
            total_layout_nodes=10_000,
            total_hit_entries=10_000,
            affected_layout_nodes=64,
            affected_hit_entries=64,
            damage_regions=8,
        )

        rejected = report["rejected_trailing_debounce"]
        target = report["frame_cadence_geometry_publication"]
        self.assertEqual(rejected["mismatched_geometry_window_ms"], 8_076.0)
        self.assertLess(target["retained_geometry_commits"], 500)
        self.assertGreater(target["coalesced_resize_events"], 1_500)
        self.assertEqual(target["semantic_projection_visits"], 0)

    def test_model_rejects_invalid_or_inconsistent_inputs(self):
        valid = [25, 40.0, 80.0, 1000.0 / 60.0, 10_000, 10_000, 10_000, 64, 64, 8]
        for index, invalid in (
            (0, 0),
            (1, 0.0),
            (2, -1.0),
            (3, 0.0),
            (4, 0),
            (5, 0),
            (6, 0),
            (7, -1),
            (8, -1),
            (9, 0),
        ):
            values = valid.copy()
            values[index] = invalid
            with self.subTest(index=index, invalid=invalid):
                with self.assertRaises(ValueError):
                    pressure_report(*values)

        with self.assertRaises(ValueError):
            pressure_report(*valid[:7], 10_001, 64, 8)
        with self.assertRaises(ValueError):
            pressure_report(*valid[:8], 10_001, 8)

    def test_artifact_output_rejects_the_system_drive(self):
        with self.assertRaises(ValueError):
            validate_output_path(r"C:\zircon-profiles\resize.json")
        self.assertEqual(
            validate_output_path(r"E:\zircon-profiles\resize.json").drive.upper(),
            "E:",
        )
        with self.assertRaises(ValueError):
            validate_output_path(r"G:\zircon-profiles\resize.json")

    def test_model_is_bound_to_current_zircon_and_unreal_resize_state_machines(self):
        resize = RESIZE_EVENTS.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
        lifecycle = EVENT_LOOP_LIFECYCLE.read_text(encoding="utf-8")
        redraw = EVENT_LOOP_REDRAW.read_text(encoding="utf-8")
        capture = PROFILE_CAPTURE.read_text(encoding="utf-8")
        profile_resize = PROFILE_RESIZE.read_text(encoding="utf-8")
        unreal_app = UNREAL_SLATE_APPLICATION.read_text(encoding="utf-8")
        unreal_window = UNREAL_WINDOW.read_text(encoding="utf-8")

        self.assertNotIn("Duration::from_millis(80)", resize)
        self.assertNotIn("pending_resize_reflow_deadline", lifecycle)
        self.assertIn("redraw.requires_frame_update()", redraw)
        self.assertIn("into_interactive_frame_update()", resize)
        self.assertIn("[int]$AutoResizeStepCount = 24", capture)
        self.assertIn("[int]$AutoResizeDelayMs = 40", capture)
        self.assertIn("@($steps) + [pscustomobject]", profile_resize)

        on_size_changed = function_body(
            unreal_app,
            "bool FSlateApplication::OnSizeChanged",
            "void FSlateApplication::OnOSPaint",
        )
        cache_size = on_size_changed.index("Window->SetCachedSize")
        resize_renderer = on_size_changed.index("Renderer->RequestResize")
        draw = on_size_changed.index("PrivateDrawWindows")
        self.assertLess(cache_size, resize_renderer)
        self.assertLess(resize_renderer, draw)

        set_cached_size = function_body(
            unreal_window,
            "void SWindow::SetCachedSize",
            "bool SWindow::IsMorphing",
        )
        self.assertIn("InvalidateRootChildOrder();", set_cached_size)

        suite = pressure_suite(10_000, 10_000, 10_000, 64, 64, 8)
        self.assertEqual(
            suite["schema"],
            "zircon.editor.window_resize_reflow_pressure_suite.v2",
        )
        binding = suite["source_binding"]
        self.assertEqual(len(binding["git_revision"]), 40)
        self.assertEqual(
            {entry["path"] for entry in binding["files"]},
            {
                str(path.relative_to(ROOT)).replace("\\", "/")
                for path in (
                    RESIZE_EVENTS,
                    EVENT_LOOP_REDRAW,
                    SHELL_METRICS,
                    UNREAL_SLATE_APPLICATION,
                    UNREAL_WINDOW,
                )
            },
        )
        self.assertTrue(
            all(len(entry["sha256"]) == 64 for entry in binding["files"])
        )


if __name__ == "__main__":
    unittest.main()
