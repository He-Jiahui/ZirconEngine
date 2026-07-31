from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    path = ROOT / relative
    return path.read_text(encoding="utf-8") if path.exists() else ""


class Editor07SampleGridGenerationContractTests(unittest.TestCase):
    def test_ui_exposes_one_sample_grid_generation_owner(self) -> None:
        ui = source("zircon_editor/src/ui/mod.rs")
        module = source("zircon_editor/src/ui/sample_grid/mod.rs")

        self.assertIn("pub(crate) mod sample_grid;", ui)
        for required in ["SampleGridGeneration", "SampleGridPoint", "SampleGridTick"]:
            self.assertIn(required, module)

    def test_generation_splits_static_and_dynamic_identity(self) -> None:
        generation = source("zircon_editor/src/ui/sample_grid/generation.rs")

        for required in [
            "static_generation",
            "dynamic_generation",
            "format_tick",
            "Arc<[SampleGridTick]>",
            "Arc<[SampleGridPoint]>",
        ]:
            self.assertIn(required, generation)
        self.assertRegex(
            generation,
            re.compile(
                r"let dynamic_generation\s*=\s*dynamic_generation\(\s*input\.x_min"
            ),
        )

    def test_projection_builds_generation_before_host_paint(self) -> None:
        projection = source(
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
            "pane_component_projection/sample_grid.rs"
        )
        host_data = source(
            "zircon_editor/src/ui/retained_host/host_contract/data/"
            "template_nodes/sample_grid.rs"
        )

        self.assertIn("SampleGridGeneration::new", projection)
        self.assertIn("generation: SampleGridGeneration", host_data)
        self.assertNotIn("ModelRc", host_data)

    def test_painter_consumes_preformatted_typed_slices(self) -> None:
        surface = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_sample_grid/surface.rs"
        )
        text = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_sample_grid/text.rs"
        )
        points = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_sample_grid/points.rs"
        )

        for owner in [surface, text, points]:
            self.assertNotIn("row_count()", owner)
            self.assertNotIn("row_data(", owner)
        self.assertNotIn("fn format_tick", text)
        self.assertIn("tick.label()", text)

    def test_behavior_suite_locks_generation_boundaries(self) -> None:
        tests = source("zircon_editor/src/ui/sample_grid/tests.rs")

        for test_name in [
            "ticks_are_preformatted_once_in_generation",
            "selection_changes_only_dynamic_generation",
            "point_drag_changes_only_dynamic_generation",
            "axis_and_tick_changes_update_static_generation",
            "range_changes_update_static_and_dynamic_generation",
        ]:
            self.assertIn(f"fn {test_name}", tests)


if __name__ == "__main__":
    unittest.main()
