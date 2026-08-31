import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
POPUP_ROWS = REPO_ROOT / "zircon_runtime/src/ui/surface/render/popup_rows.rs"
SURFACE = REPO_ROOT / "zircon_runtime/src/ui/surface/surface.rs"


class RuntimePopupShadowCoverageContractTests(unittest.TestCase):
    def test_popup_background_publishes_two_coverage_layers_before_authoritative_fill(self):
        source = POPUP_ROWS.read_text(encoding="utf-8")

        self.assertIn("POPUP_SHADOW_NEAR_OPACITY", source)
        self.assertIn("POPUP_SHADOW_FAR_OPACITY", source)
        self.assertIn("popup_shadow_frame(frame, 2.0, 2.0)", source)
        self.assertIn("popup_shadow_frame(frame, 1.0, 1.0)", source)
        self.assertEqual(source.count("z_index.saturating_sub("), 2)
        self.assertLess(
            source.index("popup_shadow_frame(frame, 2.0, 2.0)"),
            source.index("        frame,\n        clip_frame,\n        z_index,"),
        )

    def test_shadow_layers_do_not_become_popup_hit_geometry(self):
        source = SURFACE.read_text(encoding="utf-8")

        self.assertIn("command.z_index == popup_base_z(arranged.z_index)", source)
        self.assertIn("command.style.painter_state == UiPainterResolvedState::Open", source)

    def test_popup_builders_reserve_shadow_commands_without_reallocating(self):
        for relative_path in (
            "zircon_runtime/src/ui/surface/render/popup_menu.rs",
            "zircon_runtime/src/ui/surface/render/popup_options.rs",
        ):
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertIn("saturating_mul(3).saturating_add(3)", source, relative_path)


if __name__ == "__main__":
    unittest.main()
