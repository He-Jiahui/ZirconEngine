from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools/session_coordinator/ownership_transfers.py"


class OwnershipTransferLiveLeaseSqlFilterPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        service = next(
            node
            for node in tree.body
            if isinstance(node, ast.ClassDef) and node.name == "OwnershipTransferService"
        )
        cls.function = next(
            node
            for node in service.body
            if isinstance(node, ast.FunctionDef) and node.name == "_live_leases"
        )

    def test_live_lease_query_filters_expiration_in_sql(self) -> None:
        sql_literals = [
            node.value
            for node in ast.walk(self.function)
            if isinstance(node, ast.Constant) and isinstance(node.value, str)
        ]
        self.assertTrue(
            any("expires_at >= ?" in value for value in sql_literals),
            sql_literals,
        )

    def test_live_lease_filter_does_not_parse_every_row_in_python(self) -> None:
        parse_calls = [
            node
            for node in ast.walk(self.function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "parse_utc"
        ]
        self.assertEqual(parse_calls, [])


if __name__ == "__main__":
    unittest.main()
