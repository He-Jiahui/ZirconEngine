from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PREVIEW = ROOT / "zircon_editor/src/ui/layouts/views/preview_images.rs"
VIEWS_MOD = ROOT / "zircon_editor/src/ui/layouts/views/mod.rs"
VISUAL_ASSETS = ROOT / (
    "zircon_editor/src/ui/layouts/views/view_projection/visual_assets.rs"
)
ROOT_OVERLAY = ROOT / "zircon_editor/src/ui/retained_host/ui/root_template_overlay.rs"
WORKBENCH_PROJECTION = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs"
)
CHROME_PROJECTION = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "chrome_template_projection.rs"
)
VALUE_MEDIA = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection/value_media/mod.rs"
)
PANE_PROJECTION_MOD = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection/mod.rs"
)
PANE_PREVIEW_FORWARDER = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection/preview_images.rs"
)
TEMPLATE_IMAGE_COMMAND = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_images/command.rs"
)
VALUE_TEXT = (
    ROOT
    / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection/value_media/text.rs"
)


class EditorTextValueMediaProjectionPerformanceContractTests(unittest.TestCase):
    def test_projection_publishes_visual_locators_without_decoding_pixels(self) -> None:
        projection_sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (
                VISUAL_ASSETS,
                ROOT_OVERLAY,
                WORKBENCH_PROJECTION,
                CHROME_PROJECTION,
                VALUE_MEDIA,
            )
        )

        self.assertNotIn("load_preview_image", projection_sources)
        self.assertNotIn("preview_size", projection_sources)
        self.assertIn("!media_source.trim().is_empty()", projection_sources)
        self.assertIn("preview_image: Default::default()", projection_sources)

    def test_projection_preview_decoder_and_forwarder_are_hard_cut(self) -> None:
        views_mod = VIEWS_MOD.read_text(encoding="utf-8")
        pane_mod = PANE_PROJECTION_MOD.read_text(encoding="utf-8")

        self.assertFalse(PREVIEW.exists())
        self.assertFalse(PANE_PREVIEW_FORWARDER.exists())
        self.assertNotIn("mod preview_images;", views_mod)
        self.assertNotIn("mod preview_images;", pane_mod)

    def test_unknown_image_aspect_materializes_before_final_geometry(self) -> None:
        source = TEMPLATE_IMAGE_COMMAND.read_text(encoding="utf-8")
        command = source.split("fn push_template_image_command", 1)[1].split(
            "fn template_node_image_tint", 1
        )[0]

        self.assertIn("image_materialization_rect", command)
        self.assertIn(
            "image_rect_for_node(node, rect, image.width, image.height)", command
        )
        self.assertLess(
            command.index("image_materialization_rect"),
            command.index("template_image_pixels("),
        )
        self.assertLess(
            command.index("template_image_pixels("),
            command.index("image_rect_for_node(node, rect, image.width, image.height)"),
        )

    def test_collection_summary_reads_length_without_recursive_ui_value_conversion(self) -> None:
        source = VALUE_TEXT.read_text(encoding="utf-8")
        projection = source.split("pub(super) fn projected_value_text", 1)[1]
        summary = source.split("fn display_toml_value", 1)[1]

        self.assertIn(".map(display_toml_value)", projection)
        self.assertIn("toml::Value::Array(values) => format!(\"{} items\", values.len())", summary)
        self.assertIn("toml::Value::Table(values) => format!(\"{} entries\", values.len())", summary)
        self.assertIn("_ => UiValue::from_toml(value).display_text()", summary)


if __name__ == "__main__":
    unittest.main()
