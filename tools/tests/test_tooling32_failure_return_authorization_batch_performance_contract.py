from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator.failure_return_delegations import (
    FailureReturnDelegationService,
)


class FailureReturnAuthorizationBatchPerformanceContractTests(unittest.TestCase):
    def test_prepare_proofs_loads_authorizations_once(self) -> None:
        source = inspect.getsource(FailureReturnDelegationService.prepare_proofs)

        self.assertEqual(1, source.count("self._authorizations_for("))
        self.assertIn("authorizations.get(lifecycle_key)", source)
        self.assertNotIn("self._authorization_for(", source)

    def test_authorization_projection_keeps_latest_requested_key(self) -> None:
        source = inspect.getsource(FailureReturnDelegationService)

        self.assertIn("requested_keys = set(lifecycle_keys)", source)
        self.assertIn("lifecycle_key not in requested_keys", source)
        self.assertIn("authorizations.setdefault(", source)


if __name__ == "__main__":
    unittest.main()
