from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools/session_coordinator/offline_queue.py"


class OfflineQueueScanProjectionPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        service = next(
            node
            for node in tree.body
            if isinstance(node, ast.ClassDef) and node.name == "OfflineCommandSpool"
        )
        cls.function = next(
            node
            for node in service.body
            if isinstance(node, ast.FunctionDef) and node.name == "_ordered"
        )

    def test_scan_does_not_probe_root_before_glob(self) -> None:
        exists_calls = [
            node
            for node in ast.walk(self.function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr == "exists"
        ]
        self.assertEqual(exists_calls, [])

    def test_scan_does_not_copy_sorted_list_to_tuple(self) -> None:
        tuple_calls = [
            node
            for node in ast.walk(self.function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "tuple"
        ]
        self.assertEqual(tuple_calls, [])


if __name__ == "__main__":
    unittest.main()
