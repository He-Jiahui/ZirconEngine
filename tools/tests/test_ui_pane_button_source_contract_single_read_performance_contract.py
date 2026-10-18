import ast
import inspect
import unittest

from tools.analysis.performance.ui.ui_pane_button_fallback_damage_pressure import source_contract


class UiPaneButtonSourceContractSingleReadPerformanceContractTests(unittest.TestCase):
    def test_source_contract_reads_each_source_once(self) -> None:
        tree = ast.parse(inspect.getsource(source_contract))
        calls = [
            node.func.attr
            for node in ast.walk(tree)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
        ]

        self.assertEqual(2, calls.count("read_bytes"))
        self.assertNotIn("read_text", calls)


if __name__ == "__main__":
    unittest.main()
