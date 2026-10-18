import ast
import inspect
import unittest

from tools.analysis.performance.runtime.runtime_ui_dispatch_route_sharing_pressure import (
    source_binding,
    validate_candidate_source_contract,
)


def _attribute_calls(function: object) -> list[str]:
    tree = ast.parse(inspect.getsource(function))
    return [
        node.func.attr
        for node in ast.walk(tree)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
    ]


class RuntimeUiDispatchRouteSourceBindingSingleReadPerformanceContractTests(
    unittest.TestCase
):
    def test_source_binding_reads_each_critical_source_once(self) -> None:
        calls = _attribute_calls(source_binding)
        self.assertEqual(1, calls.count("read_bytes"))
        self.assertNotIn("read_text", calls)

    def test_candidate_contract_accepts_preloaded_sources(self) -> None:
        parameters = inspect.signature(validate_candidate_source_contract).parameters
        self.assertIn("sources", parameters)
        source = inspect.getsource(source_binding)
        self.assertIn("validate_candidate_source_contract(", source)
        self.assertIn("payload.decode(\"utf-8\")", source)


if __name__ == "__main__":
    unittest.main()
