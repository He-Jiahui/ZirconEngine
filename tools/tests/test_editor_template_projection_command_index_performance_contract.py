from pathlib import Path
import unittest

from tools.editor_template_projection_command_index_pressure import run


ROOT = Path(__file__).resolve().parents[2]
PROJECTION_CACHE = ROOT / (
    "zircon_editor/src/ui/layouts/views/view_projection/projection_cache.rs"
)
COMMAND_INDEX = ROOT / (
    "zircon_editor/src/ui/layouts/views/view_projection/projection_cache/"
    "render_command_index.rs"
)


class EditorTemplateProjectionCommandIndexPerformanceContractTests(unittest.TestCase):
    def source(self) -> str:
        return PROJECTION_CACHE.read_text(encoding="utf-8")

    def test_projection_cache_retains_node_command_ranges(self) -> None:
        source = self.source()
        index_source = COMMAND_INDEX.read_text(encoding="utf-8")

        self.assertIn("render_command_index: ViewTemplateRenderCommandIndex", source)
        self.assertIn(
            "render_command_ranges: BTreeMap<UiNodeId, (usize, usize)>",
            index_source,
        )
        self.assertIn("pub(super) fn build", index_source)
        self.assertIn("pub(super) fn indexed_render_commands", index_source)

    def test_text_only_update_avoids_full_topology_and_geometry_scans(self) -> None:
        source = self.source()
        update = source.split("let mut changed_geometry = BTreeMap::new();", 1)[1].split(
            "entry.width_bits = width_bits;", 1
        )[0]

        self.assertIn("topology_requires_full_sync", update)
        self.assertIn("render_command_index_matches_changed_bindings", update)
        self.assertIn("collect_changed_geometry", update)
        self.assertNotIn(
            "for command in &entry.surface.render_extract.list.commands", update
        )

    def test_pressure_model_replaces_four_full_passes_with_local_ranges(self) -> None:
        result = run(
            update_count=4096,
            render_command_count=65_536,
            changed_control_count=8,
            changed_geometry_node_count=8,
            commands_per_node=2,
        )

        self.assertEqual(result["retired"]["command_visits"], 1_073_741_824)
        self.assertEqual(result["indexed"]["command_visits"], 131_072)
        self.assertEqual(result["delta"]["avoided_command_visits"], 1_073_610_752)
        self.assertEqual(result["delta"]["work_reduction_ratio"], 8192.0)


if __name__ == "__main__":
    unittest.main()
