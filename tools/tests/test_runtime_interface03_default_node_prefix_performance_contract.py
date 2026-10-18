import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/component/descriptor/default_node_template.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/component/descriptor/default_node_template/tests/prefix_performance_tests.rs"


class RuntimeInterface03DefaultNodePrefixPerformanceContract(unittest.TestCase):
    def test_native_template_uses_one_prefix_pass(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn native(")
        end = source.index("\n    pub fn with_node_id_prefix", start)
        body = source[start:end]

        self.assertIn("native_prefixes", body)
        self.assertIn("node_id_prefix", body)
        self.assertIn("control_id_prefix", body)
        self.assertNotIn("normalize_prefix", body)
        self.assertNotIn("control_prefix", body)

    def test_default_node_prefix_has_behavior_and_release_contract(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod prefix_performance_tests", source)
        self.assertIn("legacy_prefixes", bench)
        self.assertIn("RUNTIME_INTERFACE03_SINGLE_PASS_DEFAULT_NODE_PREFIX_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)


if __name__ == "__main__":
    unittest.main()
