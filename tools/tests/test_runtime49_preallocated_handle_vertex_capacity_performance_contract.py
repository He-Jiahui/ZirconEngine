from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PRIMITIVES = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/primitives"
)
HANDLE_VERTICES = PRIMITIVES / "handles/build_handle_vertices.rs"
APPEND = PRIMITIVES / "line_geometry/append"
APPEND_MOD = APPEND / "mod.rs"
LINE_GEOMETRY_MOD = PRIMITIVES / "line_geometry/mod.rs"


class Runtime49PreallocatedHandleVertexCapacityPerformanceContractTests(
    unittest.TestCase
):
    def test_handle_builder_preallocates_the_shared_topology_capacity(self) -> None:
        source = HANDLE_VERTICES.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]
        normalized = " ".join(production.split())

        self.assertIn("const LINE_VERTEX_CAPACITY: usize = 2;", production)
        self.assertIn("fn handle_element_vertex_capacity(", production)
        self.assertIn("ARROW_HEAD_VERTEX_CAPACITY", production)
        self.assertIn("CROSS_VERTEX_CAPACITY", production)
        self.assertIn("RING_VERTEX_CAPACITY", production)
        self.assertIn(".map(handle_element_vertex_capacity)", normalized)
        self.assertIn(".fold(0usize, usize::saturating_add)", normalized)
        self.assertIn("Vec::with_capacity(vertex_capacity)", production)
        self.assertNotIn("let mut vertices = Vec::new();", production)

    def test_line_helpers_export_the_capacity_constants_used_by_the_builder(self) -> None:
        append_exports = APPEND_MOD.read_text(encoding="utf-8")
        line_geometry_exports = LINE_GEOMETRY_MOD.read_text(encoding="utf-8")

        for constant in (
            "ARROW_HEAD_VERTEX_CAPACITY",
            "CROSS_VERTEX_CAPACITY",
            "RING_VERTEX_CAPACITY",
        ):
            self.assertIn(constant, append_exports)
            self.assertIn(constant, line_geometry_exports)

    def test_rust_coverage_locks_each_non_degenerate_helper_topology(self) -> None:
        helper_contracts = {
            "append_arrow_head.rs": (
                "ARROW_HEAD_VERTEX_CAPACITY",
                "arrow_head_capacity_matches_non_degenerate_output",
            ),
            "append_cross.rs": (
                "CROSS_VERTEX_CAPACITY",
                "cross_capacity_matches_output",
            ),
            "append_ring.rs": (
                "RING_VERTEX_CAPACITY",
                "ring_capacity_matches_non_degenerate_output",
            ),
        }

        for file_name, (constant, test_name) in helper_contracts.items():
            with self.subTest(file_name=file_name):
                source = (APPEND / file_name).read_text(encoding="utf-8")
                self.assertIn(f"pub(crate) const {constant}", source)
                self.assertIn(test_name, source)
                self.assertIn(f"assert_eq!(vertices.len(), {constant});", source)


if __name__ == "__main__":
    unittest.main()
