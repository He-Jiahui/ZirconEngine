from pathlib import Path
import re
import unittest

from tools.runtime01_shared_registry_name_pressure import run


ROOT = Path(__file__).resolve().parents[2]
REGISTRY_NAME_RS = ROOT / (
    "zircon_runtime/src/core/runtime/descriptors/registry_name.rs"
)
BATCH_VALIDATOR = ROOT / (
    "tools/zircon-validation-runtime01-shared-registry-name-batch.ps1"
)


def source() -> str:
    return REGISTRY_NAME_RS.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def function_body(start: str, end: str) -> str:
    text = source()
    return text.split(start, 1)[1].split(end, 1)[0]


class Runtime01SharedRegistryNameContract(unittest.TestCase):
    def test_registry_name_uses_shared_immutable_value_storage(self) -> None:
        text = source()
        definition = compact(text.split("impl RegistryName", 1)[0])

        self.assertIn("usestd::sync::Arc;", compact(text))
        self.assertIn("value:Arc<str>", definition)
        self.assertNotIn("value:String", definition)

    def test_new_promotes_the_validated_owned_string_without_cloning(self) -> None:
        body = compact(function_body("pub fn new(", "pub fn from_parts("))

        self.assertGreaterEqual(body.count("InvalidRegistryName(value)"), 3)
        self.assertIn("value:Arc::from(value)", body)
        self.assertNotIn("Arc::from(value.clone())", body)

    def test_from_parts_promotes_the_built_string_without_cloning(self) -> None:
        body = compact(function_body("pub fn from_parts(", "pub fn as_str("))

        self.assertIn("value:Arc::from(value)", body)
        self.assertNotIn("Arc::from(value.clone())", body)

    def test_shared_storage_behavior_has_a_rust_contract(self) -> None:
        self.assertIn("registry_name_clones_share_value_storage", source())

    def test_clone_work_model_eliminates_payload_allocations(self) -> None:
        clone_work = run()["clone_work"]

        self.assertEqual(clone_work["owned_clone_count"], 524_288)
        self.assertEqual(clone_work["baseline_payload_allocation_count"], 524_288)
        self.assertEqual(clone_work["candidate_payload_allocation_count"], 0)
        self.assertGreater(clone_work["baseline_cloned_payload_bytes"], 0)
        self.assertEqual(clone_work["candidate_cloned_payload_bytes"], 0)

    def test_release_benchmark_contract_is_explicit(self) -> None:
        text = source()

        self.assertIn("registry_name_clone_release_benchmark_evidence", text)
        self.assertIn("RUNTIME01_REGISTRY_NAME_BENCH_V1", text)
        self.assertIn("const NAMES: usize = 65_536", text)
        self.assertIn("const CLONES_PER_NAME: usize = 8", text)
        self.assertIn("const SAMPLE_PAIRS: usize = 21", text)
        self.assertEqual(text.count("saturating_mul(2) <= legacy_p"), 2)

    def test_batch_validator_runs_correctness_then_release_benchmark(self) -> None:
        text = BATCH_VALIDATOR.read_text(encoding="utf-8")

        self.assertEqual(text.count("[pscustomobject]@{"), 2)
        self.assertIn("registry_name_clones_share_value_storage", text)
        self.assertIn("registry_name_clone_release_benchmark_evidence", text)
        self.assertIn('"--ignored"', text)
        self.assertIn("RUNTIME01_BATCH_PASS", text)

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f",
        )
        self.assertEqual(len(binding["source_sha256"]), 3)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_model_rejects_invalid_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(names=0)
        with self.assertRaises(ValueError):
            run(clones_per_name=0)


if __name__ == "__main__":
    unittest.main()
