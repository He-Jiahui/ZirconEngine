import ast
import inspect
import unittest

from tools.analysis.performance.editor.editor_inspector_projection_pressure import build_source_binding


class EditorInspectorSourceBindingSingleReadPerformanceContractTests(unittest.TestCase):
    def test_source_binding_uses_one_content_read_per_present_source(self) -> None:
        tree = ast.parse(inspect.getsource(build_source_binding))
        calls = [
            node.func.attr
            for node in ast.walk(tree)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
        ]

        self.assertEqual(2, calls.count("read_bytes"))
        self.assertNotIn("read_text", calls)
        self.assertNotIn("stat", calls)


if __name__ == "__main__":
    unittest.main()
