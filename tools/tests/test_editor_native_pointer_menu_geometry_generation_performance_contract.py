from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOST = ROOT / "zircon_editor/src/ui/retained_host/host_contract"
MENU_GEOMETRY_ROOT = HOST / "native_pointer/menu_geometry.rs"
MENU_GEOMETRY = HOST / "native_pointer/menu_geometry"


class EditorNativePointerMenuGeometryGenerationPerformanceContractTests(
    unittest.TestCase
):
    def test_menu_geometry_borrows_every_model_row(self) -> None:
        offenders = []
        for source_path in [MENU_GEOMETRY_ROOT, *MENU_GEOMETRY.rglob("*.rs")]:
            source = source_path.read_text(encoding="utf-8")
            if "row_data(" in source:
                offenders.append(str(source_path.relative_to(ROOT)))

        self.assertEqual([], offenders)

    def test_reset_state_compatibility_entries_are_removed(self) -> None:
        sources = "\n".join(
            source_path.read_text(encoding="utf-8")
            for source_path in [MENU_GEOMETRY_ROOT, *MENU_GEOMETRY.rglob("*.rs")]
        )

        for function_name in (
            "menu_handles_point",
            "menu_popup_handles_point",
            "menu_damage_frame",
        ):
            self.assertIsNone(
                re.search(rf"fn {function_name}\(", sources),
                msg=f"legacy reset-state entry remains: {function_name}",
            )

    def test_pointer_event_callers_use_explicit_menu_state(self) -> None:
        callers = (
            HOST / "native_pointer/button_dispatch/menu_press.rs",
            HOST / "native_pointer/move_dispatch/menu.rs",
            HOST / "native_pointer/scroll_dispatch/menu.rs",
        )

        for caller in callers:
            source = caller.read_text(encoding="utf-8")
            self.assertIn("menu_handles_point_with_state", source)
            self.assertIn("menu_popup_handles_point_with_state", source)
            self.assertIn("menu_damage_frame_with_state", source)
            self.assertIn("menu_state", source)


if __name__ == "__main__":
    unittest.main()
