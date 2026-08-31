from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXECUTABLE_CONTRACT_RS = ROOT / (
    "zircon_plugins/neural/editor/src/onnx/executable_contract.rs"
)


def source() -> str:
    return EXECUTABLE_CONTRACT_RS.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def validation_body() -> str:
    text = source().split("pub(super) fn validate_executable_v1_shapes", 1)[1]
    return text.split("fn gemm_shapes_are_executable", 1)[0]


class Plugins02StackBoundedShapeAdmissionContract(unittest.TestCase):
    def test_v1_shape_admission_has_an_explicit_stack_input_bound(self) -> None:
        self.assertIn("const MAX_EXECUTABLE_V1_INPUTS: usize = 5;", source())

    def test_input_shape_references_are_collected_on_the_stack(self) -> None:
        body = compact(validation_body())

        self.assertIn(
            "letmutinput_shapes:[&[u32];MAX_EXECUTABLE_V1_INPUTS]="
            "[&[];MAX_EXECUTABLE_V1_INPUTS];",
            body,
        )
        self.assertIn("inputs=&input_shapes[..node.inputs.len()]", body)

    def test_per_node_heap_collector_is_removed(self) -> None:
        body = compact(validation_body())

        self.assertNotIn("collect::<Option<Vec<_>>>()", body)

    def test_stack_bounded_admission_has_a_direct_rust_contract(self) -> None:
        self.assertIn("stack_bounded_input_shapes_preserve_relu_admission", source())

    def test_rust_benchmark_uses_the_managed_workload_and_sampling_contract(self) -> None:
        text = source()

        self.assertIn("plugins02_stack_bounded_shape_admission_performance", text)
        self.assertIn("const ADMISSIONS_PER_SAMPLE: usize = 65_536;", text)
        self.assertIn("const WARMUP_PAIRS: usize = 4;", text)
        self.assertIn("const SAMPLE_PAIRS: usize = 21;", text)
        self.assertIn('#[ignore = "managed performance evidence"]', text)

    def test_rust_benchmark_measures_allocator_and_latency_tails(self) -> None:
        body = compact(source())

        self.assertIn("#[global_allocator]", source())
        self.assertIn("legacy_allocations", source())
        self.assertIn("stack_allocations", source())
        self.assertIn("legacy_ns_raw", source())
        self.assertIn("stack_ns_raw", source())
        self.assertIn("p50_reduction_percent>=70.0", body)
        self.assertIn("p95_reduction_percent>=40.0", body)

    def test_rust_benchmark_emits_machine_readable_performance_evidence(self) -> None:
        self.assertIn(
            "PERF_RESULT plugins02_stack_bounded_shape_admission", source()
        )


if __name__ == "__main__":
    unittest.main()
