from pathlib import Path
import unittest

from tools.editor_idle_pointer_move_mailbox_pressure import run


ROOT = Path(__file__).resolve().parents[2]


class EditorIdlePointerMoveMailboxPerformanceContract(unittest.TestCase):
    def test_event_loop_owns_one_bounded_latest_value_mailbox(self):
        event_loop = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs"
        ).read_text()
        mailbox = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/pointer_move_mailbox.rs"
        ).read_text()

        self.assertIn("pending_idle_pointer_move: UiIdlePointerMoveMailbox", event_loop)
        self.assertIn("pending_idle_pointer_move: None", mailbox)
        self.assertIn("self.pending_idle_pointer_move.replace(next)", mailbox)
        self.assertNotIn("Vec<", mailbox.split("#[cfg(test)]", 1)[0])
        self.assertNotIn("VecDeque<", mailbox.split("#[cfg(test)]", 1)[0])

    def test_non_move_window_events_flush_the_pending_move_first(self):
        events = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs"
        ).read_text()

        flush = events.index("self.flush_pending_idle_pointer_move();")
        translate = events.index("event_uses_platform_input(&event)")
        dispatch = events.index("match event")
        self.assertLess(flush, translate)
        self.assertLess(flush, dispatch)
        self.assertIn("self.try_defer_idle_pointer_move(&event)", events)

    def test_wait_and_proxy_boundaries_flush_the_latest_move(self):
        event_loop = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs"
        ).read_text()
        lifecycle = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs"
        ).read_text()

        self.assertIn("self.flush_pending_idle_pointer_move();", lifecycle)
        proxy = event_loop.split("fn proxy_wake_up", 1)[1].split("fn about_to_wait", 1)[0]
        self.assertIn("self.flush_pending_idle_pointer_move();", proxy)

    def test_capture_sensitive_moves_bypass_coalescing(self):
        mailbox = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/pointer_move_mailbox.rs"
        ).read_text()
        ui_context = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/globals/ui_context.rs"
        ).read_text()

        self.assertIn("self.pressed_mouse_button_count != 0", mailbox)
        self.assertIn("pointer_move_requires_immediate_dispatch", mailbox)
        self.assertIn("state.resize_state.resize_active", ui_context)
        self.assertIn("!state.drag_state.drag_tab_id.is_empty()", ui_context)
        self.assertIn("PointerSource::Mouse", mailbox)
        self.assertIn("primary: true", mailbox)
        self.assertIn("pending_device_id() != Some(*device_id)", mailbox)

    def test_superseded_sequences_receive_an_explicit_terminal_outcome(self):
        mailbox = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/pointer_move_mailbox.rs"
        ).read_text()
        outcome = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input_outcome.rs"
        ).read_text()

        self.assertIn("UiCoalescedInputRange", mailbox)
        self.assertIn("record_idle_pointer_move_batch", mailbox)
        self.assertIn("Coalesced", outcome)
        self.assertIn("ui.input.outcome.coalesced_first_sequence", outcome)
        self.assertIn("ui.input.outcome.coalesced_last_sequence", outcome)
        self.assertIn("ui.input.outcome.coalesced_count", outcome)
        self.assertIn("ui.idle_hover.received_move_count", outcome)
        self.assertIn("ui.idle_hover.coalesced_move_count", outcome)
        self.assertIn("ui.idle_hover.dispatched_move_count", outcome)

    def test_capture_counters_are_batched_once_at_flush(self):
        mailbox = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/pointer_move_mailbox.rs"
        ).read_text()
        outcome = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input_outcome.rs"
        ).read_text()

        defer_path = mailbox.split("pub(super) fn try_defer_idle_pointer_move", 1)[1].split(
            "pub(super) fn flush_pending_idle_pointer_move", 1
        )[0]
        self.assertNotIn("profile_counter!", defer_path)
        self.assertNotIn("record_counter_batch", defer_path)
        self.assertIn("record_counter_batch", outcome)
        self.assertIn("received_count", outcome)
        self.assertIn("coalesced.count", outcome)

    def test_deferred_moves_translate_only_the_latest_metadata_at_flush(self):
        mailbox = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/pointer_move_mailbox.rs"
        ).read_text()

        defer_path = mailbox.split("pub(super) fn try_defer_idle_pointer_move", 1)[1].split(
            "pub(super) fn flush_pending_idle_pointer_move", 1
        )[0]
        flush_path = mailbox.split(
            "pub(super) fn flush_pending_idle_pointer_move", 1
        )[1].split("#[cfg(test)]", 1)[0]

        self.assertIn("self.reserve_input_metadata()", defer_path)
        self.assertNotIn("translate_platform_input_event", defer_path)
        self.assertIn("translate_reserved_pointer_move_event", flush_path)

    def test_reserved_metadata_defers_window_id_allocation(self):
        input_source = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input.rs"
        ).read_text()
        metadata_source = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/window/metadata.rs"
        ).read_text()
        metadata_contract = (
            ROOT
            / "zircon_runtime_interface/src/ui/dispatch/input/metadata.rs"
        ).read_text()

        reserve = input_source.split("fn reserve_input_metadata", 1)[1].split("}", 1)[0]
        self.assertIn("native_input_metadata_without_window_id", reserve)
        self.assertNotIn("UiWindowId::new", reserve)
        self.assertIn("attach_native_window_id", metadata_source)
        self.assertIn("pub struct $name(pub String);", metadata_contract)
        self.assertIn("define_string_id!(UiWindowId);", metadata_contract)

    def test_pressure_model_preserves_capture_moves_and_bounds_idle_dispatches(self):
        result = run(
            idle_pointer_move_count=65_536,
            idle_event_batch_count=256,
            capture_pointer_move_count=4_096,
        )

        self.assertEqual(result["mailbox_path"]["idle_dispatched_moves"], 256)
        self.assertEqual(result["mailbox_path"]["idle_coalesced_moves"], 65_280)
        self.assertEqual(result["mailbox_path"]["capture_dispatched_moves"], 4_096)
        self.assertEqual(result["mailbox_path"]["total_spatial_queries"], 4_352)
        self.assertEqual(result["single_hit_path"]["total_spatial_queries"], 69_632)
        self.assertEqual(
            result["delta"]["single_hit_to_mailbox_spatial_query_ratio"], 16.0
        )
        self.assertEqual(
            result["profiling_capture_path"]["old_per_event_recorder_lock_count"],
            196_352,
        )
        self.assertEqual(
            result["profiling_capture_path"]["batched_recorder_lock_count"], 256
        )
        self.assertEqual(
            result["profiling_capture_path"]["batched_counter_sample_count"], 1_536
        )
        self.assertEqual(
            result["input_metadata_path"]["old_idle_window_id_allocation_count"],
            65_536,
        )
        self.assertEqual(
            result["input_metadata_path"]["mailbox_idle_window_id_allocation_count"],
            256,
        )
        self.assertEqual(
            result["input_metadata_path"]["old_idle_platform_translation_count"],
            65_536,
        )
        self.assertEqual(
            result["input_metadata_path"]["mailbox_idle_platform_translation_count"],
            256,
        )
        self.assertEqual(result["delta"]["idle_window_id_allocation_ratio"], 256.0)
        self.assertEqual(result["delta"]["idle_platform_translation_ratio"], 256.0)
        self.assertEqual(result["delta"]["capture_recorder_lock_ratio"], 767.0)
        self.assertFalse(result["scope"]["cpu_measured"])


if __name__ == "__main__":
    unittest.main()
