import unittest
from pathlib import Path


class RuntimePlatformWindowCapabilityOwnerStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.owner = (
            self.repo_root
            / "zircon_runtime/src/platform/capability/matrix/window.rs"
        )
        self.owner_dir = self.owner.with_suffix("")

    def test_window_capabilities_use_focused_folder_backed_owners(self) -> None:
        owner_source = self.owner.read_text(encoding="utf-8")
        production_lines = [
            line
            for line in owner_source.splitlines()
            if line.strip() and not line.lstrip().startswith("//")
        ]

        self.assertLessEqual(len(production_lines), 12)
        for declaration in (
            "mod backend;",
            "mod cursor;",
            "mod drag_drop;",
            "mod ime;",
            "mod lifecycle;",
        ):
            self.assertIn(declaration, owner_source)

        expected_children = {
            "backend.rs": ("fn window_backend",),
            "cursor.rs": (
                "fn cursor_boundary_backend",
                "fn cursor_options_backend",
                "fn pointer_position_backend",
                "fn raw_mouse_motion_backend",
            ),
            "drag_drop.rs": ("fn file_drag_drop_backend",),
            "ime.rs": ("fn ime_backend",),
            "lifecycle.rs": (
                "fn monitor_inventory_backend",
                "fn window_event_backend",
                "fn window_lifecycle_backend",
                "fn window_metrics_backend",
            ),
        }
        for child_name, anchors in expected_children.items():
            child = self.owner_dir / child_name
            self.assertTrue(child.is_file(), child)
            child_source = child.read_text(encoding="utf-8")
            self.assertIn(
                "pub(in crate::platform::capability::matrix)", child_source
            )
            for anchor in anchors:
                self.assertIn(anchor, child_source)

        self.assertNotIn("impl PlatformCapabilityMatrix", owner_source)
        self.assertNotIn("CapabilityStatus::", owner_source)

        report_source = (self.owner.parent / "build_report.rs").read_text(
            encoding="utf-8"
        )
        for call in (
            "self.window_backend(target, target_mode)",
            "self.monitor_inventory_backend(target, target_mode)",
            "self.window_event_backend(target, target_mode)",
            "self.window_lifecycle_backend(target, target_mode)",
            "self.window_metrics_backend(target, target_mode)",
            "self.ime_backend(target, target_mode)",
            "self.cursor_boundary_backend(target, target_mode)",
            "self.cursor_options_backend(target, target_mode)",
            "self.pointer_position_backend(target, target_mode)",
            "self.raw_mouse_motion_backend(target, target_mode)",
            "self.file_drag_drop_backend(target, target_mode)",
        ):
            self.assertIn(call, report_source)


if __name__ == "__main__":
    unittest.main()
