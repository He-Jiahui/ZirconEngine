from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator.ownership_matrix import OwnershipMatrixService


class OwnershipMatrixLeaseIndexPerformanceContractTests(unittest.TestCase):
    def test_live_leases_are_indexed_before_entry_projection(self) -> None:
        lease_source = inspect.getsource(OwnershipMatrixService._leases)
        entry_source = inspect.getsource(OwnershipMatrixService._entry)

        self.assertIn('str(row["path_key"]): row', lease_source)
        self.assertIn("ancestor_key", entry_source)
        self.assertIn("leases.get(ancestor_key)", entry_source)
        self.assertNotIn("for row in leases\n            if path_key ==", entry_source)


if __name__ == "__main__":
    unittest.main()
