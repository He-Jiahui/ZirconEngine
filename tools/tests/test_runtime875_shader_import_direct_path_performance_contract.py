"""Source and cost-model contract for direct shader import path assembly."""

import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/core/framework/render/shader/module_import.rs"
LOWER = ROOT / "zircon_runtime/src/core/framework/render/shader/module_import/tests/direct_path_tests.rs"


class Runtime875ShaderImportDirectPathTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.owner = cls.source.split("pub fn derive_shader_import_path(", 1)[1].split(
            "fn wgsl_include_path_from_line(", 1
        )[0]
        cls.borrowed = cls.source.split("fn shader_module_path_segments(", 1)[1].split(
            "fn strip_shader_asset_extension(", 1
        )[0]
        cls.segment = cls.source.split("fn shader_module_segment(", 1)[1].split(
            "fn terminal_directory_was_folded(", 1
        )[0]

    def test_final_path_is_written_directly_into_input_bounded_output(self) -> None:
        self.assertIn("let mut import_path = String::with_capacity(", self.owner)
        self.assertIn('import_path.push_str("::")', self.owner)
        self.assertIn("shader_module_segment(&mut import_path,", self.owner)
        self.assertIn("import_path,", self.owner)

    def test_module_selection_borrows_normalized_path(self) -> None:
        self.assertIn("Result<(&[String], bool), ShaderImportPathDerivationError>", self.borrowed)
        self.assertIn("&normalized_path[root_index + 1..]", self.borrowed)
        self.assertNotIn(".to_vec()", self.borrowed)
        self.assertNotIn(".to_string()", self.borrowed)

    def test_segment_normalization_appends_without_child_strings(self) -> None:
        self.assertIn("output: &mut String", self.segment)
        self.assertIn("output.push(", self.segment)
        self.assertNotIn("String::new()", self.segment)
        self.assertNotIn("output.insert(", self.segment)

    def test_no_staging_join_or_collection_in_success_path(self) -> None:
        self.assertNotIn("segments.join(", self.owner)
        self.assertNotIn("Vec::with_capacity", self.owner)
        self.assertNotIn("segments.push(", self.owner)

    def test_lower_parity_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod direct_path_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("runtime875_shader_import_direct_path_preserves_derivations", lower)
        self.assertIn("RUNTIME875_SHADER_IMPORT_DIRECT_PATH_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_module_path_temporary_allocation_model(self) -> None:
        paths, segments = 4_096, 32
        old_clones = old_module_slots = old_normalized_segments = paths * segments
        old_staging_slots = paths * (segments + 1)
        old_joined_strings = paths
        self.assertEqual(
            (old_clones, old_module_slots, old_normalized_segments, old_staging_slots, old_joined_strings),
            (131_072, 131_072, 131_072, 135_168, 4_096),
        )
        self.assertEqual((0, 0, 0, 0, 0), (0,) * 5)


if __name__ == "__main__":
    unittest.main()
