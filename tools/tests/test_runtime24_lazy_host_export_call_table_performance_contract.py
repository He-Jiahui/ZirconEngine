import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/script/vm/host/host_export_registry.rs"
).read_text(encoding="utf-8")


def function_body(signature: str) -> str:
    start = SOURCE.index(signature)
    opening = SOURCE.index("{", start)
    depth = 0
    for index in range(opening, len(SOURCE)):
        if SOURCE[index] == "{":
            depth += 1
        elif SOURCE[index] == "}":
            depth -= 1
            if depth == 0:
                return SOURCE[opening + 1 : index]
    raise AssertionError(f"unterminated Rust function: {signature}")


class LazyHostExportCallTablePerformanceContract(unittest.TestCase):
    def test_registration_only_advances_authoritative_generation(self) -> None:
        body = function_body("pub fn register_module(")

        self.assertNotIn("build_script_call_table", body)
        self.assertIn("state.generation = next_generation", body)

    def test_call_table_reads_share_the_generation_aware_builder(self) -> None:
        table_accessor = function_body("pub fn script_call_table(&self)")
        call_path = function_body("pub fn call_with_capabilities(")

        self.assertIn("current_script_call_table", table_accessor)
        self.assertIn("current_script_call_table", call_path)

    def test_builder_reuses_the_current_generation(self) -> None:
        body = function_body("fn current_script_call_table(")

        self.assertIn("state.call_table.generation() == state.generation", body)
        self.assertIn("build_script_call_table", body)

    def test_actual_release_benchmark_enforces_latency_and_copy_reduction(self) -> None:
        self.assertIn("fn runtime24_lazy_host_export_call_table_performance_acceptance()", SOURCE)
        self.assertIn("RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_WARMUP_PAIRS: usize = 4", SOURCE)
        self.assertIn("RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_SAMPLE_PAIRS: usize = 21", SOURCE)
        self.assertIn("registry.register_module(descriptor, callbacks)", SOURCE)
        self.assertIn("registry.script_call_table()", SOURCE)
        self.assertIn("legacy_samples_ns", SOURCE)
        self.assertIn("optimized_samples_ns", SOURCE)
        self.assertIn("optimized_p50_ns.saturating_mul(10) <= legacy_p50_ns", SOURCE)
        self.assertIn("optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns", SOURCE)
        self.assertIn("optimized_copy_count.saturating_mul(100) <= legacy_copy_count", SOURCE)
        self.assertIn("assert_eq!(legacy_checksum, optimized_checksum)", SOURCE)
        self.assertIn("RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_PERF", SOURCE)


if __name__ == "__main__":
    unittest.main()
