import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "tools/runtime_ui_pointer_path_authority_pressure.py"
SPEC = importlib.util.spec_from_file_location("pointer_path_authority_pressure", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class RuntimeUiPointerPathAuthorityPressureTests(unittest.TestCase):
    def test_ordinary_pointer_reduces_three_owned_sequences_to_one(self) -> None:
        report = MODULE.run(10, route_depths=(64,))
        ordinary = report["cases"][0]["ordinary"]

        self.assertEqual(ordinary["head_owned_path_sequences_per_event"], 3)
        self.assertEqual(ordinary["candidate_owned_path_sequences_per_event"], 1)
        self.assertEqual(ordinary["head_vec_allocations_lower_bound"], 30)
        self.assertEqual(ordinary["candidate_vec_allocations_lower_bound"], 10)
        self.assertEqual(ordinary["head_node_identity_writes"], 1_920)
        self.assertEqual(ordinary["candidate_node_identity_writes"], 640)

    def test_capture_keeps_distinct_physical_and_dispatch_authorities(self) -> None:
        report = MODULE.run(10, route_depths=(64,))
        captured = report["cases"][0]["captured_with_physical_hit"]

        self.assertEqual(captured["head_owned_path_sequences_per_event"], 3)
        self.assertEqual(captured["candidate_owned_path_sequences_per_event"], 2)
        self.assertEqual(captured["head_node_identity_writes"], 1_920)
        self.assertEqual(captured["candidate_node_identity_writes"], 1_280)

    def test_report_refuses_non_positive_inputs(self) -> None:
        with self.assertRaises(ValueError):
            MODULE.run(0)
        with self.assertRaises(ValueError):
            MODULE.run(1, route_depths=(0,))

    def test_report_is_bound_to_worktree_and_head_baseline_sources(self) -> None:
        binding = MODULE.run(1, route_depths=(1,))["source_binding"]

        self.assertTrue(binding["git_revision"])
        self.assertTrue(binding["manifest_sha256"])
        self.assertIn(
            "zircon_runtime_interface/src/ui/surface/hit.rs",
            binding["worktree_sha256"],
        )
        self.assertIn(
            "zircon_runtime/src/ui/surface/surface/event_routing.rs",
            binding["head_baseline_sha256"],
        )


if __name__ == "__main__":
    unittest.main()
