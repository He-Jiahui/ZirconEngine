from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMMAND = ROOT / "zircon_editor/src/ui/workbench/layout/layout_command.rs"
CORE_COMMAND = ROOT / "zircon_editor/src/core/editor_event/workbench/layout_command.rs"
CONVERSION = ROOT / "zircon_editor/src/ui/workbench/event/core_event_conversion.rs"
MANAGER = ROOT / "zircon_editor/src/ui/workbench/layout/manager/apply.rs"
HOST = ROOT / "zircon_editor/src/ui/host/layout_commands.rs"
DRAWER_RESIZE = ROOT / "zircon_editor/src/ui/retained_host/drawer_resize.rs"


class EditorLayoutGeometryCommandPerformanceContractTests(unittest.TestCase):
    def test_geometry_only_classification_is_owned_by_layout_command(self) -> None:
        source = COMMAND.read_text(encoding="utf-8")
        start = source.index("impl LayoutCommand")
        method = source[start:]

        self.assertIn("pub const fn is_geometry_only(&self) -> bool", method)
        self.assertIn("Self::ResizeSplit", method)
        self.assertIn("Self::SetDrawerExtent", method)
        self.assertIn("Self::SetDrawerRegionExtent", method)
        self.assertIn(
            "SetDrawerRegionExtent", CORE_COMMAND.read_text(encoding="utf-8")
        )
        conversion = CONVERSION.read_text(encoding="utf-8")
        self.assertEqual(conversion.count("LayoutCommand::SetDrawerRegionExtent"), 4)

    def test_manager_skips_global_drawer_repair_for_geometry_only_commands(self) -> None:
        source = MANAGER.read_text(encoding="utf-8")
        apply_start = source.index("    pub fn apply(")
        helper_start = source.index("\nfn append_instance_to_floating_workspace", apply_start)
        apply = source[apply_start:helper_start]

        self.assertIn("let geometry_only = cmd.is_geometry_only();", apply)
        self.assertIn("if matches!(&result, Ok(diff) if diff.changed) && !geometry_only", apply)
        self.assertIn("set_drawer_extents_atomically", apply)
        helper_start = source.index("fn set_drawer_extents_atomically")
        helper_end = source.index("\nfn drawer_region_slots", helper_start)
        helper = source[helper_start:helper_end]
        self.assertIn(".get_mut(slot)", helper)
        self.assertIn("slots.iter().copied().find", "".join(helper.split()))
        self.assertLess(helper.index(".find("), helper.index("for slot in slots"))

    def test_host_rebuilds_session_metadata_only_for_changed_structure(self) -> None:
        source = HOST.read_text(encoding="utf-8")
        start = source.index("    fn apply_layout_command_inner")
        end = source.index("\n    pub(super) fn open_view", start)
        function = source[start:end]

        self.assertIn("let geometry_only = cmd.is_geometry_only();", function)
        self.assertIn("if diff.changed && !geometry_only", function)
        generic = function[function.index("let focused_view"):]
        self.assertNotIn("sync_legacy_drawers_from_active_activity_window", generic)

    def test_drawer_region_resize_dispatches_one_atomic_layout_event(self) -> None:
        source = DRAWER_RESIZE.read_text(encoding="utf-8")
        start = source.index("pub(crate) fn dispatch_resize_to_group")
        end = source.index("\nfn group_slot", start)
        dispatch = source[start:end]

        self.assertIn("LayoutCommand::SetDrawerRegionExtent", dispatch)
        self.assertEqual(dispatch.count("dispatch_layout_command("), 1)
        self.assertNotIn("for slot in", dispatch)


if __name__ == "__main__":
    unittest.main()
