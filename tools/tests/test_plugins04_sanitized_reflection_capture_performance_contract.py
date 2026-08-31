import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CAPTURE_SOURCE = (
    REPO_ROOT
    / "zircon_plugins"
    / "rendering"
    / "features"
    / "reflection_probes"
    / "runtime"
    / "src"
    / "capture"
    / "execute.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.find(signature)
    if start < 0:
        return ""
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        character = source[index]
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function body for {signature}")


class Plugins04SanitizedReflectionCapturePerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = CAPTURE_SOURCE.read_text(encoding="utf-8")
        cls.request_body = function_body(
            cls.source,
            "pub fn request_reflection_probe_capture(",
        )
        cls.poll_body = function_body(
            cls.source,
            "pub fn poll_reflection_probe_capture(",
        )
        cls.cancel_body = function_body(
            cls.source,
            "pub fn cancel_reflection_probe_capture(",
        )

    def test_capture_delegates_one_validated_snapshot_to_the_framework(self) -> None:
        self.assertIn("let render_request = request.render_request()?", self.request_body)
        self.assertIn(
            ".request_environment_capture(scene.clone(), render_request)",
            self.request_body,
        )
        self.assertEqual(self.request_body.count("scene.clone()"), 1)

    def test_plugin_boundary_does_not_materialize_faces_or_render_sidebands(self) -> None:
        self.assertNotIn("REFLECTION_PROBE_CAPTURE_FACE_VIEWS", self.request_body)
        self.assertNotIn("SceneViewportRenderPacket {", self.request_body)
        self.assertNotIn("RenderOverlayExtract", self.request_body)
        self.assertNotIn("virtual_geometry_debug", self.request_body)
        self.assertNotIn("Vec::with_capacity", self.request_body)

    def test_capture_lifecycle_remains_handle_based_and_nonblocking(self) -> None:
        self.assertIn(".poll_environment_capture(handle)", self.poll_body)
        self.assertIn(".cancel_environment_capture(handle)", self.cancel_body)
        self.assertNotIn("loop {", self.request_body)
        self.assertNotIn("loop {", self.poll_body)
        self.assertNotIn("loop {", self.cancel_body)

    def test_rust_contract_covers_the_nonblocking_framework_boundary(self) -> None:
        self.assertIn(
            "capture_execution_is_a_nonblocking_framework_boundary",
            self.source,
        )
        self.assertIn('assert!(source.contains("request_environment_capture"))', self.source)
        self.assertIn('assert!(source.contains("poll_environment_capture"))', self.source)
        self.assertIn('assert!(source.contains("cancel_environment_capture"))', self.source)
        self.assertIn('assert!(!source.contains("Vec::with_capacity"))', self.source)


if __name__ == "__main__":
    unittest.main()
