from pathlib import Path
import unittest

from tools.editor_chrome_command_stream_allocation_pressure import run


ROOT = Path(__file__).resolve().parents[2]
CHROME_STREAM = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs"
)
EXTRACTION = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "chrome_command_stream/extraction/entry.rs"
)
STREAM_MODEL = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "chrome_command_stream/stream/model.rs"
)
STREAM_TESTS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "chrome_command_stream/tests/stream_model.rs"
)


class EditorChromeCommandStreamAllocationPerformanceContractTests(unittest.TestCase):
    def test_build_adopts_extracted_commands_without_an_extend_buffer(self) -> None:
        source = CHROME_STREAM.read_text(encoding="utf-8")
        build = source.split("fn build_chrome_command_stream_with_residency", 1)[1]
        build = build.split("#[cfg(test)]", 1)[0]

        self.assertIn("ChromeCommandStream::from_extracted_commands", build)
        self.assertNotIn("ChromeCommandStream::full_rebuild", build)
        self.assertNotIn("ChromeCommandStream::patch", build)
        self.assertNotIn("stream.extend_commands", build)
        self.assertNotIn("stream.push_clip", build)

    def test_extraction_emits_damage_clip_in_the_same_collection(self) -> None:
        source = EXTRACTION.read_text(encoding="utf-8")
        body = source.split("fn extract_chrome_commands", 1)[1]

        clip = body.index("damage_clip_command")
        chain = body.index(".chain(")
        recorded = body.index("recorded_frame.commands", chain)

        self.assertLess(clip, chain)
        self.assertLess(chain, recorded)
        self.assertIn("ChromeCommandKind::Clip", source)

    def test_stream_constructor_and_rust_regression_preserve_vector_identity(self) -> None:
        model = STREAM_MODEL.read_text(encoding="utf-8")
        tests = STREAM_TESTS.read_text(encoding="utf-8")

        constructor = model.split("fn from_extracted_commands", 1)[1]
        constructor = constructor.split("fn full_rebuild", 1)[0]
        self.assertIn("commands,", constructor)
        self.assertNotIn("extend", constructor)
        regression = tests.split(
            "fn extracted_command_vector_is_adopted_without_reallocation", 1
        )[1]
        regression = regression.split("#[test]", 1)[0]
        self.assertIn("commands.as_ptr()", regression)
        self.assertIn("stream.commands().as_ptr()", regression)
        self.assertIn("ChromeCommandKind::Clip", regression)

    def test_image_resource_compaction_state_skips_redundant_command_probe(self) -> None:
        model = STREAM_MODEL.read_text(encoding="utf-8")

        stream_fields = model.split("struct ChromeCommandStream", 1)[1]
        stream_fields = stream_fields.split("impl ChromeCommandStream", 1)[0]
        self.assertIn("image_resources_compacted: bool", stream_fields)

        compaction = model.split("fn compact_image_resources_with_residency", 1)[1]
        compaction = compaction.split("fn into_parts", 1)[0]
        early_return = compaction.index("if self.image_resources_compacted")
        global_probe = compaction.index("self.commands.iter().any")
        publish_clean = compaction.rindex("self.image_resources_compacted = true")
        self.assertLess(early_return, global_probe)
        self.assertGreater(publish_clean, global_probe)

        push = (
            ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/"
            "chrome_command_stream/stream/push/command.rs"
        ).read_text(encoding="utf-8")
        append = push.split("fn push_command", 1)[1]
        self.assertIn("self.image_resources_compacted = false", append)

    def test_pressure_model_counts_only_the_removed_vector_boundary(self) -> None:
        result = run(present_count=4096, commands_per_present=32768)

        self.assertEqual(
            result["retired_extract_then_extend"]["chrome_command_vector_allocations"],
            8192,
        )
        self.assertEqual(
            result["direct_vector_adoption"]["chrome_command_vector_allocations"],
            4096,
        )
        self.assertEqual(result["delta"]["avoided_vector_allocations"], 4096)
        self.assertEqual(result["delta"]["allocation_reduction_ratio"], 2.0)
        self.assertEqual(
            result["delta"]["avoided_inter_vector_command_header_moves"],
            134217728,
        )
        self.assertEqual(
            result["retired_redundant_image_compaction_probe"][
                "command_visits"
            ],
            134217728,
        )
        self.assertEqual(
            result["explicit_compaction_state"]["redundant_command_visits"],
            0,
        )
        self.assertEqual(
            result["delta"]["avoided_redundant_compaction_command_visits"],
            134217728,
        )
        self.assertFalse(result["is_product_timing"])


if __name__ == "__main__":
    unittest.main()
