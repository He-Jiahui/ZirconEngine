from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
EVENT_LOOP = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs"
)
REDRAW = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs"
)
PRESENT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs"
)
LIFECYCLE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs"
)


def function_body(source: str, signature: str, next_signature: str) -> str:
    return source.split(signature, 1)[1].split(next_signature, 1)[0]


class EditorSurfacePresentRetryPerformanceContract(unittest.TestCase):
    def test_retryable_present_defers_instead_of_spinning_native_redraw(self) -> None:
        source = PRESENT.read_text(encoding="utf-8")
        branch = source.split(
            "Err(HostPresenterError::RetryableSurfacePresent) =>", 1
        )[1].split("Err(error) =>", 1)[0]

        self.assertIn("defer_surface_present_retry", branch)
        self.assertNotIn("queue_redraw", branch)
        self.assertNotIn("request_redraw", branch)

    def test_retry_state_is_separate_bounded_and_part_of_wait_policy(self) -> None:
        event_loop = EVENT_LOOP.read_text(encoding="utf-8")
        redraw = REDRAW.read_text(encoding="utf-8")
        lifecycle = LIFECYCLE.read_text(encoding="utf-8")

        self.assertIn("pending_surface_present_retry: HostRedrawRequest", event_loop)
        self.assertIn("SURFACE_PRESENT_RETRY_BASE_DELAY", event_loop)
        self.assertIn("SURFACE_PRESENT_RETRY_MAX_DELAY", event_loop)
        self.assertIn("attempt.min(5)", redraw)
        self.assertIn(".min(super::SURFACE_PRESENT_RETRY_MAX_DELAY)", redraw)
        self.assertIn("self.pending_surface_present_retry_deadline", lifecycle)
        self.assertIn("ControlFlow::WaitUntil(deadline)", lifecycle)

    def test_real_redraw_merges_and_consumes_deferred_retry(self) -> None:
        source = REDRAW.read_text(encoding="utf-8")
        body = function_body(
            source,
            "fn take_redraw_for_present",
            "fn take_surface_present_retry",
        )

        queued = body.index("self.take_pending_redraw()")
        retry = body.index("self.take_surface_present_retry()")
        self.assertLess(queued, retry)
        self.assertIn("queued.merge", body)


if __name__ == "__main__":
    unittest.main()
