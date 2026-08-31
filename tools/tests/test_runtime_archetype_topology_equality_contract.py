import unittest
from pathlib import Path


class RuntimeArchetypeTopologyEqualityContractTests(unittest.TestCase):
    STATUS = (
        "runtime_08_15_archetype_topology_equality_receipt_"
        "static_passed_cargo_deferred"
    )

    def test_topology_receipt_replaces_constant_index_equality(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        index = (
            repo_root / "zircon_runtime/src/scene/ecs/archetype/index.rs"
        ).read_text(encoding="utf-8")
        tests = (
            repo_root / "zircon_runtime/src/scene/ecs/archetype/index/tests.rs"
        ).read_text(encoding="utf-8")

        for anchor in (
            "struct ArchetypeTopologySnapshot<'a>",
            "fn topology_snapshot(&self) -> ArchetypeTopologySnapshot<'_>",
            "self.index.by_signature == other.index.by_signature",
            "self.index.by_component == other.index.by_component",
            ".zip(&other.index.records)",
            "left.id() == right.id()",
            "left.signature() == right.signature()",
            "left.entities() == right.entities()",
            "self.topology_snapshot() == other.topology_snapshot()",
        ):
            self.assertIn(anchor, index)
        self.assertNotIn("fn eq(&self, _other: &Self) -> bool", index)
        self.assertNotIn("fn eq(&self, _other: &Self) -> bool {\n        true", index)

        for test_name in (
            "topology_snapshot_distinguishes_registered_signatures",
            "topology_snapshot_distinguishes_entity_row_membership",
            "topology_snapshot_ignores_diagnostics_and_membership_history",
        ):
            self.assertIn(f"fn {test_name}()", tests)

    def test_status_is_mirrored_by_runtime_structure_and_review_plans(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            "docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            "docs/plans/engine-code-structure-convention.md",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md",
        )
        for relative_path in mirrors:
            plan = (repo_root / relative_path).read_text(encoding="utf-8")
            self.assertIn(self.STATUS, plan, relative_path)


if __name__ == "__main__":
    unittest.main()
