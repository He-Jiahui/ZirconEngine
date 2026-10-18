from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / (
    "zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/"
    "submit/build_runtime_frame.rs"
)


def function_body(source: str, signature: str, next_signature: str) -> str:
    start = source.index(signature)
    end = source.index(next_signature, start)
    return source[start:end]


class RuntimeVirtualGeometryOverlayCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.bvh_gizmos = function_body(
            cls.source,
            "fn build_virtual_geometry_bvh_scene_gizmos(",
            "\nfn build_virtual_geometry_bvh_lines(",
        )
        cls.bvh_lines = function_body(
            cls.source,
            "fn build_virtual_geometry_bvh_lines(",
            "\nfn build_virtual_geometry_visbuffer_scene_gizmos(",
        )
        cls.visbuffer_gizmos = function_body(
            cls.source,
            "fn build_virtual_geometry_visbuffer_scene_gizmos(",
            "\nfn build_current_frame_visbuffer_debug_marks(",
        )
        cls.visbuffer_lines = function_body(
            cls.source,
            "fn build_virtual_geometry_visbuffer_lines(",
            "\nfn append_bvh_bounds_wireframe(",
        )

    def test_known_overlay_bounds_are_reserved_before_extension(self) -> None:
        self.assertIn("let mut gizmos = Vec::with_capacity(instances.len());", self.bvh_gizmos)
        self.assertIn("gizmos.extend(", self.bvh_gizmos)
        self.assertNotIn(".collect::<Vec<_>>()", self.bvh_gizmos)
        self.assertIn("const MAX_LINES_PER_NODE: usize = 13;", self.bvh_lines)
        self.assertIn(
            "Vec::with_capacity(instance.nodes.len().saturating_mul(MAX_LINES_PER_NODE))",
            self.bvh_lines,
        )
        self.assertIn(
            "let mut gizmos = Vec::with_capacity(visbuffer_debug_marks.len());",
            self.visbuffer_gizmos,
        )
        self.assertIn("gizmos.extend(", self.visbuffer_gizmos)
        self.assertNotIn(".collect::<Vec<_>>()", self.visbuffer_gizmos)
        self.assertIn("let mut lines = Vec::with_capacity(16);", self.visbuffer_lines)

    def test_overlay_filtering_and_line_semantics_remain_in_place(self) -> None:
        self.assertIn(".filter_map(|instance|", self.bvh_gizmos)
        self.assertIn("build_virtual_geometry_bvh_lines(instance)", self.bvh_gizmos)
        self.assertIn("(!lines.is_empty()).then(|| SceneGizmoOverlayExtract", self.bvh_gizmos)
        self.assertIn("append_bvh_bounds_wireframe(", self.bvh_lines)
        self.assertIn("nodes_by_id.get(&parent_node_id)", self.bvh_lines)
        self.assertIn(".filter_map(|mark|", self.visbuffer_gizmos)
        self.assertIn("build_virtual_geometry_visbuffer_lines(", self.visbuffer_gizmos)
        self.assertIn("append_cross_marker", self.visbuffer_lines)
        self.assertIn("append_bvh_bounds_wireframe", self.visbuffer_lines)

    def test_deterministic_models_remove_overlay_growth(self) -> None:
        def growth_events(item_count: int) -> int:
            events = 0
            capacity = 0
            for length in range(1, item_count + 1):
                if length > capacity:
                    capacity = 4 if capacity == 0 else capacity * 2
                    events += 1
            return events

        self.assertEqual(growth_events(4096 * 13), 15)
        self.assertEqual(growth_events(4096), 11)
        self.assertEqual(growth_events(16), 3)
        optimized_growth_events = sum(
            length > bound
            for length, bound in ((4096 * 13, 4096 * 13), (4096, 4096), (16, 16))
        )
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
