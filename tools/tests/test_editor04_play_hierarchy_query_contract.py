from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class Editor04PlayHierarchyQueryContractTests(unittest.TestCase):
    def test_world_query_has_typed_component_and_hierarchy_projections(self) -> None:
        source = (ROOT / "zircon_runtime_interface/src/world_sync/query.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub enum WorldQuery", source)
        self.assertIn("Components(ComponentWorldQuery)", source)
        self.assertIn("Hierarchy(WorldHierarchyQuery)", source)

    def test_every_materialized_query_result_carries_generation(self) -> None:
        source = (ROOT / "zircon_runtime_interface/src/world_sync/query.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("ComponentRows {", source)
        self.assertIn("HierarchyRows {", source)
        self.assertGreaterEqual(source.count("generation: u64"), 3)
        self.assertNotIn("Rows(Vec<EntityRow>)", source)

    def test_runtime_inspection_reuses_the_interface_hierarchy_row(self) -> None:
        source = (ROOT / "zircon_runtime/src/scene/inspection/hierarchy.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("WorldHierarchyRow as WorldInspectionHierarchyRow", source)
        self.assertNotIn("pub struct WorldInspectionHierarchyRow", source)

    def test_runtime_query_uses_the_cached_hierarchy_artifact(self) -> None:
        source = (ROOT / "zircon_runtime/src/scene/inspection/snapshot.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("WorldQuery::Hierarchy", source)
        self.assertIn("self.inspection_artifact()", source)
        self.assertIn("hierarchy_result_for_generation", source)


if __name__ == "__main__":
    unittest.main()
