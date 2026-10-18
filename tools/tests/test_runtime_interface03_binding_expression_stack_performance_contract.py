import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / (
    "zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs"
)
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/compiler/binding_program/tests/expression_stack_performance_tests.rs"


class RuntimeInterface03BindingExpressionStackPerformanceContract(unittest.TestCase):
    def test_expression_stack_is_reused_across_binding_validation(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn is_well_formed(&self)")
        end = source.index("\n    pub fn handle_for_node_binding", start)
        body = source[start:end]

        stack_index = body.index("let mut expression_pending")
        binding_loop_index = body.index("self.bindings.iter().all")
        self.assertLess(stack_index, binding_loop_index)
        self.assertIn("UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY", body)
        self.assertIn("&mut expression_pending", body)

    def test_expression_stack_reuse_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("expression_is_well_formed_allocating", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_REUSED_BINDING_EXPRESSION_STACK_BENCH_V1",
            bench_source,
        )
        self.assertIn("saturating_mul(2)", bench_source)


if __name__ == "__main__":
    unittest.main()
