"""Keep Astra benchmark evidence scoped, bounded, and statistically explicit."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]

MANIFEST_BENCH = ROOT / "zircon_runtime/src/asset/project/manifest/validation/tests/optimization_batch_ir_runtime629_tests.rs"
GLTF_BENCH = ROOT / "zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/tests/optimization_batch_is_runtime629_tests.rs"
DIRTY_BENCH = ROOT / "zircon_editor/src/core/asset/dirty/registry/tests/optimization_tests.rs"
WIDGET_BENCH = ROOT / "zircon_editor/src/ui/asset_editor/promote_widget/tests/optimization_batch_iw_editor633_tests.rs"
EXPORT_BENCH = ROOT / "zircon_editor/src/core/export/pipeline/tests/optimization_batch_iv_editor633_tests.rs"

DOCS = [
    ROOT / "docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-manifest-membership.md",
    ROOT / "docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-gltf-mesh-dependency-index.md",
    ROOT / "docs/plans/optimize/zircon_editor/612/2026-09-01-reset-delta-direct-materialization.md",
    ROOT / "docs/plans/optimize/zircon_editor/633/2026-09-01-preallocated-widget-dependency-closure.md",
    ROOT / "docs/plans/optimize/zircon_editor/633/2026-09-01-bitset-export-stage-planning.md",
]


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


class AstraBenchmarkEvidenceScopeContractTests(unittest.TestCase):
    def assert_explicit_percentiles(self, source: str) -> None:
        self.assertIn("percentile_method=nearest_rank", source)
        self.assertRegex(source, r"p50(?:_ns)?")
        self.assertRegex(source, r"p95(?:_ns)?")
        self.assertRegex(source, r"p99(?:_ns)?")
        self.assertIn("benchmark_scope=microbenchmark", source)
        self.assertIn("real_caller_required=true", source)

    def benchmark_body(self, source: str, function_name: str) -> str:
        start = source.index(f"fn {function_name}")
        next_function = source.find("\nfn ", start + 4)
        return source[start:] if next_function < 0 else source[start:next_function]

    def test_manifest_benchmark_uses_the_product_root_bound(self):
        source = read(MANIFEST_BENCH)
        self.assertIn("use zircon_runtime_interface::project::MAX_PROJECT_ASSET_ROOTS;", source)
        self.assertIn("const ROOT_COUNT: usize = MAX_PROJECT_ASSET_ROOTS;", source)
        self.assertNotIn("32_768", source)
        self.assertIn("const SAMPLE_COUNT: usize = 31;", source)
        self.assert_explicit_percentiles(source)

    def test_other_benchmarks_report_p50_p95_p99_and_scope(self):
        cases = (
            (GLTF_BENCH, "optimization_batch_is_runtime629_preallocated_mesh_dependency_benchmark", "const SAMPLE_PAIRS: usize = 31;"),
            (DIRTY_BENCH, "optimization_batch_ic_editor612_reset_delta_seed_p95", "const SAMPLE_PAIRS: usize = 31;"),
            (WIDGET_BENCH, "optimization_batch_iw_editor633_preallocated_widget_dependency_benchmark", "const SAMPLE_PAIRS: usize = 31;"),
            (EXPORT_BENCH, "optimization_batch_r6_wave_editor633_export_stage_validation_bitset_p95", "const SAMPLE_PAIRS: usize = 31;"),
        )
        for path, function_name, sample_declaration in cases:
            with self.subTest(path=path):
                source = read(path)
                self.assertIn(sample_declaration, source)
                self.assert_explicit_percentiles(self.benchmark_body(source, function_name))

    def test_benchmark_docs_do_not_present_helpers_as_product_acceptance(self):
        for path in DOCS:
            with self.subTest(path=path):
                text = read(path).lower()
                self.assertIn("microbenchmark", text)
                self.assertRegex(text, r"real caller|真实 caller|真实调用")
                self.assertRegex(text, r"pending|待测|待验证")
                self.assertRegex(text, r"p50.*p95.*p99|p50/p95/p99")

    def test_astra_performance_plan_records_the_correction(self):
        text = read(ROOT / "docs/plans/astra/performance/01-bounded-hotpaths.md").lower()
        self.assertIn("runtime629", text)
        self.assertIn("max_project_asset_roots", text)
        self.assertIn("microbenchmark", text)
        self.assertIn("p50/p95/p99", text)
        self.assertRegex(text, r"真实 caller|real caller")


if __name__ == "__main__":
    unittest.main()
