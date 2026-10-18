import ast
import unittest
from pathlib import Path


SOURCE = Path(__file__).parents[1] / "runtime_domain_dependency_audit.py"


class RuntimeDomainDependencyLineIndexPerformanceContractTests(unittest.TestCase):
    def test_use_parser_indexes_newlines_once(self):
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        function = self._function(tree, "_rust_use_paths")
        calls = self._call_names(function)

        self.assertIn("_line_number_at", calls)
        self.assertNotIn("count", calls)

    def test_grouped_use_parser_reuses_the_same_line_index(self):
        tree = ast.parse(SOURCE.read_text(encoding="utf-8"))
        function = self._function(tree, "_grouped_crate_use_references")
        calls = self._call_names(function)

        self.assertIn("_line_number_at", calls)
        self.assertNotIn("count", calls)

    @staticmethod
    def _function(tree: ast.AST, name: str) -> ast.FunctionDef:
        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef) and node.name == name:
                return node
        raise AssertionError(f"missing function: {name}")

    @staticmethod
    def _call_names(node: ast.AST) -> set[str]:
        names: set[str] = set()
        for child in ast.walk(node):
            if not isinstance(child, ast.Call):
                continue
            function = child.func
            if isinstance(function, ast.Name):
                names.add(function.id)
            elif isinstance(function, ast.Attribute):
                names.add(function.attr)
        return names


if __name__ == "__main__":
    unittest.main()
