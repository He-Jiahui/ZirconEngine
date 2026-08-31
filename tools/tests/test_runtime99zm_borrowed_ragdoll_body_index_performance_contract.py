from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_plugins/physics/runtime/src/skeletal/runtime.rs"


def source_text() -> str:
    return SOURCE.read_text(encoding="utf-8")


def compact_source() -> str:
    return "".join(source_text().split())


class Runtime99ZMBorrowedRagdollBodyIndexPerformanceContract(unittest.TestCase):
    def test_pose_feed_builds_one_joint_bounded_borrowed_body_index(self) -> None:
        source = source_text()
        compact = compact_source()

        self.assertIn("usestd::collections::{BTreeMap,HashMap};", compact)
        self.assertIn(
            "HashMap<EntityId,Option<&'aPhysicsBodySyncState>>",
            compact,
        )
        self.assertIn(
            "letmutbodies_by_entity=HashMap::with_capacity(requested_body_count);",
            compact,
        )
        self.assertIn("ifrequested_body_count==0{returnHashMap::new();}", compact)
        self.assertEqual(source.count("HashMap::with_capacity"), 1)

    def test_body_index_preserves_first_duplicate_entity(self) -> None:
        compact = compact_source()

        self.assertIn("joint.skeleton_binding.is_some()", compact)
        self.assertIn(
            "bodies_by_entity.entry(joint.entity).or_insert(None);",
            compact,
        )
        self.assertIn("slot.get_or_insert(body);", compact)
        self.assertNotIn("bodies_by_entity.insert(body.entity,Some(body));", compact)

    def test_pose_feed_reuses_the_index_for_parent_and_output_lookups(self) -> None:
        compact = compact_source()

        self.assertEqual(compact.count("index_synced_bodies(sync)"), 1)
        self.assertIn(
            "collect_synced_bone_world_by_path(sync,ragdolls,&bodies_by_entity)",
            compact,
        )
        self.assertIn(
            "resolve_joint_body(&bodies_by_entity,joint.entity,joint.connected_entity)",
            compact,
        )

    def test_joint_lookup_keeps_direct_entity_precedence_without_linear_scans(self) -> None:
        compact = compact_source()

        direct = "bodies_by_entity.get(&joint_entity).copied().flatten()"
        fallback = (
            "connected_entity.and_then(|entity|"
            "bodies_by_entity.get(&entity).copied().flatten())"
        )
        self.assertIn(direct, compact)
        self.assertIn(fallback, compact)
        self.assertLess(compact.index(direct), compact.index(fallback))
        self.assertNotIn("sync.bodies.iter()", compact)
        self.assertNotIn("HashMap<EntityId,PhysicsBodySyncState>", compact)


if __name__ == "__main__":
    unittest.main()
