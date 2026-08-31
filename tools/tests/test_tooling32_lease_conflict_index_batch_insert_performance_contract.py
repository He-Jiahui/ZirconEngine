from __future__ import annotations

import inspect
import random
import unittest

from tools.session_coordinator import leases
from tools.session_coordinator.leases import LeaseService


class LeaseConflictIndexBatchInsertPerformanceContractTests(unittest.TestCase):
    def test_indexed_overlap_preserves_hierarchy_boundaries(self) -> None:
        requested = ("input", "tools/editor/panel.rs")
        requested_set = set(requested)

        self.assertTrue(
            leases._lease_path_overlaps_any("input", requested, requested_set)
        )
        self.assertTrue(
            leases._lease_path_overlaps_any(
                "input/runtime/frame.rs", requested, requested_set
            )
        )
        self.assertTrue(
            leases._lease_path_overlaps_any("tools", requested, requested_set)
        )
        self.assertFalse(
            leases._lease_path_overlaps_any("input_state", requested, requested_set)
        )
        self.assertFalse(
            leases._lease_path_overlaps_any(
                "tools/editor/panel.rs.bak", requested, requested_set
            )
        )

    def test_indexed_overlap_matches_pairwise_contract(self) -> None:
        randomizer = random.Random(20260831)
        requested = tuple(
            sorted(
                {
                    "/".join(
                        f"segment-{randomizer.randrange(32):02d}"
                        for _ in range(randomizer.randrange(1, 6))
                    )
                    for _ in range(500)
                }
            )
        )
        requested_set = set(requested)

        for _ in range(5000):
            candidate = "/".join(
                f"segment-{randomizer.randrange(32):02d}"
                for _ in range(randomizer.randrange(1, 7))
            )
            expected = any(
                leases.lease_paths_overlap(candidate, requested_key)
                for requested_key in requested
            )
            self.assertEqual(
                expected,
                leases._lease_path_overlaps_any(
                    candidate, requested, requested_set
                ),
                candidate,
            )

    def test_acquire_uses_indexed_conflicts_and_one_batch_insert(self) -> None:
        source = inspect.getsource(LeaseService.acquire_in_connection)

        self.assertIn("normalized_keys", source)
        self.assertIn("_lease_path_overlaps_any(", source)
        self.assertNotIn(
            "any(lease_paths_overlap(row[\"path_key\"], item.key)", source
        )
        self.assertEqual(1, source.count("connection.executemany("))


if __name__ == "__main__":
    unittest.main()
