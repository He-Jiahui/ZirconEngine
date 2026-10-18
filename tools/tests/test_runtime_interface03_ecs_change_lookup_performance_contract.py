import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
ECS_SOURCE = ROOT / "zircon_runtime_interface/src/ui/ecs.rs"
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/ecs/tests/query_performance_tests.rs"


class RuntimeInterface03EcsChangeLookupPerformanceContract(unittest.TestCase):
    def test_change_lookup_uses_binary_search_with_linear_fallback(self) -> None:
        source = ECS_SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn change(&self, node_id: UiNodeId)")
        end = source.index("\n    pub fn changes_by_kind(", start)
        body = source[start:end]

        self.assertIn("binary_search_by_key", body)
        self.assertIn("self.changes.iter().find", body)

    def test_change_lookup_has_release_benchmark(self) -> None:
        source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("RUNTIME_INTERFACE03_BINARY_ECS_CHANGE_LOOKUP_BENCH_V1", source)
        self.assertIn("change_linear", source)
        self.assertIn("saturating_mul(5)", source)


if __name__ == "__main__":
    unittest.main()
