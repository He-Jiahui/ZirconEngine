import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MOUSE_MOTION_PATH = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "surface"
    / "input"
    / "mouse_motion.rs"
)


def rust_function(source: str, name: str) -> str:
    match = re.search(rf"fn\s+{re.escape(name)}\s*\(", source)
    if match is None:
        raise AssertionError(f"missing Rust function: {name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing Rust function body: {name}")
    depth = 1
    index = opening + 1
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function: {name}")
    return source[opening + 1 : index - 1]


class RuntimeUiMouseMotionFastPathPerformanceContractTests(unittest.TestCase):
    def test_unrouted_motion_does_not_materialize_a_surface_route_trace(self):
        source = MOUSE_MOTION_PATH.read_text(encoding="utf-8")
        dispatch = rust_function(source, "dispatch_mouse_motion_input")

        self.assertIn("UiInputRoutePolicy::Unrouted", dispatch)
        self.assertNotIn("annotate_route_policy", dispatch)
        self.assertNotIn("annotate_result_route_steps", dispatch)

    def test_motion_event_moves_once_into_the_public_result(self):
        source = MOUSE_MOTION_PATH.read_text(encoding="utf-8")
        dispatch = rust_function(source, "dispatch_mouse_motion_input")

        self.assertIn("UiInputEvent::MouseMotion(motion)", dispatch)
        self.assertNotIn("event.clone()", dispatch)


if __name__ == "__main__":
    unittest.main()
