import unittest
from pathlib import Path


class RuntimePlatformCapabilityBackendOwnerStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.owner = (
            self.repo_root / "zircon_runtime/src/platform/capability/backends.rs"
        )
        self.owner_dir = self.owner.with_suffix("")

    def test_platform_backends_use_domain_owned_declarations(self) -> None:
        owner_source = self.owner.read_text(encoding="utf-8")
        production_lines = [
            line
            for line in owner_source.splitlines()
            if line.strip() and not line.lstrip().startswith("//")
        ]
        self.assertLessEqual(len(production_lines), 42)

        expected_children = {
            "cursor.rs": (
                "pub enum CursorBoundaryBackend",
                "pub enum CursorOptionsBackend",
                "pub enum PointerPositionBackend",
                "pub enum RawMouseMotionBackend",
            ),
            "drag_drop.rs": ("pub enum FileDragDropBackend",),
            "event_loop.rs": ("pub enum EventLoopPolicy",),
            "gamepad.rs": (
                "pub enum GamepadBackend",
                "pub enum GamepadEventBackend",
                "pub enum GamepadRumbleBackend",
            ),
            "input.rs": (
                "pub enum KeyboardEventBackend",
                "pub enum MouseButtonBackend",
                "pub enum MouseWheelBackend",
                "pub enum TouchEventBackend",
                "pub enum GestureEventBackend",
                "pub enum InputBackend",
            ),
            "linux.rs": ("pub enum LinuxWindowProtocol",),
            "window.rs": (
                "pub enum WindowBackend",
                "pub enum MonitorBackend",
                "pub enum WindowEventBackend",
                "pub enum WindowLifecycleBackend",
                "pub enum WindowMetricsBackend",
                "pub enum ImeBackend",
            ),
        }
        for child_name, anchors in expected_children.items():
            module_name = child_name.removesuffix(".rs")
            self.assertIn(f"mod {module_name};", owner_source)
            child = self.owner_dir / child_name
            self.assertTrue(child.is_file(), child)
            child_source = child.read_text(encoding="utf-8")
            for anchor in anchors:
                self.assertIn(anchor, child_source)

        self.assertNotIn("pub enum", owner_source)
        self.assertEqual(owner_source.count("pub use "), len(expected_children))

        capability_facade = (self.owner.parent / "mod.rs").read_text(encoding="utf-8")
        self.assertIn("pub use backends::{", capability_facade)
        for symbol in (
            "WindowBackend",
            "MonitorBackend",
            "ImeBackend",
            "CursorBoundaryBackend",
            "KeyboardEventBackend",
            "InputBackend",
            "GamepadBackend",
            "FileDragDropBackend",
            "LinuxWindowProtocol",
            "EventLoopPolicy",
        ):
            self.assertIn(symbol, capability_facade)


if __name__ == "__main__":
    unittest.main()
