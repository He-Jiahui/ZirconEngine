from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator.ownership_transfers import OwnershipTransferService


class OwnershipTransferPathKeyProjectionPerformanceContractTests(unittest.TestCase):
    def test_preview_projects_each_path_key_once(self) -> None:
        source = inspect.getsource(OwnershipTransferService.preview)

        self.assertIn("path_key = path.casefold()", source)
        self.assertEqual(source.count("path.casefold()"), 1)
        self.assertIn("baseline_hashes.get(path_key)", source)
        self.assertIn("attributions.get(path_key)", source)

    def test_apply_projects_each_requested_path_key_once(self) -> None:
        source = inspect.getsource(OwnershipTransferService.apply)

        self.assertIn("path_key = item.path.casefold()", source)
        self.assertEqual(source.count("item.path.casefold()"), 1)
        self.assertGreaterEqual(source.count("path_key,"), 2)

    def test_precondition_projects_each_requested_path_key_once(self) -> None:
        source = inspect.getsource(
            OwnershipTransferService._validate_apply_preconditions
        )

        self.assertIn("path_key = item.path.casefold()", source)
        self.assertEqual(source.count("item.path.casefold()"), 1)
        self.assertIn("baseline_hashes.get(path_key)", source)
        self.assertIn("attributions.get(path_key)", source)


if __name__ == "__main__":
    unittest.main()
