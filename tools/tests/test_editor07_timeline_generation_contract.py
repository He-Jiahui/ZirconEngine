from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    path = ROOT / relative
    return path.read_text(encoding="utf-8") if path.exists() else ""


class Editor07TimelineGenerationContractTests(unittest.TestCase):
    def test_ui_exposes_one_timeline_generation_owner(self) -> None:
        ui = source("zircon_editor/src/ui/mod.rs")
        module = source("zircon_editor/src/ui/timeline_strip/mod.rs")

        self.assertIn("pub(crate) mod timeline_strip;", ui)
        for required in [
            "TimelineStripGeneration",
            "TimelineStripGenerationInput",
            "TimelineStripKey",
            "TimelineStripTick",
        ]:
            self.assertIn(required, module)

    def test_generation_owns_preformatted_visual_ticks_and_identity(self) -> None:
        generation = source("zircon_editor/src/ui/timeline_strip/generation.rs")

        for required in [
            "static_generation",
            "dynamic_generation",
            "format_time",
            "static_content_for_plot_width",
            "Arc<[TimelineStripTick]>",
            "Arc<[TimelineStripKey]>",
            "OnceLock",
            "STATIC_CONTENT_CACHE_CAPACITY",
            "static_content_cache_entry_count",
        ]:
            self.assertIn(required, generation)
        self.assertRegex(
            generation,
            re.compile(
                r"let dynamic_generation\s*=\s*dynamic_generation\(\s*current_time"
            ),
        )
        self.assertIn("static_generation(duration, tick_interval, &track_label, &keys)", generation)
        self.assertIn("dynamic_generation(current_time, &keys)", generation)
        self.assertNotIn("Arc<Mutex<BTreeMap<usize", generation)

    def test_projection_hard_cuts_raw_timeline_fields(self) -> None:
        projection = source(
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
            "pane_component_projection/timeline_strip.rs"
        )
        host_data = source(
            "zircon_editor/src/ui/retained_host/host_contract/data/"
            "template_nodes/timeline_strip.rs"
        )

        self.assertIn("TimelineStripGeneration::new", projection)
        self.assertIn("generation: TimelineStripGeneration", host_data)
        for retired_field in ["pub duration:", "pub current_time:", "pub tick_interval:", "ModelRc<"]:
            self.assertNotIn(retired_field, host_data)

    def test_painter_consumes_generation_without_tick_formatting_or_model_reads(self) -> None:
        painter = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_timeline_strip.rs"
        )
        surface = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_timeline_strip/surface.rs"
        )
        text = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_timeline_strip/text.rs"
        )
        keys = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_timeline_strip/keys.rs"
        )

        self.assertIn("static_content_for_plot_width", painter)
        for owner in [surface, text, keys]:
            self.assertNotIn("row_count()", owner)
            self.assertNotIn("row_data(", owner)
        self.assertNotIn("fn timeline_ticks", surface)
        self.assertNotIn("fn format_time", text)
        self.assertIn("tick.label()", text)

    def test_behavior_suite_covers_static_dynamic_and_visual_budget_boundaries(self) -> None:
        tests = source("zircon_editor/src/ui/timeline_strip/tests.rs")

        for test_name in [
            "ticks_are_preformatted_once_per_visual_budget",
            "scrub_changes_only_dynamic_generation",
            "reprojection_reuses_static_content_for_scrub",
            "track_or_tick_changes_update_static_generation",
            "key_geometry_changes_static_and_selection_changes_dynamic",
            "visual_budget_is_bounded_and_preserves_endpoints",
            "visual_budget_clamps_to_the_hard_cap",
            "visual_budget_cache_is_bounded",
            "invalid_input_is_normalized",
        ]:
            self.assertIn(f"fn {test_name}", tests)


if __name__ == "__main__":
    unittest.main()
