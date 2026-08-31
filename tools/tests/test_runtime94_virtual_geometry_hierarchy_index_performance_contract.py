from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
NODE_CULL = ROOT / (
    "zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/"
    "build_virtual_geometry_debug_snapshot/node_cull.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime94VirtualGeometryHierarchyIndexPerformanceContractTests(unittest.TestCase):
    def test_traversal_builds_one_borrowed_hierarchy_index(self) -> None:
        source = NODE_CULL.read_text(encoding="utf-8")
        traversal = function_region(
            source,
            "fn build_node_and_cluster_cull_traversal_records(",
            "fn build_node_and_cluster_cull_page_request_ids(",
        )

        self.assertEqual(traversal.count("index_hierarchy_nodes("), 1)
        self.assertIn("hierarchy_nodes_by_id.get(&node_id).copied()", traversal)
        self.assertNotIn(".iter().find(", traversal)

    def test_index_is_capacity_sized_and_preserves_first_duplicate(self) -> None:
        source = NODE_CULL.read_text(encoding="utf-8")
        index = function_region(source, "fn index_hierarchy_nodes(", "fn push_traversal_record(")

        self.assertIn("HashMap::with_capacity(hierarchy_nodes.len())", index)
        self.assertIn("nodes_by_id.entry(node.node_id).or_insert(node);", index)
        self.assertNotIn(".collect::<HashMap", index)

    def test_duplicate_first_match_semantics_are_covered_by_rust(self) -> None:
        source = NODE_CULL.read_text(encoding="utf-8")

        self.assertIn(
            "fn runtime94_hierarchy_node_index_preserves_first_authored_duplicate()",
            source,
        )
        self.assertIn("assert_eq!(nodes_by_id[&7].cluster_start, 11);", source)


if __name__ == "__main__":
    unittest.main()
