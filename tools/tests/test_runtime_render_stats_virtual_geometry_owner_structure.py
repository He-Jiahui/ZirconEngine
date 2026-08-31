import unittest
from pathlib import Path


class RuntimeRenderStatsVirtualGeometryOwnerStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.owner_dir = (
            self.repo_root
            / "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/virtual_geometry"
        )
        self.owner = self.owner_dir / "mod.rs"
        self.legacy_owner = self.owner_dir.with_suffix(".rs")

    def test_virtual_geometry_diagnostics_use_focused_child_owners(self) -> None:
        self.assertFalse(self.legacy_owner.exists(), self.legacy_owner)
        owner_source = self.owner.read_text(encoding="utf-8")
        production_lines = [
            line
            for line in owner_source.splitlines()
            if line.strip() and not line.lstrip().startswith("//")
        ]

        self.assertLessEqual(len(production_lines), 20)
        for declaration in (
            "mod admission;",
            "mod culling;",
            "mod debug;",
            "mod execution;",
            "mod residency;",
        ):
            self.assertIn(declaration, owner_source)
        for call in (
            "admission::record(store, stats);",
            "debug::record(store, stats);",
            "residency::record(store, stats);",
            "execution::record(store, stats);",
            "culling::record(store, stats);",
        ):
            self.assertIn(call, owner_source)
        self.assertNotIn("record_count(", owner_source)
        self.assertNotIn("record_bool(", owner_source)
        self.assertNotIn('"render.virtual_geometry.', owner_source)

        expected_children = {
            "admission.rs": (
                "render.virtual_geometry.cluster_budget",
                "render.virtual_geometry.payload.source.authored",
                "RenderVirtualGeometryPayloadSource::AutomaticFallback",
            ),
            "debug.rs": (
                "render.virtual_geometry.forced_mip_present",
                "render.virtual_geometry.debug.freeze_cull",
                "render.virtual_geometry.debug.visualize_visbuffer",
            ),
            "residency.rs": (
                "render.virtual_geometry.requested_page_count",
                "render.virtual_geometry.resident_page_count",
                "render.virtual_geometry.replaced_page_count",
            ),
            "execution.rs": (
                "render.virtual_geometry.indirect_draw_count",
                "render.virtual_geometry.execution_segment_count",
                "render.virtual_geometry.execution_missing_segment_count",
            ),
            "culling.rs": (
                "render.virtual_geometry.cluster_selection.input_source.unavailable",
                "render.virtual_geometry.node_and_cluster_cull.record_count",
                "render.virtual_geometry.hardware_rasterization_record_count",
            ),
        }
        child_sources = []
        for child_name, anchors in expected_children.items():
            child = self.owner_dir / child_name
            self.assertTrue(child.is_file(), child)
            child_source = child.read_text(encoding="utf-8")
            self.assertLess(child_source.count("\n") + 1, 260, child)
            self.assertIn("pub(super) fn record", child_source)
            for anchor in anchors:
                self.assertIn(anchor, child_source)
            child_sources.append(child_source)

        combined = "\n".join(child_sources)
        self.assertEqual(combined.count("record_count("), 40)
        self.assertEqual(combined.count("record_bool("), 14)
        self.assertEqual(combined.count('"render.virtual_geometry.'), 54)


if __name__ == "__main__":
    unittest.main()
