from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "tools/session_coordinator/manifest_retention.py"


class ManifestRetentionDatabaseSizeSingleProbePerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tree = ast.parse(SOURCE.read_text(encoding="utf-8"))

    def test_size_helper_uses_one_stat_without_exists_probe(self) -> None:
        helper = next(
            node
            for node in self.tree.body
            if isinstance(node, ast.FunctionDef) and node.name == "_file_size_or_zero"
        )
        calls = [
            node.func.attr
            for node in ast.walk(helper)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
        ]
        self.assertEqual(calls.count("stat"), 1)
        self.assertNotIn("exists", calls)

    def test_database_size_sites_share_the_single_probe_helper(self) -> None:
        calls = [
            node
            for node in ast.walk(self.tree)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "_file_size_or_zero"
        ]
        self.assertEqual(len(calls), 3)


if __name__ == "__main__":
    unittest.main()
