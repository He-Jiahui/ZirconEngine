from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools/session_coordinator/failure_return_delegations.py"


class FailureDelegationAuthorizationSqlFilterPerformanceContract(unittest.TestCase):
    def test_authorization_query_filters_valid_requested_lifecycle_json(self) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        service = next(
            node
            for node in tree.body
            if isinstance(node, ast.ClassDef)
            and node.name == "FailureReturnDelegationService"
        )
        function = next(
            node
            for node in service.body
            if isinstance(node, ast.FunctionDef) and node.name == "_authorizations_for"
        )
        source = ast.unparse(function)
        self.assertIn("json_valid(payload_json)", source)
        self.assertIn("json_extract(payload_json", source)
        self.assertIn("placeholders", source)


if __name__ == "__main__":
    unittest.main()
