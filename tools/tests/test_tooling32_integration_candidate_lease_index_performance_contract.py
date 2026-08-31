from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator.integration_candidates import (
    IntegrationCandidateService,
)


class IntegrationCandidateLeaseIndexPerformanceContractTests(unittest.TestCase):
    def test_lease_evidence_indexes_rows_before_candidate_lookup(self) -> None:
        source = inspect.getsource(IntegrationCandidateService._lease_evidence)

        self.assertIn("rows_by_key", source)
        self.assertIn("ancestor_key", source)
        self.assertIn("rows_by_key.get(ancestor_key)", source)
        self.assertNotIn("for row in rows\n                    if path.key ==", source)


if __name__ == "__main__":
    unittest.main()
