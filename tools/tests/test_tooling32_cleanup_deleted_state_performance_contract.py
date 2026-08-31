from __future__ import annotations

import unittest
from pathlib import Path


SCRIPT = (
    Path(__file__).resolve().parents[1]
    / "session_coordinator"
    / "cleanup.py"
)


class CleanupDeletedStatePerformanceContractTests(unittest.TestCase):
    def test_tracks_current_candidate_deletion_without_scanning_prior_results(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("target_deleted = False", source)
        self.assertIn("target_deleted = True", source)
        self.assertNotIn("target_text in deleted", source)

    def test_preserves_ordered_deleted_result(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("deleted.append(target_text)", source)
        self.assertIn("CleanupResult(tuple(deleted), tuple(denied))", source)


if __name__ == "__main__":
    unittest.main()
