from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
BAKED_MESH_RS = ROOT / "zircon_runtime/src/navigation/runtime/baked_mesh.rs"


def compact(source: str) -> str:
    return re.sub(r"\s+", "", source)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime08dBorrowedPolygonIndicesPerformanceContractTests(unittest.TestCase):
    def test_polygon_build_borrows_the_asset_index_slice(self) -> None:
        source = BAKED_MESH_RS.read_text(encoding="utf-8")
        from_asset = compact(
            function_region(
                source,
                "    fn from_asset(",
                "    pub(super) fn contains_xz(",
            )
        )

        self.assertIn(
            "letindex_set=&asset.indices[start.min(asset.indices.len())..end];",
            from_asset,
        )
        self.assertNotIn(".to_vec()", from_asset)

    def test_edge_and_vertex_projection_share_the_borrowed_slice(self) -> None:
        source = BAKED_MESH_RS.read_text(encoding="utf-8")
        from_asset = compact(
            function_region(
                source,
                "    fn from_asset(",
                "    pub(super) fn contains_xz(",
            )
        )

        self.assertIn("letedge_keys=polygon_edge_keys(index_set);", from_asset)
        self.assertIn("letmutvertices=index_set.iter()", from_asset)

    def test_existing_polygon_topology_oracles_remain_present(self) -> None:
        source = BAKED_MESH_RS.read_text(encoding="utf-8")

        for test_name in (
            "adjacency_uses_the_shared_edge_index_not_rectangle_overlap",
            "triangle_indices_produce_canonical_undirected_edge_keys",
            "mesh_builder_connects_triangles_through_their_shared_edge",
        ):
            self.assertIn(f"fn {test_name}()", source)


if __name__ == "__main__":
    unittest.main()
