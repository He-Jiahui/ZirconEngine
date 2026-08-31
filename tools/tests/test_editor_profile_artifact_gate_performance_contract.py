from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
HOST_CONTRACT = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/host_contract"
)
PRESENT = HOST_CONTRACT / "window/event_loop/redraw/present.rs"
ENVIRONMENT = HOST_CONTRACT / "profiling_artifacts/environment.rs"
EXPORT = HOST_CONTRACT / "profiling_artifacts/export.rs"


class EditorProfileArtifactGatePerformanceContract(unittest.TestCase):
    def test_one_shot_state_short_circuits_capture_environment_read(self) -> None:
        source = PRESENT.read_text(encoding="utf-8")
        success = source.split("Ok(diagnostics) =>", 1)[1]
        success = success.split(
            "Err(HostPresenterError::RetryableSurfacePresent)", 1
        )[0]
        gate = success.split("presenter_backend.filter", 1)[1]
        gate = gate.split("})", 1)[0]

        already_requested = gate.index(
            "!event_loop_state.profile_artifact_capture_requested"
        )
        environment_read = gate.index("profile_capture_enabled()")

        self.assertLess(already_requested, environment_read)
        self.assertNotIn("should_queue_profile_artifacts", gate)

    def test_forced_softbuffer_export_has_no_unreachable_second_gate(self) -> None:
        environment = ENVIRONMENT.read_text(encoding="utf-8")
        export = EXPORT.read_text(encoding="utf-8")

        self.assertNotIn("is_forced_softbuffer_screenshot_run", environment)
        self.assertNotIn("is_forced_softbuffer_screenshot_run", export)


if __name__ == "__main__":
    unittest.main()
