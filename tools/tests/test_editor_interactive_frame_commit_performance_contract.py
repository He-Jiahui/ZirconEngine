from pathlib import Path
import unittest

from tools.editor_interactive_frame_commit_pressure import run


ROOT = Path(__file__).resolve().parents[2]
EDITOR = ROOT / "zircon_editor/src/ui/retained_host"
HOST = EDITOR / "host_contract"


def function_source(source: str, start: str, end: str) -> str:
    return source.split(start, 1)[1].split(end, 1)[0]


class EditorInteractiveFrameCommitPerformanceContractTests(unittest.TestCase):
    def test_redraw_request_retains_interactive_frame_update_authority(self) -> None:
        request = (HOST / "redraw/request.rs").read_text(encoding="utf-8")
        constructors = (HOST / "redraw/request/constructors.rs").read_text(
            encoding="utf-8"
        )
        merge = (HOST / "redraw/request/merge.rs").read_text(encoding="utf-8")
        query = (HOST / "redraw/request/query.rs").read_text(encoding="utf-8")

        self.assertGreaterEqual(request.count("interactive_frame_update: bool"), 3)
        self.assertIn("fn into_interactive_frame_update", constructors)
        self.assertIn("interactive_frame_update: true", constructors)
        self.assertIn("interactive_frame_update || next_interactive_frame_update", merge)
        self.assertIn("fn prefers_interactive_frame_update", query)

    def test_pointer_and_resize_redraws_mark_interactive_commits(self) -> None:
        redraw = (HOST / "window/event_loop/redraw.rs").read_text(encoding="utf-8")
        resize = (HOST / "window/event_loop/events/resize.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("result.redraw().into_interactive_frame_update()", redraw)
        self.assertIn("redraw.prefers_interactive_frame_update()", redraw)
        self.assertIn("request_interactive_frame_update", redraw)
        queue_resize = function_source(
            resize,
            "fn queue_resize_frame",
            "pub(super) fn handle_window_moved",
        )
        self.assertIn("into_interactive_frame_update()", queue_resize)

    def test_interactive_frame_callback_has_a_dedicated_host_route(self) -> None:
        callbacks = (HOST / "globals/callbacks/host.rs").read_text(encoding="utf-8")
        context = (HOST / "globals/ui_context.rs").read_text(encoding="utf-8")
        window = (HOST / "window/redraw.rs").read_text(encoding="utf-8")
        wiring = (EDITOR / "app/callback_wiring/host_shell/runtime.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("interactive_frame_requested", callbacks)
        self.assertIn("on_interactive_frame_requested", context)
        self.assertIn("invoke_interactive_frame_requested", context)
        self.assertIn("fn request_interactive_frame_update", window)
        self.assertIn("host.borrow_mut().commit_interactive_frame_update()", wiring)

    def test_interactive_commit_excludes_maintenance_from_input_to_present(self) -> None:
        source = (EDITOR / "app/host_lifecycle/tick.rs").read_text(encoding="utf-8")
        interactive = function_source(
            source,
            "fn commit_interactive_frame_update",
            "fn commit_pending_frame_update",
        )

        self.assertIn("self.commit_pending_frame_update();", interactive)
        self.assertIn("self.ui.set_lifecycle_frame_update(Some(Instant::now()))", interactive)
        self.assertIn('"ui.interactive_frame.maintenance_deferred_count"', interactive)
        for maintenance_call in (
            "pump_editor_job_events",
            "pump_runtime_task_diagnostics",
            "refresh_project_session_heartbeat_if_due",
            "poll_editor_autosave",
            "poll_model_import",
            "pump_plugin_lifecycle_messages",
            "refresh_project_assets",
        ):
            self.assertNotIn(maintenance_call, interactive)

    def test_pressure_model_removes_maintenance_stages_from_click_critical_path(self) -> None:
        result = run(
            interaction_count=4_096,
            maintenance_stage_count=24,
            committed_frame_stage_count=3,
        )

        self.assertEqual(result["old_critical_path_stage_visits"], 110_592)
        self.assertEqual(result["new_critical_path_stage_visits"], 12_288)
        self.assertEqual(result["deferred_maintenance_stage_visits"], 98_304)
        self.assertEqual(result["critical_path_stage_reduction_ratio"], 9.0)

    def test_product_click_profile_requires_the_interactive_commit_counter(self) -> None:
        evidence = (ROOT / "tools/ui-profile-counter-evidence.ps1").read_text(
            encoding="utf-8"
        )
        capture = (ROOT / "tools/ui-profile-capture.ps1").read_text(encoding="utf-8")

        self.assertIn("Test-ZirconInteractiveFrameCommitCounterGate", evidence)
        self.assertIn("ui.interactive_frame.maintenance_deferred_count", evidence)
        self.assertIn("Test-InteractiveFrameCommitCounterGate", capture)
        self.assertIn("$interactiveFrameCommitOk", capture)


if __name__ == "__main__":
    unittest.main()
