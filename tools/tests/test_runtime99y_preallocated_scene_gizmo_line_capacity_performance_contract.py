from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCENE_GIZMO = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_gizmo"
)
BUILDER = SCENE_GIZMO / "build/build_scene_gizmo_line_vertices.rs"
APPEND = SCENE_GIZMO / "append"
APPEND_MOD = APPEND / "mod.rs"


class Runtime99YPreallocatedSceneGizmoLineCapacityPerformanceContractTests(
    unittest.TestCase
):
    def test_builder_preallocates_the_visible_line_topology_capacity(self) -> None:
        source = BUILDER.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]
        normalized = " ".join(production.split())

        self.assertIn("const LINE_VERTEX_CAPACITY: usize = 2;", production)
        self.assertIn("fn scene_gizmo_line_vertex_capacity", production)
        self.assertIn(".map(wire_shape_vertex_capacity)", normalized)
        self.assertIn(
            ".filter(|icon| !has_icon_texture(icon.id))",
            normalized,
        )
        self.assertIn(".map(icon_fallback_vertex_capacity)", normalized)
        self.assertEqual(production.count("has_icon_texture(icon.id)"), 2)
        self.assertIn("Vec::with_capacity(vertex_capacity)", production)
        self.assertNotIn("let mut vertices = Vec::new();", production)

    def test_append_module_exports_both_capacity_functions(self) -> None:
        source = APPEND_MOD.read_text(encoding="utf-8")

        self.assertIn("wire_shape_vertex_capacity", source)
        self.assertIn("icon_fallback_vertex_capacity", source)

    def test_rust_coverage_locks_capacity_to_real_helper_output(self) -> None:
        wire_source = (APPEND / "append_wire_shape.rs").read_text(encoding="utf-8")
        icon_source = (APPEND / "append_icon_fallback_lines.rs").read_text(
            encoding="utf-8"
        )
        builder_source = BUILDER.read_text(encoding="utf-8")

        self.assertIn(
            "scene_gizmo_line_capacity_matches_non_degenerate_wire_shapes",
            wire_source,
        )
        self.assertIn(
            "assert_eq!(vertices.len(), wire_shape_vertex_capacity(&shape));",
            wire_source,
        )
        self.assertIn(
            "scene_gizmo_line_capacity_matches_icon_fallbacks",
            icon_source,
        )
        self.assertIn(
            "assert_eq!(vertices.len(), icon_fallback_vertex_capacity(&icon));",
            icon_source,
        )
        self.assertIn(
            "scene_gizmo_line_capacity_counts_only_missing_icon_fallbacks",
            builder_source,
        )


if __name__ == "__main__":
    unittest.main()
