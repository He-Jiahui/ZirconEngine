from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/gameplay_host/components.rs"


class StreamedComponentEntityIdsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.function = cls.source.split(
            "pub(super) fn find_by_component", maxsplit=1
        )[1].split("pub(super) fn entity_exists", maxsplit=1)[0]

    def test_entity_id_projection_serializes_borrowed_rows(self) -> None:
        self.assertIn("struct ComponentEntityIds<'a>", self.source)
        self.assertIn("&'a [(u64, &'a serde_json::Value)]", self.source)
        self.assertIn("serializer.serialize_seq(Some(self.0.len()))", self.source)
        self.assertIn("sequence.serialize_element(entity)?", self.source)

    def test_find_by_component_avoids_a_second_entity_vector(self) -> None:
        self.assertIn("to_json_string(&ComponentEntityIds(&rows))", self.function)
        self.assertNotIn("rows.into_iter()", self.function)
        self.assertNotIn("collect::<Vec", self.function)

    def test_rust_guard_preserves_entity_order_and_json_shape(self) -> None:
        self.assertIn(
            "component_entity_ids_serialize_in_row_order_without_values",
            self.source,
        )
        self.assertIn('r#"[7,11]"#', self.source)


if __name__ == "__main__":
    unittest.main()
