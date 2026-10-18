from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools/session_coordinator/failure_return_delegations.py"


class FailureDelegationOriginLeaseSqlOverlapPerformanceContract(unittest.TestCase):
    def test_origin_lease_query_filters_separator_bounded_overlap_in_sql(self) -> None:
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
            if isinstance(node, ast.FunctionDef) and node.name == "_require_origin_lease"
        )
        source = ast.unparse(function)
        self.assertGreaterEqual(source.count("substr("), 2)
        overlap_calls = [
            node
            for node in ast.walk(function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "lease_paths_overlap"
        ]
        self.assertEqual(overlap_calls, [])


if __name__ == "__main__":
    unittest.main()
