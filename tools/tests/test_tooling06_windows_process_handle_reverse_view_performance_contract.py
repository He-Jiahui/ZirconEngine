from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools/session_coordinator/processes.py"


class WindowsProcessHandleReverseViewPerformanceContract(unittest.TestCase):
    def test_termination_cleanup_does_not_copy_handle_views_to_tuples(self) -> None:
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        function = next(
            node
            for node in tree.body
            if isinstance(node, ast.FunctionDef)
            and node.name == "_terminate_windows_process_tree"
        )
        copied_views = [
            node
            for node in ast.walk(function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "tuple"
            and node.args
            and isinstance(node.args[0], ast.Call)
            and isinstance(node.args[0].func, ast.Attribute)
            and node.args[0].func.attr in {"items", "values"}
        ]
        self.assertEqual(copied_views, [])


if __name__ == "__main__":
    unittest.main()
