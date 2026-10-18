import ast
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
VALIDATOR_PATH = REPO_ROOT / "tools" / "validation" / "validate_performance_comparison_receipt.py"


class PerformanceComparisonReceiptSingleReportReadPerformanceContractTests(unittest.TestCase):
    def test_validator_reuses_each_report_payload_and_digest(self) -> None:
        module = ast.parse(VALIDATOR_PATH.read_text(encoding="utf-8"))
        functions = {
            node.name: node
            for node in module.body
            if isinstance(node, ast.FunctionDef)
        }

        loader_source = ast.unparse(functions["_load_json_report"])
        binding_source = ast.unparse(functions["_validate_report_binding"])
        distinct_source = ast.unparse(functions["_validate_distinct_reports"])

        self.assertEqual(loader_source.count("read_bytes()"), 1)
        self.assertIn("hashlib.sha256(payload)", loader_source)
        self.assertNotIn("read_text(", loader_source)
        self.assertNotIn("read_bytes(", binding_source)
        self.assertNotIn("read_bytes(", distinct_source)


if __name__ == "__main__":
    unittest.main()
