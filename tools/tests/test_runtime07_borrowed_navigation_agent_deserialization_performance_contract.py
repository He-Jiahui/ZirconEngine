from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/gameplay_host/navigation.rs"


class BorrowedNavigationAgentDeserializationPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.function = cls.source.split(
            "pub(super) fn move_entity_with_navigation", maxsplit=1
        )[1].split("pub(super) fn navigation_next_point", maxsplit=1)[0]

    def test_navigation_agent_deserializes_from_borrowed_json(self) -> None:
        self.assertIn("use serde::Deserialize;", self.source)
        self.assertIn(
            "NavMeshAgentDescriptor::deserialize(value).ok()",
            self.function,
        )

    def test_navigation_agent_path_does_not_clone_the_json_tree(self) -> None:
        self.assertNotIn("value.clone()", self.function)
        self.assertNotIn("serde_json::from_value", self.function)

    def test_rust_guard_preserves_descriptor_round_trip(self) -> None:
        self.assertIn(
            "borrowed_navigation_agent_deserialization_preserves_descriptor",
            self.source,
        )
        self.assertIn("NavMeshAgentDescriptor::deserialize(&value)", self.source)


if __name__ == "__main__":
    unittest.main()
