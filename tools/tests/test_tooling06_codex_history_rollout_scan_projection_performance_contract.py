from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools/session_coordinator/codex_sync/history.py"


class CodexHistoryRolloutScanProjectionPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        service = next(
            node
            for node in tree.body
            if isinstance(node, ast.ClassDef) and node.name == "CodexHistoricalEvidenceCollector"
        )
        cls.functions = {
            node.name: node
            for node in service.body
            if isinstance(node, ast.FunctionDef)
        }

    def test_rollout_scan_does_not_probe_month_root_before_rglob(self) -> None:
        function = self.functions["_rollouts"]
        exists_calls = [
            node
            for node in ast.walk(function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr == "exists"
        ]
        self.assertEqual(exists_calls, [])

    def test_rollout_scan_streams_sorted_projection(self) -> None:
        function = self.functions["_rollouts"]
        tuple_calls = [
            node
            for node in ast.walk(function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "tuple"
            and not (
                isinstance(node.args[0], ast.Call)
                and isinstance(node.args[0].func, ast.Attribute)
                and node.args[0].func.attr == "values"
            )
        ]
        self.assertEqual(tuple_calls, [])


if __name__ == "__main__":
    unittest.main()
