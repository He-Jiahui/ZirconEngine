from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class RuntimeHostIntentOutboxTransactionPerformanceContract(unittest.TestCase):
    def test_frame_reset_advances_host_request_watermarks_without_clearing_outbox(self) -> None:
        state = source("zircon_runtime/src/input/runtime/input_state.rs")
        manager = source("zircon_runtime/src/input/runtime/default_input_manager.rs")

        for owner in ["cursor_host", "ime_host", "gamepad_rumble"]:
            self.assertIn(f"{owner}_requests_frame_start", state)
            self.assertIn(
                f"state.{owner}_requests_frame_start = state.{owner}_requests.len();",
                manager,
            )
            self.assertNotIn(f"state.{owner}_requests.clear();", manager)

    def test_frame_snapshot_reads_only_rows_after_each_watermark(self) -> None:
        manager = source("zircon_runtime/src/input/runtime/default_input_manager.rs")
        normalized = "".join(manager.split())

        for owner in ["cursor_host", "ime_host", "gamepad_rumble"]:
            self.assertIn(
                f"state.{owner}_requests[state.{owner}_requests_frame_start..].to_vec()",
                normalized,
            )

    def test_drain_resets_watermarks_after_taking_pending_rows(self) -> None:
        manager = source("zircon_runtime/src/input/runtime/default_input_manager.rs")

        for owner in ["cursor_host", "ime_host", "gamepad_rumble"]:
            self.assertIn(f"state.{owner}_requests_frame_start = 0;", manager)
            self.assertIn(f"std::mem::take(&mut state.{owner}_requests)", manager)

    def test_runtime_ui_ime_outboxes_join_dynamic_session_collection(self) -> None:
        runtime_ui = source(
            "zircon_runtime/src/dynamic_api/session/runtime_ui/host_request_drain.rs"
        )
        session = source("zircon_runtime/src/dynamic_api/session/state.rs")
        normalized_session = "".join(session.split())

        self.assertIn("fn drain_ime_host_requests_into", runtime_ui)
        self.assertIn("runtime_surface.input.drain_ime_host_requests()", runtime_ui)
        self.assertIn("self.runtime_ui.drain_ime_host_requests_into", normalized_session)

    def test_host_request_page_serializes_a_borrowed_prefix(self) -> None:
        frame = source("zircon_runtime/src/dynamic_api/frame.rs")
        session = source("zircon_runtime/src/dynamic_api/session/state.rs")

        self.assertIn("fn encode_host_request_page", frame)
        self.assertIn("requests: &'a [ZrRuntimeHostRequestV1]", frame)
        self.assertIn("encode_host_request_page(", session)
        self.assertNotIn("pending.requests[..count].to_vec()", session)

    def test_host_request_publication_keeps_spatial_scale_counters(self) -> None:
        session = source("zircon_runtime/src/dynamic_api/session/state.rs")

        for counter in [
            "host_request.runtime_ui_ime_rows",
            "host_request.core_ime_rows",
            "host_request.rumble_rows",
            "host_request.cursor_rows",
            "host_request.batch_rows",
            "host_request.pending_rows",
            "host_request.page_encode_attempt",
            "host_request.page_rows",
            "host_request.encoded_bytes",
        ]:
            self.assertIn(counter, session)

    def test_app_collects_finished_rumble_effects_once_per_nonempty_host_batch(self) -> None:
        drain = source("zircon_app/src/entry/runtime_entry_app/host_requests/drain.rs")
        routing = source("zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs")
        rumble = source("zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs")
        normalized_drain = "".join(drain.split())
        cleanup = "clear_finished_rumble_effects("

        self.assertEqual(normalized_drain.count(cleanup), 1)
        self.assertIn("self.gamepad_rumble_effects.as_mut()", normalized_drain)
        self.assertLess(
            normalized_drain.index("if!requests.is_empty()"),
            normalized_drain.index(cleanup),
        )
        self.assertLess(
            normalized_drain.index(cleanup),
            normalized_drain.index("forrequestinrequests"),
        )
        self.assertNotIn("clear_finished_rumble_effects", routing)

        leaf_start = rumble.index("fn apply_runtime_gamepad_rumble_request")
        leaf_end = rumble.index("#[cfg(not(feature = \"gamepad-gilrs\"))]", leaf_start)
        self.assertNotIn(cleanup, rumble[leaf_start:leaf_end])


if __name__ == "__main__":
    unittest.main()
