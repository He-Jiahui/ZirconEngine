import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/window/runtime_event_adapter/tests/key_name_performance_tests.rs"


class RuntimeInterface03LogicalKeyNamePerformanceContract(unittest.TestCase):
    def test_unknown_logical_key_uses_stack_decimal_encoding(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn logical_key_name(")
        end = source.index("\n}\n", start) + len("\n}")
        body = source[start:end]

        self.assertIn("push_u32_decimal", body)
        self.assertNotIn("key_code.to_string()", body)
        self.assertIn("String::with_capacity(10)", body)

    def test_logical_key_name_has_behavior_and_release_contract(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod key_name_performance_tests", source)
        self.assertIn("logical_key_name_formatting", bench)
        self.assertIn("RUNTIME_INTERFACE03_STACK_LOGICAL_KEY_NAME_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)


if __name__ == "__main__":
    unittest.main()
