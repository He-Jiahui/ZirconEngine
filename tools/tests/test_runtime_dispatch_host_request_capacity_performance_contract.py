from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
OUTCOME = ROOT / "zircon_runtime/src/ui/dispatch/input_manager/outcome.rs"
LOWER_TEST = (
    ROOT / "zircon_runtime/src/ui/dispatch/input_manager/outcome/tests/host_request_capacity_tests.rs"
)


def rust_block(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated Rust block: {signature}")


class RuntimeDispatchHostRequestCapacityPerformanceContractTests(unittest.TestCase):
    def test_dispatch_metadata_reserves_a_lower_bound_without_a_second_result_pass(self) -> None:
        source = OUTCOME.read_text(encoding="utf-8")
        body = rust_block(source, "fn collect_dispatch_metadata(")

        self.assertEqual(body.count("for result in results"), 1)
        self.assertIn("let mut remaining_results = results.len();", body)
        self.assertIn("host_requests.reserve(remaining_results.max(result.host_requests.len()))", body)
        self.assertIn("remaining_results = remaining_results.saturating_sub(1);", body)
        self.assertIn("host_requests.extend(result.host_requests.iter().cloned())", body)
        self.assertNotIn("flat_map", body)
        self.assertNotIn("results.iter().any", body)

    def test_capacity_slice_has_a_lower_regression_and_managed_benchmark_marker(self) -> None:
        source = LOWER_TEST.read_text(encoding="utf-8")

        self.assertIn(
            "runtime_hotpath_batch_runtime820_dispatch_host_request_capacity_preserves_order",
            source,
        )
        self.assertIn(
            "RUNTIME820_DISPATCH_HOST_REQUEST_CAPACITY_BENCH_V1",
            source,
        )
        self.assertIn("optimized_reservations_per_batch=1", source)
        self.assertIn("optimized_lower_bound_capacity={RESULTS_PER_BATCH}", source)
        self.assertIn("const RESULTS_PER_BATCH: usize = 4_096;", source)

    def test_capacity_model_keeps_empty_batches_allocation_free(self) -> None:
        result_count = 4096
        legacy_reservations = 0
        optimized_reservations = 1
        optimized_lower_bound_capacity = result_count

        self.assertEqual(legacy_reservations, 0)
        self.assertEqual(optimized_reservations, 1)
        self.assertEqual(optimized_lower_bound_capacity, result_count)
        self.assertEqual(growth_events(result_count), 11)
        self.assertEqual(growth_events(result_count, result_count), 0)


def growth_events(item_count: int, initial_capacity: int = 0) -> int:
    capacity = initial_capacity
    events = 0
    for length in range(1, item_count + 1):
        if length > capacity:
            capacity = 4 if capacity == 0 else capacity * 2
            events += 1
    return events


if __name__ == "__main__":
    unittest.main()
