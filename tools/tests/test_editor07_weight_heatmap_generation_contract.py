from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
GENERATION = ROOT / "zircon_editor/src/ui/weight_heatmap/generation.rs"
HOST_DATA = ROOT / "zircon_editor/src/ui/retained_host/host_contract/data/template_nodes/weight_heatmap.rs"
PAINTER = ROOT / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_weight_heatmap.rs"
FIELD = ROOT / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_weight_heatmap/field.rs"


class WeightHeatmapGenerationContractTests(unittest.TestCase):
    def test_generation_owns_bounded_static_field_and_dynamic_selection_identity(self):
        source = GENERATION.read_text(encoding="utf-8")

        self.assertIn("pub(crate) struct WeightHeatmapGeneration", source)
        self.assertIn("static_field_for_plot_size", source)
        self.assertIn("STATIC_FIELD_CACHE_CAPACITY", source)
        self.assertIn("MAX_HEATMAP_CELLS", source)
        self.assertIn("normalized_unit_value", source)
        self.assertIn("source.selected", source)
        self.assertIn("insert_or_get", source)

    def test_host_and_painter_no_longer_materialize_model_rows_for_each_paint(self):
        host_data = HOST_DATA.read_text(encoding="utf-8")
        painter = PAINTER.read_text(encoding="utf-8")
        field = FIELD.read_text(encoding="utf-8")

        self.assertIn("WeightHeatmapGeneration", host_data)
        self.assertNotIn("ModelRc", host_data)
        self.assertNotIn("heatmap_sources", painter)
        self.assertIn("static_field_for_plot_size", field)
        self.assertNotIn("TemplatePaneWeightHeatmapSourceData", field)


if __name__ == "__main__":
    unittest.main()
