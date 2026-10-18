import ast
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PRESSURE_PATH = REPO_ROOT / "tools" / "performance" / "editor" / "editor_menu_pointer_resize_pressure.py"


class EditorMenuPointerSourceBindingSingleReadPerformanceContractTests(unittest.TestCase):
    def test_source_binding_reads_each_present_source_once(self) -> None:
        module = ast.parse(PRESSURE_PATH.read_text(encoding="utf-8"))
        build_source_binding = next(
            node
            for node in module.body
            if isinstance(node, ast.FunctionDef) and node.name == "build_source_binding"
        )
        calls = [
            node.func.attr
            for node in ast.walk(build_source_binding)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
        ]

        self.assertEqual(calls.count("read_bytes"), 1)
        self.assertNotIn("read_text", calls)
        self.assertNotIn("stat", calls)
        self.assertIn("decode", calls)
        self.assertIn("len", ast.unparse(build_source_binding))


if __name__ == "__main__":
    unittest.main()
