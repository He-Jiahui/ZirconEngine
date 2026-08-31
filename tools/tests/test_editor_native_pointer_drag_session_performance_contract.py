from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RETAINED_HOST = ROOT / "zircon_editor/src/ui/retained_host"
NATIVE_POINTER = RETAINED_HOST / "host_contract/native_pointer"


class EditorNativePointerDragSessionPerformanceContractTests(unittest.TestCase):
    def owner_sources(self) -> dict[Path, str]:
        sources: dict[Path, str] = {}
        for owner in ("drag_resize", "tab_drag_damage"):
            module = NATIVE_POINTER / f"{owner}.rs"
            sources[module] = module.read_text(encoding="utf-8")
            for path in (NATIVE_POINTER / owner).rglob("*.rs"):
                sources[path] = path.read_text(encoding="utf-8")
        return sources

    def test_pointer_move_updates_scalar_drag_state_without_full_clone(self) -> None:
        move = (
            NATIVE_POINTER / "drag_resize/tab_drag/lifecycle/move_event.rs"
        ).read_text(encoding="utf-8")
        active = (
            NATIVE_POINTER / "drag_resize/tab_drag/lifecycle/move_event/active.rs"
        ).read_text(encoding="utf-8")
        start = (
            NATIVE_POINTER / "drag_resize/tab_drag/lifecycle/move_event/start.rs"
        ).read_text(encoding="utf-8")
        context = (
            RETAINED_HOST / "host_contract/globals/ui_context.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("get_drag_state()", move + active + start)
        self.assertNotIn("HostDragStateData", move + active + start)
        self.assertIn("drag_pointer_snapshot", context)
        self.assertIn("set_drag_pointer_position", context)
        self.assertIn("activate_drag_at", context)

    def test_stable_drag_target_compares_typed_route_before_key_allocation(self) -> None:
        drag_drop = (
            RETAINED_HOST / "app/workspace_docking/drag_drop.rs"
        ).read_text(encoding="utf-8")
        group = (RETAINED_HOST / "tab_drag/group.rs").read_text(encoding="utf-8")
        sync = drag_drop.split("pub(super) fn sync_drag_target_group", 1)[1].split(
            "pub(super) fn dispatch_drag_drop_from_pointer", 1
        )[0]

        self.assertIn("host_shell_pointer_route_matches_group_key", sync)
        self.assertLess(
            sync.index("host_shell_pointer_route_matches_group_key"),
            sync.index("host_shell_pointer_route_group_key"),
        )
        self.assertNotIn("get_drag_state()", sync)
        self.assertIn("drag_target_group_matches", sync)
        self.assertIn("set_drag_target_group", sync)
        self.assertIn("fn host_shell_pointer_route_matches_group_key", group)

    def test_drag_payload_and_damage_use_borrowed_model_rows(self) -> None:
        cloning_sources = [
            str(path.relative_to(ROOT))
            for path, source in self.owner_sources().items()
            if "row_data(" in source
        ]

        self.assertEqual([], cloning_sources)

    def test_duplicate_resize_point_returns_idle_before_callback_and_redraw(self) -> None:
        resize_move = (
            NATIVE_POINTER / "drag_resize/resize_capture/move_event.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("update_resize_pointer_if_active", resize_move)
        unchanged = resize_move.index("Some(false)")
        callback = resize_move.index("invoke_host_resize_pointer_event")
        redraw = resize_move.rindex("resize_pointer_redraw(")
        self.assertLess(unchanged, callback)
        self.assertLess(unchanged, redraw)


if __name__ == "__main__":
    unittest.main()
