import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
REGISTRY = (
    REPO_ROOT
    / "zircon_runtime_interface"
    / "src"
    / "hub_protocol"
    / "recent_projects"
    / "registry.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?:\s*<[^>{{}}]*>)?\s*\(",
        source,
    )
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class RecentRegistryPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.registry = REGISTRY.read_text(encoding="utf-8")

    def test_validation_checks_canonical_order_without_remerging(self) -> None:
        validate = function_body(self.registry, "validate")
        self.assertIn("previous_project", validate)
        self.assertIn("is_canonical_successor", validate)
        self.assertIn("keys.entry(key)", validate)
        self.assertIn("Entry::Vacant", validate)
        self.assertIn("Entry::Occupied", validate)
        self.assertNotIn("keys.insert(key.clone()", validate)
        self.assertNotIn("self.projects.iter().cloned()", validate)
        self.assertNotIn("merge_hub_recent_projects", validate)
        self.assertEqual(validate.count("for project in &self.projects"), 1)
        self.assertIn("let mut canonical_order = true", validate)
        self.assertIn("if !canonical_order", validate)
        order = function_body(self.registry, "is_canonical_successor")
        self.assertIn("last_opened_unix_ms", order)
        self.assertIn("previous.path <= current.path", order)

    def test_release_evidence_tracks_clone_and_normalization_reduction(self) -> None:
        self.assertIn(
            "PERF_RESULT runtime_interface06_recent_registry_validation",
            self.registry,
        )
        self.assertIn("legacy_entry_clones=8", self.registry)
        self.assertIn("optimized_entry_clones=0", self.registry)
        self.assertIn("legacy_path_normalizations=16", self.registry)
        self.assertIn("optimized_path_normalizations=8", self.registry)
        self.assertIn("legacy_accepted_path_key_clones=8", self.registry)
        self.assertIn("optimized_accepted_path_key_clones=0", self.registry)
        self.assertIn("legacy_entry_visits=16", self.registry)
        self.assertIn("optimized_entry_visits=8", self.registry)
        self.assertIn("PERF_VALIDATIONS_PER_SAMPLE: usize = 128", self.registry)
        self.assertIn("validations_per_sample={PERF_VALIDATIONS_PER_SAMPLE}", self.registry)
        self.assertIn("fn measure_batch", self.registry)

if __name__ == "__main__":
    unittest.main()
