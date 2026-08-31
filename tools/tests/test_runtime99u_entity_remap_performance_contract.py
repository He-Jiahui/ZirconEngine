from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TRANSACTION = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "scene"
    / "dynamic_scene"
    / "scene"
    / "spawn"
    / "transaction.rs"
)


class Runtime99uEntityRemapPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = TRANSACTION.read_text(encoding="utf-8")
        cls.remap_body = cls.source.split("fn build_entity_remap(", 1)[1].split(
            "struct EntityIdReservationProbe<'world>", 1
        )[0]

    def test_remap_builder_reuses_one_successor_probe(self) -> None:
        self.assertIn(
            "const ENTITY_REMAP_SUCCESSOR_PROBE_MIN_ENTITIES: usize = 16;",
            self.source,
        )
        self.assertIn(
            "if scene.entities.len() < ENTITY_REMAP_SUCCESSOR_PROBE_MIN_ENTITIES",
            self.remap_body,
        )
        self.assertIn("return build_entity_remap_linear(scene, world);", self.remap_body)
        self.assertIn("let mut probe = EntityIdReservationProbe::new(world);", self.remap_body)
        self.assertIn("probe.reserve(entity.source_entity)?", self.remap_body)
        self.assertNotIn("let mut reserved = BTreeSet::new();", self.remap_body)

    def test_probe_caches_successors_for_occupied_entity_ids(self) -> None:
        self.assertIn("struct EntityIdReservationProbe<'world>", self.source)
        self.assertIn(
            "successor_by_occupied: HashMap<EntityId, Option<EntityId>>",
            self.source,
        )
        self.assertIn(
            "self.successor_by_occupied.get(&candidate).copied()",
            self.source,
        )

    def test_probe_compresses_the_walked_chain_after_each_reservation(self) -> None:
        self.assertIn("for skipped in self.path.drain(..)", self.source)
        self.assertIn(
            "self.successor_by_occupied.insert(skipped, successor);",
            self.source,
        )
        self.assertIn("self.world.contains_entity(candidate)", self.source)

    def test_transaction_wires_separate_behavior_and_release_benchmark_tests(self) -> None:
        self.assertIn('#[path = "transaction/performance_tests.rs"]', self.source)
        self.assertIn("mod performance_tests;", self.source)


if __name__ == "__main__":
    unittest.main()
