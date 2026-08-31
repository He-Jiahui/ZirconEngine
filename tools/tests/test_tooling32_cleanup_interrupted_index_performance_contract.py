from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator.cleanup_deletion import (
    interrupted_target_deletions,
)


class CleanupInterruptedIndexPerformanceContractTests(unittest.TestCase):
    def test_interrupted_lookup_builds_target_index_once(self) -> None:
        source = inspect.getsource(interrupted_target_deletions)

        self.assertIn("latest_by_target", source)
        self.assertEqual(1, source.count("for deletion_id, payload in started.items():"))
        self.assertNotIn("reversed(tuple(started.items()))", source)
        self.assertIn("latest_by_target.get(target_key)", source)


if __name__ == "__main__":
    unittest.main()
