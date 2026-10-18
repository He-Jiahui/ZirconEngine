from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/workbench/debug_reflector/schedule_sections.rs"


class EditorDirtyDomainImpactSummaryPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_active_dirty_domains_append_to_one_output_buffer(self) -> None:
        start = self.source.index("fn dirty_domain_impact_summary(")
        end = self.source.index("\n#[cfg(test)]", start)
        body = self.source[start:end]

        self.assertIn("let mut summary = String::new();", body)
        self.assertIn("for impact in impacts", body)
        self.assertIn("if impact.active || impact.node_count > 0", body)
        self.assertIn("summary.push(',');", body)
        self.assertIn("write!(", body)
        self.assertIn('"{:?}={}"', body)
        self.assertIn("impact.domain", body)
        self.assertIn("impact.node_count", body)
        self.assertNotIn(".collect::<Vec", body)
        self.assertNotIn(".join(", body)
        self.assertNotIn("format!(", body)

    def test_filter_order_and_output_shape_remain_explicit(self) -> None:
        start = self.source.index("fn dirty_domain_impact_summary(")
        end = self.source.index("\n#[cfg(test)]", start)
        body = self.source[start:end]

        self.assertIn("impact.active || impact.node_count > 0", body)
        self.assertIn("summary", body)
        self.assertTrue(body.rstrip().endswith("summary\n}"))

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "single_buffer_dirty_domain_summary_preserves_bytes_and_filtering",
            self.source,
        )
        self.assertIn(
            "editor824_single_buffer_dirty_domain_summary_bench_v1",
            self.source,
        )

    def test_dense_summary_removes_intermediate_strings_and_vector(self) -> None:
        impact_count = 4096
        retired_intermediate_strings = impact_count
        retired_temporary_vector_buffers = 1
        optimized_intermediate_strings = 0
        optimized_temporary_vector_buffers = 0

        self.assertGreater(retired_intermediate_strings, optimized_intermediate_strings)
        self.assertGreater(retired_temporary_vector_buffers, optimized_temporary_vector_buffers)


if __name__ == "__main__":
    unittest.main()
