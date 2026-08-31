from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


class Editor04PlayInspectorQueryContractTests(unittest.TestCase):
    def test_world_query_has_a_typed_focused_inspector_projection(self) -> None:
        query = read("zircon_runtime_interface/src/world_sync/query.rs")

        self.assertIn("InspectionFields(WorldInspectionFieldsQuery)", query)
        self.assertIn("pub struct WorldInspectionFieldRow", query)
        self.assertIn("InspectionFields {", query)
        self.assertIn("EntityMissing {", query)

    def test_runtime_reuses_the_focused_fields_artifact_under_producer_bounds(self) -> None:
        snapshot = read("zircon_runtime/src/scene/inspection/snapshot.rs")

        self.assertIn("WorldQuery::InspectionFields", snapshot)
        self.assertIn("inspection_fields_artifact", snapshot)
        self.assertIn("WorldQueryInspectionFieldsCandidate", snapshot)
        self.assertIn("budget.validate_payload", snapshot)

    def test_runtime_and_interface_share_one_inspection_field_owner(self) -> None:
        runtime_mod = read("zircon_runtime/src/scene/inspection/mod.rs")
        runtime_field = ROOT / "zircon_runtime/src/scene/inspection/field.rs"

        self.assertIn(
            "WorldInspectionFieldRow as WorldInspectionField", runtime_mod
        )
        self.assertFalse(runtime_field.exists())

    def test_foreign_output_counts_focused_fields_structurally(self) -> None:
        host_count = read("zircon_runtime_host/src/foreign_output/item_count.rs")
        frame_count = read("zircon_runtime/src/dynamic_api/frame.rs")

        for source in (host_count, frame_count):
            self.assertIn("WorldQueryResult::InspectionFields", source)
            self.assertIn("fields.len()", source)
            self.assertIn("WorldQueryResult::EntityMissing", source)


if __name__ == "__main__":
    unittest.main()
