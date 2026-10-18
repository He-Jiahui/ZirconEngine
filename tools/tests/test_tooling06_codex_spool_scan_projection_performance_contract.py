from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools/session_coordinator/codex_sync/spool.py"


class CodexSpoolScanProjectionPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        service = next(
            node
            for node in tree.body
            if isinstance(node, ast.ClassDef) and node.name == "CodexTriggerSpool"
        )
        cls.functions = {
            node.name: node
            for node in service.body
            if isinstance(node, ast.FunctionDef)
        }

    def test_pending_scans_do_not_probe_root_existence(self) -> None:
        for name in ("pending_count", "validated_pending"):
            exists_calls = [
                node
                for node in ast.walk(self.functions[name])
                if isinstance(node, ast.Call)
                and isinstance(node.func, ast.Attribute)
                and node.func.attr == "exists"
            ]
            self.assertEqual(exists_calls, [], name)

    def test_ordered_pending_has_no_tuple_projection_before_sort(self) -> None:
        function = self.functions["_ordered_pending"]
        tuple_calls = [
            node
            for node in ast.walk(function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "tuple"
        ]
        self.assertEqual(tuple_calls, [])


if __name__ == "__main__":
    unittest.main()
