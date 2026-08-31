import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui.rs"


def function_source(source: str, start_anchor: str, end_anchor: str) -> str:
    start = source.index(start_anchor)
    end = source.index(end_anchor, start)
    return source[start:end]


class RuntimeSessionUiSurfaceLastConsumerM0PerformanceContract(unittest.TestCase):
    def test_generic_reverse_route_moves_the_event_into_surface_zero(self):
        source = SOURCE.read_text(encoding="utf-8")
        dispatch = function_source(
            source,
            "    pub(super) fn dispatch_input(",
            "\n    pub(super) fn dispatch_pointer(",
        )

        self.assertIn("let mut event = Some(event);", dispatch)
        self.assertIn("for surface_index in (0..self.surfaces.len()).rev()", dispatch)
        self.assertIn(
            "input_event_for_surface(&mut event, surface_index == 0)", dispatch
        )
        self.assertNotIn("event.clone()", dispatch)

    def test_noncapture_pointer_route_moves_the_event_into_surface_zero(self):
        source = SOURCE.read_text(encoding="utf-8")
        dispatch = function_source(
            source,
            "    fn dispatch_pointer_input(",
            "\n    fn dispatch_pointer_to_surface(",
        )

        self.assertIn("let mut event = Some(event);", dispatch)
        self.assertIn(
            "input_event_for_surface(&mut event, surface_index == 0)", dispatch
        )
        self.assertNotIn("event.clone()", dispatch)

    def test_shared_fanout_helper_moves_only_the_last_surface_event(self):
        source = SOURCE.read_text(encoding="utf-8")
        helper = function_source(
            source,
            "fn input_event_for_surface(",
            "\nfn split_global_node_id(",
        )

        last_surface = helper.index("if last_surface")
        final_move = helper.index("event.take()")
        earlier_clone = helper.index("event.as_ref().cloned()")
        self.assertLess(last_surface, final_move)
        self.assertLess(final_move, earlier_clone)
        self.assertEqual(helper.count(".cloned()"), 1)

    def test_pointer_capture_keeps_direct_single_surface_dispatch(self):
        source = SOURCE.read_text(encoding="utf-8")
        dispatch = function_source(
            source,
            "    fn dispatch_pointer_input(",
            "\n    fn dispatch_pointer_to_surface(",
        )

        capture = dispatch.index("capture_surface_for_event(")
        last_consumer = dispatch.index("let mut event = Some(event);")
        self.assertLess(capture, last_consumer)
        self.assertIn("kind,\n                event,", dispatch[capture:last_consumer])


if __name__ == "__main__":
    unittest.main()
