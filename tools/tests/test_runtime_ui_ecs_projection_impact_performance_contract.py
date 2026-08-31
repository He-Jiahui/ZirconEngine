from pathlib import Path
import unittest

from tools.runtime_ui_ecs_projection_impact_pressure import run


ROOT = Path(__file__).resolve().parents[2]
COMPUTE = ROOT / "zircon_runtime_interface/src/ui/ecs/compute.rs"
SURFACE_PROJECTION = ROOT / "zircon_runtime/src/ui/surface/ecs_projection.rs"


def function_body(source: str, name: str, next_name: str) -> str:
    return source.split(f"fn {name}", 1)[1].split(f"fn {next_name}", 1)[0]


class RuntimeUiEcsProjectionImpactPerformanceContractTests(unittest.TestCase):
    def test_surface_delta_helpers_move_precomputed_derived_authorities(self) -> None:
        source = SURFACE_PROJECTION.read_text(encoding="utf-8")

        self.assertIn(".schedule_mask\n", source)
        self.assertIn(".schedule_impacts\n", source)
        self.assertIn(".dirty_domain_impacts\n", source)
        self.assertNotIn(".schedule_mask()", source)
        self.assertNotIn(".schedule_impacts()", source)
        self.assertNotIn(".dirty_domain_impacts()", source)

    def test_domain_and_schedule_impacts_bucket_in_one_input_pass(self) -> None:
        source = COMPUTE.read_text(encoding="utf-8")
        domain_body = function_body(
            source,
            "projection_dirty_domain_impacts_from_domains",
            "projection_schedule_impacts_from_domains",
        )
        schedule_body = function_body(
            source,
            "projection_schedule_impacts_from_domains",
            "projection_stage_dirty_reasons",
        )

        self.assertIn("for (node_id, domains) in domains_by_node", domain_body)
        self.assertIn("for (node_id, domains) in domains_by_node", schedule_body)
        self.assertNotIn("let entries =", domain_body)
        self.assertNotIn("let entries =", schedule_body)
        self.assertNotIn("for (node_id, domains) in &entries", schedule_body)
        self.assertEqual(schedule_body.count("from_dirty_domains(domains)"), 1)
        self.assertNotIn("projection_stage_dirty_reasons(stage", schedule_body)
        self.assertIn("ProjectionScheduleImpactBucket", schedule_body)

    def test_lower_regression_preserves_deterministic_deduplicated_outputs(self) -> None:
        source = COMPUTE.read_text(encoding="utf-8")

        self.assertIn(
            "projection_impact_buckets_preserve_canonical_order_and_deduplicate_nodes",
            source,
        )
        self.assertIn("node_ids.sort_unstable();", source)
        self.assertIn("node_ids.dedup();", source)
        self.assertIn("bucket.node_ids.sort_unstable();", source)
        self.assertIn("bucket.node_ids.dedup();", source)

    def test_pressure_model_removes_repeated_source_scans_and_temporary_vectors(self) -> None:
        result = run()

        self.assertEqual(
            result["repeated_scan_aggregation"]["total_source_entry_reads"],
            210_000,
        )
        self.assertEqual(
            result["single_pass_bucket_aggregation"]["total_source_entry_reads"],
            20_000,
        )
        self.assertEqual(
            result["delta"]["source_entry_read_reduction_ratio"], 10.5
        )
        self.assertEqual(
            result["single_pass_bucket_aggregation"][
                "per_node_stage_reason_vector_allocations"
            ],
            0,
        )
        self.assertEqual(
            result["single_pass_bucket_aggregation"]["intermediate_entry_slots"],
            0,
        )


if __name__ == "__main__":
    unittest.main()
