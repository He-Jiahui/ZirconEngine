import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)


class EditorPopupControlIndexContractTests(unittest.TestCase):
    def test_popup_state_uses_bridge_control_index_instead_of_full_tree_scans(self):
        control_state = (WORKBENCH_BRIDGE / "control_state.rs").read_text(
            encoding="utf-8"
        )
        popup_state = (WORKBENCH_BRIDGE / "popup_state.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "pub(super) fn control_node_ids_with_descendants", control_state
        )
        self.assertIn("self.control_node_ids_with_descendants", popup_state)
        self.assertIn("self.control_string_array", popup_state)
        self.assertNotIn("surface.tree.nodes.values()", popup_state)
        self.assertNotIn("fn control_string_array(", popup_state)

        context_menu = (WORKBENCH_BRIDGE / "context_menu.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("self.control_node_ids_with_descendants", context_menu)
        self.assertIn(".control_float(", context_menu)
        self.assertNotIn("surface.tree.nodes.values()", context_menu)
        self.assertNotIn("fn control_float_property(", context_menu)

    def test_static_menu_state_uses_the_same_indexed_property_authority(self):
        for filename in ("run_mode_menu.rs", "layout_menu.rs"):
            source = (WORKBENCH_BRIDGE / filename).read_text(encoding="utf-8")
            self.assertIn("self.control_string_array", source, filename)
            self.assertNotIn(
                "use super::popup_state::control_string_array", source, filename
            )

    def test_overlay_state_reads_use_the_retained_control_index(self):
        for filename in ("command_palette.rs", "notifications.rs", "icon_tooltip.rs"):
            source = (WORKBENCH_BRIDGE / filename).read_text(encoding="utf-8")
            self.assertNotIn("tree.nodes.values()", source, filename)

        command_palette = (WORKBENCH_BRIDGE / "command_palette.rs").read_text(
            encoding="utf-8"
        )
        notifications = (WORKBENCH_BRIDGE / "notifications.rs").read_text(
            encoding="utf-8"
        )
        icon_tooltip = (WORKBENCH_BRIDGE / "icon_tooltip.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn(".control_integer(", command_palette)
        self.assertIn(".control_integer(", notifications)
        self.assertIn("self.control_node_id", icon_tooltip)
        self.assertIn(".set_popup_control_anchor", icon_tooltip)
        self.assertNotIn("popup_anchor_x", icon_tooltip)
        self.assertNotIn("popup_anchor_y", icon_tooltip)
        self.assertNotIn("popup_anchor_width", icon_tooltip)
        self.assertNotIn("popup_anchor_height", icon_tooltip)

    def test_runtime_is_the_only_tooltip_flip_and_clamp_authority(self):
        icon_tooltip = (WORKBENCH_BRIDGE / "icon_tooltip.rs").read_text(
            encoding="utf-8"
        )
        runtime_popup_position = (
            REPO_ROOT / "zircon_runtime/src/ui/surface/render/popup_position.rs"
        ).read_text(encoding="utf-8")
        runtime_feedback = (
            REPO_ROOT / "zircon_runtime/src/ui/surface/render/feedback.rs"
        ).read_text(encoding="utf-8")
        runtime_extract = (
            REPO_ROOT / "zircon_runtime/src/ui/surface/render/extract.rs"
        ).read_text(encoding="utf-8")
        runtime_command_palette = (
            REPO_ROOT / "zircon_runtime/src/ui/surface/render/command_palette.rs"
        ).read_text(encoding="utf-8")
        runtime_notification_center = (
            REPO_ROOT / "zircon_runtime/src/ui/surface/render/notification_center.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("fn tooltip_placement(", icon_tooltip)
        self.assertNotIn("TOOLTIP_HEIGHT", icon_tooltip)
        self.assertNotIn("TOOLTIP_GAP", icon_tooltip)
        self.assertIn("placement.flipped()", runtime_popup_position)
        self.assertIn("clamp_to_bounds(frame, bounds)", runtime_popup_position)
        self.assertIn("fn positioned_feedback_frame(", runtime_feedback)
        self.assertIn("resolve_anchored_popup_geometry(", runtime_feedback)
        self.assertIn("resolve_anchored_popup_geometry(", runtime_command_palette)
        self.assertIn("resolve_anchored_popup_geometry(", runtime_notification_center)
        self.assertGreaterEqual(runtime_extract.count("popup_anchor_frame,"), 3)


if __name__ == "__main__":
    unittest.main()
