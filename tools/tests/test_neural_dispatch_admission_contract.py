import unittest
from pathlib import Path


class NeuralDispatchAdmissionContractTests(unittest.TestCase):
    """Static guard for checked Neural GPU dispatch admission.

    The planner must reject an unrepresentable batch/channel product and optional device-limit
    violations before publishing a compute pass descriptor.  This contract is intentionally
    source-only while managed Neural Cargo and GPU validation remain blocked.
    """

    def setUp(self) -> None:
        root = Path(__file__).resolve().parents[2]
        self.executor = root / "zircon_plugins/neural/runtime/src/gpu/graph_executor.rs"
        self.tests = root / "zircon_plugins/neural/runtime/src/tests/gpu_plan.rs"
        self.plan = root / "docs/plans/astra/optimize/01/2026-09-19-neural-dispatch-admission.md"

    def test_dispatch_error_and_optional_device_limits_are_typed(self) -> None:
        source = self.executor.read_text(encoding="utf-8")

        for anchor in (
            "DispatchDimensionOverflow {",
            "code: NnOpCode,\n        batch: u32,\n        channels: u32,",
            "DispatchDimensionLimitExceeded {",
            "groups: [u32; 3],\n        limits: [u32; 3],",
            "InvalidDispatchLimits([u32; 3]),",
            "dispatch_limits: Option<[u32; 3]>",
            "pub fn with_dispatch_limits(mut self, limits: [u32; 3]) -> Self",
            "fn validate_dispatch_limit_values(limits: [u32; 3])",
            "if limits.contains(&0)",
        ):
            self.assertIn(anchor, source)

    def test_plan_checks_limits_before_descriptor_publication(self) -> None:
        source = self.executor.read_text(encoding="utf-8")

        for anchor in (
            "if let Some(limits) = self.dispatch_limits {\n            validate_dispatch_limit_values(limits)?;\n        }",
            "let (workgroup_size, dispatch, parameter_bytes) = dispatch_and_parameters(model, op)?;",
            "validate_dispatch_limits(op.code, &dispatch, self.dispatch_limits)?;",
            "let shader = shader_for(op.code).ok_or(NnGraphBuildError::UnsupportedOp(op.code))?;",
            "ComputePassDescriptor::new(",
            "let RenderGraphComputeDispatchExtent::Fixed(groups) = dispatch else",
            "groups\n        .iter()\n        .zip(limits)",
        ):
            self.assertIn(anchor, source)

        op_dispatch = source.index(
            "let (workgroup_size, dispatch, parameter_bytes) = dispatch_and_parameters(model, op)?;"
        )
        limit_check = source.index("validate_dispatch_limits(op.code, &dispatch, self.dispatch_limits)?;")
        shader = source.index("let shader = shader_for(op.code).ok_or(NnGraphBuildError::UnsupportedOp(op.code))?;")
        descriptor = source.index("ComputePassDescriptor::new(", shader)
        self.assertLess(op_dispatch, limit_check)
        self.assertLess(limit_check, shader)
        self.assertLess(shader, descriptor)

    def test_all_batch_channel_dispatch_paths_share_checked_helper(self) -> None:
        source = self.executor.read_text(encoding="utf-8")
        helper = source[source.index("fn dispatch_batch_channel(") :]

        self.assertIn("batch\n        .checked_mul(channels)", helper)
        self.assertIn("NnGraphBuildError::DispatchDimensionOverflow", helper)
        self.assertNotIn("batch.saturating_mul(channels)", source)
        self.assertEqual(source.count("dispatch_batch_channel(op.code, output.shape[0], output.shape[1])?"), 3)

        pool = source.index("fn pool_dispatch(")
        upsample = source.index("fn upsample_dispatch(")
        conv = source.index("fn conv_dispatch(")
        self.assertLess(pool, upsample)
        self.assertLess(upsample, conv)
        for start, end in ((pool, upsample), (upsample, conv), (conv, source.index("fn dispatch_batch_channel("))):
            self.assertIn("dispatch_batch_channel(op.code, output.shape[0], output.shape[1])?", source[start:end])

    def test_regressions_cover_overflow_for_pool_upsample_conv_and_device_limits(self) -> None:
        tests = self.tests.read_text(encoding="utf-8")
        plan = self.plan.read_text(encoding="utf-8")

        for anchor in (
            "fn nn_graph_executor_rejects_batch_channel_dispatch_overflow()",
            "const OVERFLOW_DIMENSION: u32 = 1 << 16;",
            "NnOpCode::MaxPool2d",
            "NnOpCode::Upsample2d",
            "NnOpCode::Conv2d",
            "Err(NnGraphBuildError::DispatchDimensionOverflow {",
            "fn nn_graph_executor_rejects_dispatch_groups_above_device_limit()",
            ".with_dispatch_limits([3, 3, 3])",
            "Err(NnGraphBuildError::DispatchDimensionLimitExceeded {",
            "groups: [4, 4, 1],",
        ):
            self.assertIn(anchor, tests)

        for anchor in (
            "one checked helper",
            "Conv2d, MaxPool2d/AvgPool2d, and Upsample2d",
            "no saturating product remains",
            "batch == channels == 65_536",
            "device-limit regression",
        ):
            self.assertIn(anchor, plan)


if __name__ == "__main__":
    unittest.main()
