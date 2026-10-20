import unittest
import sys
from pathlib import Path

_AUDIT_SCRIPTS = (
    Path(__file__).resolve().parents[2]
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(_AUDIT_SCRIPTS))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import (
    HistoricalArchiveReader,
)

_HISTORICAL_IDENTITIES = {
    'docs/plans/_archive/astra/optimize/01/2026-09-12-neural-graph-admission.md': (
        3182,
        '8caf253981f01a5f8372523d0213629fd67f2c833713c4f209c007f8d78f3fa1',
    ),
    'docs/plans/_archive/astra/optimize/01/2026-09-12-neural-onnx-dimension-admission.md': (
        2296,
        '3885e2a51bc318ccb51d6e6bd2b73174bfc315433c14a49d443972da5c3ba8b5',
    ),
}


class NeuralGraphOnnxAdmissionContractTests(unittest.TestCase):
    """Static guard for the Neural model and ONNX admission boundaries."""

    def setUp(self) -> None:
        root = Path(__file__).resolve().parents[2]
        self.validator = root / "zircon_plugins/neural/runtime/src/model/validate.rs"
        self.model_tests = root / "zircon_plugins/neural/runtime/src/tests/model_asset.rs"
        self.executor = root / "zircon_plugins/neural/runtime/src/gpu/graph_executor.rs"
        self.gpu_tests = root / "zircon_plugins/neural/runtime/src/tests/gpu_plan.rs"
        self.reader = root / "zircon_plugins/neural/editor/src/onnx/reader.rs"
        self.plan = 'docs/plans/_archive/astra/optimize/01/2026-09-12-neural-graph-admission.md'
        self.reader_plan = 'docs/plans/_archive/astra/optimize/01/2026-09-12-neural-onnx-dimension-admission.md'

    def test_model_validation_admits_sources_and_enforces_topological_producers(self) -> None:
        source = self.validator.read_text(encoding="utf-8")

        for anchor in (
            "TensorUsedBeforeProduced {",
            "InvalidOutputTensorKind {",
            "DuplicateTensorProducer {",
            "let f16_weights = self.contains_f16_weights();",
            "if tensor.element_count().is_none()",
            "let required_bytes = tensor",
            "count.checked_mul(element_size)",
            "tensor.weight_offset.checked_add(required_bytes)",
            "self.validate_graph_topology()",
            "fn validate_graph_topology(&self)",
            "let mut producers = vec![None; self.tensors.len()];",
            "NnTensorKind::Input | NnTensorKind::Weight => {}",
            "producers[tensor_index].is_none()",
            "producers[tensor_index] = Some(op_index);",
        ):
            self.assertIn(anchor, source)

        topology_call = source.index("self.validate_graph_topology()")
        op_checks = source.index("for (op_index, op) in self.ops.iter().enumerate()")
        self.assertLess(op_checks, topology_call)

        topology = source[source.index("fn validate_graph_topology(&self)") :]
        self.assertIn("if self.ops.is_empty()", topology)
        self.assertIn("return Ok(())", topology)
        self.assertLess(
            topology.index("TensorUsedBeforeProduced"),
            topology.index("DuplicateTensorProducer"),
        )

    def test_gpu_reshape_checks_element_count_before_recording_an_alias(self) -> None:
        source = self.executor.read_text(encoding="utf-8")
        start = source.index("fn fold_reshape(")
        end = source.index("fn only_output(", start)
        body = source[start:end]

        for anchor in (
            "let source = op.inputs[0];",
            "let output = op.outputs[0];",
            "let source_descriptor = tensor_descriptor(model, source)?;",
            "let output_descriptor = tensor_descriptor(model, output)?;",
            "source_descriptor.element_count() != output_descriptor.element_count()",
            "NnGraphBuildError::InvalidShape(NnOpCode::Reshape)",
            "resource_for(source, model, aliases, io)?;",
            "if !aliases.alias(output, source)",
        ):
            self.assertIn(anchor, body)

        mismatch = body.index("source_descriptor.element_count() != output_descriptor.element_count()")
        resource_lookup = body.index("resource_for(source, model, aliases, io)?;")
        alias = body.index("if !aliases.alias(output, source)")
        self.assertLess(mismatch, resource_lookup)
        self.assertLess(resource_lookup, alias)

    def test_onnx_reader_uses_checked_signed_dimensions_and_strict_utf8(self) -> None:
        source = self.reader.read_text(encoding="utf-8")

        for anchor in (
            "InvalidDimension { value: i64 },",
            "InvalidUtf8String,",
            "fn read_packed_i64(bytes: &[u8], output: &mut Vec<u32>)",
            "output.push(read_dimension(&mut reader)?);",
            "fn read_dimension(reader: &mut ProtoReader<'_>)",
            "let value = i64::from_le_bytes(encoded.to_le_bytes());",
            "u32::try_from(value).map_err(|_| OnnxReadError::InvalidDimension { value })",
            "fn read_string(reader: &mut ProtoReader<'_>)",
            "std::str::from_utf8(reader.read_bytes()?)",
            ".map_err(|_| OnnxReadError::InvalidUtf8String)",
        ):
            self.assertIn(anchor, source)

        self.assertNotIn("from_utf8_lossy", source)
        self.assertGreaterEqual(source.count("read_dimension("), 4)
        self.assertLess(
            source.index("let value = i64::from_le_bytes(encoded.to_le_bytes());"),
            source.index("u32::try_from(value).map_err"),
        )

    def test_regressions_cover_topology_budgets_reshape_and_onnx_identity(self) -> None:
        model_tests = self.model_tests.read_text(encoding="utf-8")
        gpu_tests = self.gpu_tests.read_text(encoding="utf-8")
        reader_parent = self.reader.read_text(encoding="utf-8")
        reader = (self.reader.parent / "tests/reader.rs").read_text(encoding="utf-8")
        self.assertIn(
            '#[cfg(test)]\n#[path = "tests/reader.rs"]\nmod tests;',
            reader_parent,
        )
        for owner_path, mount in (
            ('zircon_plugins/neural/editor/src/lib.rs', '#[path = "onnx/mod.rs"]\npub mod onnx;'),
            ('zircon_plugins/neural/editor/src/onnx/mod.rs', '#[path = "reader.rs"]\nmod reader;'),
        ):
            self.assertIn(mount, (Path(__file__).resolve().parents[2] / owner_path).read_text(encoding="utf-8"))
        history = HistoricalArchiveReader(identities=_HISTORICAL_IDENTITIES)
        plan = history.read(self.plan)
        self.assertIsNotNone(plan, history.records)
        reader_plan = history.read(self.reader_plan)
        self.assertIsNotNone(reader_plan, history.records)

        for test_name in (
            "fn nn_model_asset_rejects_tensor_use_before_its_producer()",
            "fn nn_model_asset_rejects_duplicate_tensor_producers()",
            "fn nn_model_asset_rejects_writing_input_or_weight_tensors()",
            "fn nn_model_asset_accepts_weight_only_resource_models()",
            "fn nn_model_asset_rejects_declared_op_count_beyond_table_capacity_before_allocation()",
            "fn nn_model_asset_rejects_weight_blob_over_budget_before_copy()",
        ):
            self.assertIn(test_name, model_tests)

        for anchor in (
            "fn nn_graph_executor_rejects_reshape_that_changes_element_count()",
            "Err(NnGraphBuildError::InvalidShape(NnOpCode::Reshape))",
        ):
            self.assertIn(anchor, gpu_tests)

        for test_name in (
            "fn rejects_negative_packed_tensor_dimension()",
            "fn rejects_negative_unpacked_tensor_dimension()",
            "fn rejects_dimension_larger_than_u32()",
            "fn rejects_negative_value_info_dimension()",
            "fn rejects_invalid_utf8_tensor_name()",
            "fn rejects_invalid_utf8_attribute_string()",
        ):
            self.assertIn(test_name, reader)
            self.assertRegex(reader, r"#\[test\]\s*" + test_name.replace("()", r"\(\)"))

        for anchor in (
            "one producer table",
            "model_asset.rs` adds regressions",
        ):
            self.assertIn(anchor, plan)
        for anchor in (
            "applies a checked\n`u32::try_from`",
            "InvalidUtf8String",
            "negative packed and unpacked TensorProto\ndimensions",
            "oversized dimensions",
        ):
            self.assertIn(anchor, reader_plan)


if __name__ == "__main__":
    unittest.main()
