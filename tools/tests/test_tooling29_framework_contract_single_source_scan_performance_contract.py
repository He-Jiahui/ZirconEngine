import ast
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_PATH = REPO_ROOT / "tools" / "audits" / "framework_contract_partition_audit.py"


class FrameworkContractSingleSourceScanPerformanceContractTests(unittest.TestCase):
    def test_audit_reuses_the_inventory_source_path_snapshot(self) -> None:
        module = ast.parse(AUDIT_PATH.read_text(encoding="utf-8"))
        functions = {
            node.name: node
            for node in module.body
            if isinstance(node, ast.FunctionDef)
        }
        inventory = functions["_framework_source_inventory"]
        audit = functions["audit_framework_partition"]

        inventory_scans = [
            node
            for node in ast.walk(inventory)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr == "rglob"
        ]
        audit_scans = [
            node
            for node in ast.walk(audit)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr == "rglob"
        ]

        self.assertEqual(len(inventory_scans), 1)
        self.assertEqual(
            audit_scans,
            [],
            "audit must reuse source_paths returned by the inventory instead of walking the tree twice",
        )
        self.assertIn("source_paths", ast.unparse(audit))


if __name__ == "__main__":
    unittest.main()
