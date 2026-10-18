from __future__ import annotations

import ast
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
STAGING_SOURCE = ROOT / "tools/session_coordinator/artifact_product_staging.py"
PROCESS_SOURCE = ROOT / "tools/session_coordinator/processes.py"


class ProductStagingProcessIdentitySingleHandlePerformanceContract(unittest.TestCase):
    def test_owner_match_uses_combined_process_identity_check(self) -> None:
        tree = ast.parse(STAGING_SOURCE.read_text(encoding="utf-8"))
        service = next(
            node
            for node in tree.body
            if isinstance(node, ast.ClassDef) and node.name == "ArtifactProductStagingService"
        )
        function = next(
            node
            for node in service.body
            if isinstance(node, ast.FunctionDef) and node.name == "owner_matches"
        )
        called_names = {
            node.func.id
            for node in ast.walk(function)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
        }
        self.assertIn("process_matches_creation_time", called_names)
        self.assertNotIn("process_is_alive", called_names)
        self.assertNotIn("process_creation_time", called_names)

    def test_combined_windows_check_opens_one_process_handle(self) -> None:
        tree = ast.parse(PROCESS_SOURCE.read_text(encoding="utf-8"))
        function = next(
            node
            for node in tree.body
            if isinstance(node, ast.FunctionDef)
            and node.name == "process_matches_creation_time"
        )
        opens = [
            node
            for node in ast.walk(function)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr == "OpenProcess"
        ]
        self.assertEqual(len(opens), 1)


if __name__ == "__main__":
    unittest.main()
