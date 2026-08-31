from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator.integration_candidates import (
    IntegrationCandidateService,
)


class IntegrationCandidateGitIndexBatchPerformanceContractTests(unittest.TestCase):
    def test_index_helper_uses_one_nul_delimited_git_batch(self) -> None:
        source = inspect.getsource(
            IntegrationCandidateService._update_index_entries
        )

        self.assertEqual(1, source.count("subprocess.run("))
        self.assertIn('["git", "update-index", "-z", "--index-info"]', source)
        self.assertIn('+ b"\\0"', source)

    def test_candidate_tree_batches_index_entries(self) -> None:
        source = inspect.getsource(
            IntegrationCandidateService._tree_with_candidate_blobs
        )

        self.assertIn("self._update_index_entries(paths, environment=environment)", source)
        self.assertNotIn('self._git(\n                    "update-index"', source)

    def test_shared_index_alignment_batches_index_entries(self) -> None:
        source = inspect.getsource(IntegrationCandidateService._align_shared_index)

        self.assertIn("self._update_index_entries(candidate.paths)", source)
        self.assertNotIn('self._git(\n                "update-index"', source)


if __name__ == "__main__":
    unittest.main()
