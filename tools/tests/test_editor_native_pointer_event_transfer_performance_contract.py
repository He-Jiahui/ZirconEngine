import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
POINTER_EVENTS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/"
    "event_loop/events/pointer.rs"
)


def rust_function(source: str, signature: str, next_signature: str) -> str:
    return source.split(signature, 1)[1].split(next_signature, 1)[0]


class NativePointerEventTransferPerformanceContractTests(unittest.TestCase):
    def test_button_prefetches_copy_fields_and_moves_the_platform_event(self):
        source = POINTER_EVENTS.read_text(encoding="utf-8")
        body = rust_function(
            source,
            "pub(super) fn handle_pointer_button(",
            "pub(super) fn handle_mouse_wheel(",
        )

        self.assertIn("let button = pointer.event.button;", body)
        self.assertIn("let modifiers = pointer.metadata.modifiers;", body)
        self.assertIn("invoke_workbench_pointer_input(pointer, None)", body)
        self.assertNotIn("pointer.clone()", body)

    def test_scroll_prefetches_delta_and_moves_the_platform_event(self):
        source = POINTER_EVENTS.read_text(encoding="utf-8")
        body = source.split("pub(super) fn handle_mouse_wheel(", 1)[1]

        self.assertIn("let scroll_delta = pointer.event.scroll_delta;", body)
        self.assertIn("invoke_workbench_pointer_input(pointer, None)", body)
        self.assertIn("dispatch_native_pointer_scroll", body)
        self.assertNotIn("pointer.clone()", body)


if __name__ == "__main__":
    unittest.main()
