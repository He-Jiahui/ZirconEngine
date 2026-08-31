from pathlib import Path
import unittest

from tools.runtime_ui_native_atlas_retained_submission_pressure import (
    _reject_c_drive,
    run,
)


ROOT = Path(__file__).resolve().parents[2]
SOURCE_CACHE = ROOT / "zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs"
NATIVE_BITMAP_ATLAS = ROOT / "zircon_runtime/src/text/native_bitmap_atlas.rs"
NATIVE_BITMAP_FRAME = ROOT / "zircon_runtime/src/text/native_bitmap_atlas/frame.rs"
TEXT_SEGMENT_CACHE = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs"
)
NATIVE_DEPENDENCY_INDEX = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache/"
    "native_dependency_index.rs"
)


class RuntimeUiNativeAtlasRetainedSubmissionPerformanceContract(unittest.TestCase):
    def test_model_separates_readiness_lookup_from_ordered_geometry_work(self) -> None:
        result = run()

        self.assertEqual(result["inputs"]["glyph_instances_per_frame"], 24_576)
        self.assertEqual(
            result["current_two_pass_native_prepare"][
                "readiness_discovery_glyph_visits"
            ],
            100_663_296,
        )
        self.assertEqual(
            result["current_two_pass_native_prepare"][
                "unique_readiness_cache_lookups"
            ],
            8_388_608,
        )
        self.assertEqual(
            result["current_two_pass_native_prepare"][
                "ordered_geometry_glyph_visits"
            ],
            100_663_296,
        )
        self.assertEqual(
            result["retained_native_submission"]["segment_geometry_rebuilds"],
            288,
        )
        self.assertEqual(
            result["retained_native_submission"][
                "segment_geometry_rebuild_glyph_visits"
            ],
            110_592,
        )
        self.assertEqual(
            result["retained_native_submission"]["active_readiness_checks"],
            8_388_608,
        )
        self.assertEqual(
            result["retained_native_submission"][
                "readiness_patch_instance_visits"
            ],
            3_072,
        )
        self.assertEqual(
            result["retained_index_shape"]["reverse_instance_index_entries"],
            24_576,
        )
        self.assertGreater(result["delta"]["modeled_work_reduction_ratio"], 24.0)
        self.assertIn("actual CPU", result["interpretation"]["excluded"])
        self.assertIn("readiness update", result["interpretation"]["scope"])

    def test_model_rejects_invalid_topology_and_change_counts(self) -> None:
        with self.assertRaisesRegex(ValueError, "segment_count"):
            run(segment_count=0)
        with self.assertRaisesRegex(ValueError, "divisible"):
            run(segment_count=63)
        with self.assertRaisesRegex(ValueError, "unique_glyph_dependency_count"):
            run(unique_glyph_dependency_count=24_577)
        with self.assertRaisesRegex(ValueError, "readiness_key_change_count"):
            run(readiness_key_change_count=-1)

    def test_artifact_writer_rejects_c_drive(self) -> None:
        with self.assertRaisesRegex(ValueError, "C drive"):
            _reject_c_drive(Path("C:/zircon-profiles/native-atlas.json"))

    def test_source_cache_publishes_typed_readiness_generation(self) -> None:
        source = SOURCE_CACHE.read_text(encoding="utf-8")

        self.assertIn("NativeBitmapAtlasReadinessGeneration", source)
        self.assertIn("readiness_generation:", source)
        self.assertIn("pub(crate) fn readiness_generation", source)
        self.assertIn("fn advance_readiness_generation", source)
        self.assertIn("NativeBitmapAtlasReadinessChangeReceipt", source)
        self.assertIn("pending_readiness_changed_keys:", source)
        self.assertIn("pub(crate) fn take_readiness_changes", source)
        self.assertIn("record_readiness_change", source)
        self.assertIn("vertical_subpixel_bin: 0", source)
        self.assertIn("vertical_subpixel_bin: 3", source)

    def test_completed_native_frame_captures_readiness_changes_after_retry_invalidation(
        self,
    ) -> None:
        module_source = NATIVE_BITMAP_ATLAS.read_text(encoding="utf-8")
        frame_source = NATIVE_BITMAP_FRAME.read_text(encoding="utf-8")

        self.assertIn("NativeBitmapAtlasReadinessGeneration", module_source)
        self.assertIn("readiness_changes: NativeBitmapAtlasReadinessChangeReceipt", frame_source)
        self.assertIn("pub(crate) fn readiness_generation", frame_source)
        retry_invalidation = frame_source.rfind(
            "source_cache.invalidate_raster_keys(invalidated_raster_keys)"
        )
        readiness_capture = frame_source.index(
            "let readiness_changes = source_cache.take_readiness_changes();"
        )
        self.assertGreater(retry_invalidation, 0)
        self.assertGreater(readiness_capture, retry_invalidation)
        self.assertIn("readiness_changes,", frame_source[readiness_capture:])
        self.assertIn(
            '"ui_text.native_raster_plan.readiness_generation"',
            frame_source,
        )

    def test_segment_products_publish_compressed_native_dependency_indexes(self) -> None:
        segment_source = TEXT_SEGMENT_CACHE.read_text(encoding="utf-8")
        index_source = NATIVE_DEPENDENCY_INDEX.read_text(encoding="utf-8")

        self.assertIn("mod native_dependency_index;", segment_source)
        self.assertIn("NativeBitmapAtlasSegmentDependencyIndex", segment_source)
        self.assertIn("NativeBitmapAtlasFrameDependencyIndex", segment_source)
        self.assertIn("native_glyph_dependencies:", segment_source)
        self.assertIn("from_glyph_runs", segment_source)
        self.assertNotIn(
            "active_glyph_dependencies.extend(segment_products.iter().flat_map",
            segment_source,
        )
        self.assertIn("spans_by_key: HashMap<GlyphRasterKey, Range<usize>>", index_source)
        self.assertIn("locations: Arc<[NativeBitmapAtlasGlyphLocation]>", index_source)
        self.assertIn("segment_indices: Arc<[usize]>", index_source)
        self.assertIn("pub(super) fn locations_for", index_source)
        self.assertIn("pub(super) fn segment_indices_for", index_source)
        self.assertIn("native_reverse_instance_entry_count", segment_source)
        self.assertIn("native_reverse_segment_entry_count", segment_source)


if __name__ == "__main__":
    unittest.main()
