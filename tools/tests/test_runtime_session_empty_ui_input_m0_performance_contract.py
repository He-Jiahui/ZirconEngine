from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EVENTS = ROOT / "zircon_runtime/src/dynamic_api/session/events.rs"
EVENTS_DIR = ROOT / "zircon_runtime/src/dynamic_api/session/events"


class RuntimeSessionEmptyUiInputM0PerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = EVENTS.read_text(encoding="utf-8")
        cls.all_source = "\n".join(
            [cls.source]
            + [
                path.read_text(encoding="utf-8")
                for path in sorted(EVENTS_DIR.glob("*.rs"))
            ]
        )

    def function_body(self, signature: str, next_signature: str) -> str:
        start = self.source.index(signature)
        end = self.source.index(next_signature, start)
        return self.source[start:end]

    def test_non_pointer_ui_events_are_built_lazily_after_empty_surface_guard(self) -> None:
        body = self.function_body(
            "fn dispatch_runtime_ui_event(",
            "fn event_payload(",
        )

        self.assertIn("impl FnOnce(UiInputEventMetadata) -> UiInputEvent", body)
        guard = body.index("if self.runtime_ui.is_empty()")
        build = body.index("let event = event(")
        self.assertLess(guard, build)
        self.assertGreaterEqual(
            self.all_source.count("dispatch_runtime_ui_event(|metadata|"), 9
        )

    def test_pointer_ui_dispatch_returns_before_building_metadata_when_empty(self) -> None:
        body = self.function_body(
            "fn dispatch_runtime_ui_pointer(",
            "fn dispatch_runtime_ui_event(",
        )

        guard = body.index("if self.runtime_ui.is_empty()")
        dispatch = body.index(".dispatch_pointer(")
        self.assertLess(guard, dispatch)

    def test_callers_no_longer_materialize_ui_metadata_eagerly(self) -> None:
        self.assertNotIn("self.runtime_ui_input_metadata()", self.all_source)
        self.assertNotIn("fn runtime_ui_input_metadata(", self.all_source)


if __name__ == "__main__":
    unittest.main()
