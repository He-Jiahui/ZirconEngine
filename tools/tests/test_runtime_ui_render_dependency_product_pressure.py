from pathlib import Path
import unittest

from tools.analysis.performance.runtime.runtime_ui_render_dependency_product_pressure import (
    CRITICAL_SOURCE_FILES,
    REFERENCE_SOURCE_FILES,
    build_source_binding,
    run,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]
IMAGE = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs"
TEXT_SEGMENT_CACHE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs"
)
SDF_PREPARED_ATLAS = (
    ROOT / "zircon_runtime/src/text/sdf/font_bake/prepared_atlas.rs"
)
SDF_COMPILED_SEGMENTS = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/segment_product.rs"
)
SDF_VERTEX_BUFFER = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertex_buffer.rs"
)
UNREAL_FONT_CACHE = (
    ROOT
    / "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp"
)
SLINT_ITEM_CACHE = ROOT / "dev/slint/internal/core/item_rendering.rs"


class RuntimeUiRenderDependencyProductPressureTests(unittest.TestCase):
    def test_default_model_quantifies_current_residual_and_target_delta_work(self):
        result = run()

        self.assertEqual(result["inputs"]["stable_frame_count"], 4_060)
        self.assertEqual(result["current_image_prepare"]["segment_visits"], 288)
        self.assertEqual(
            result["current_image_prepare"]["texture_dependency_checks"],
            1_152,
        )
        self.assertEqual(
            result["current_image_prepare"]["binding_retention_entry_visits"],
            0,
        )
        self.assertEqual(
            result["rejected_pre_product_image_prepare"]["segment_visits"],
            262_144,
        )
        self.assertEqual(
            result["current_upstream_ui_texture_prepare"]["command_visits"],
            512,
        )
        self.assertEqual(
            result["current_upstream_ui_texture_prepare"][
                "segment_identity_visits"
            ],
            2_048,
        )
        self.assertEqual(
            result["current_upstream_ui_texture_prepare"][
                "dependency_prepare_visits"
            ],
            1_152,
        )
        self.assertEqual(
            result["target_image_dependency_product"]["segment_visits"], 288
        )
        self.assertEqual(
            result["target_image_dependency_product"][
                "texture_dependency_checks"
            ],
            1_152,
        )
        self.assertEqual(
            result["target_image_dependency_product"][
                "binding_retention_entry_visits"
            ],
            0,
        )
        self.assertEqual(
            result["current_text_delta_composition"][
                "delta_dependency_entry_visits"
            ],
            128,
        )
        self.assertEqual(
            result["current_text_delta_composition"][
                "frame_product_segment_arc_clone_count"
            ],
            256,
        )
        self.assertEqual(
            result["current_text_delta_admission"]["exact_text_input_entry_visits"],
            1_024,
        )
        self.assertEqual(
            result["current_text_delta_admission"]["frame_product_reuse_count"],
            28,
        )
        self.assertEqual(
            result["current_downstream_text_prepare"]["delta_entry_visits"],
            288,
        )
        self.assertEqual(
            result["rejected_pre_journal_downstream_text_prepare"][
                "delta_entry_visits"
            ],
            24_576,
        )
        self.assertEqual(
            result["current_downstream_text_prepare"][
                "same_cardinality_partial_vertex_write_bytes"
            ],
            46_080,
        )
        self.assertEqual(
            result["rejected_pre_journal_downstream_text_prepare"][
                "full_vertex_write_bytes"
            ],
            2_949_120,
        )
        self.assertEqual(
            result["delta"][
                "static_candidate_partial_vertex_write_reduction_ratio"
            ],
            64.0,
        )
        self.assertEqual(
            result["inputs"]["text_delta_partition"],
            {
                "paint_only_frame_count": 1,
                "resident_glyph_frame_count": 2,
                "new_glyph_frame_count": 1,
            },
        )
        self.assertEqual(
            result["target_downstream_text_prepare"]["atlas_glyph_key_visits"],
            96,
        )
        self.assertEqual(
            result["target_downstream_text_prepare"]["atlas_run_entry_visits"],
            24,
        )
        self.assertEqual(
            result["target_downstream_text_prepare"]["cpu_run_entry_visits"],
            32,
        )
        self.assertEqual(
            result["target_downstream_text_prepare"][
                "compiled_glyph_or_vertex_visits"
            ],
            128,
        )
        self.assertEqual(
            result["target_downstream_text_prepare"][
                "bounded_new_glyph_allocation_lookups"
            ],
            32,
        )
        self.assertEqual(
            result["target_downstream_text_prepare"][
                "lower_bake_slot_equality_visits"
            ],
            0,
        )
        self.assertEqual(
            result["delta"][
                "implemented_avoided_downstream_text_prepare_entry_visits"
            ],
            172_032,
        )
        self.assertEqual(
            result["current_text_frame_identity"][
                "stable_segment_identity_checks"
            ],
            0,
        )
        self.assertEqual(
            result["current_text_frame_identity"][
                "tail_delta_segment_identity_checks"
            ],
            0,
        )
        self.assertEqual(
            result["current_text_frame_identity"]["frame_key_checks"], 4_096
        )
        self.assertEqual(
            result["current_text_font_dependency_composition"][
                "segment_visits"
            ],
            288,
        )
        self.assertEqual(
            result["current_text_font_dependency_composition"][
                "dependency_entry_visits"
            ],
            576,
        )
        self.assertEqual(
            result["target_text_persistent_delta"][
                "delta_dependency_entry_visits"
            ],
            128,
        )
        self.assertEqual(
            result["target_text_persistent_delta"]["delta_run_entry_visits"],
            32,
        )
        self.assertEqual(
            result["target_text_persistent_delta"][
                "font_dependency_entry_visits"
            ],
            576,
        )
        self.assertEqual(
            result["target_text_persistent_delta"]["frame_key_checks"],
            4_096,
        )
        self.assertEqual(
            result["target_upstream_texture_dependency_product"]["command_visits"],
            512,
        )
        self.assertEqual(
            result["rejected_pre_product_upstream_ui_texture_prepare"][
                "command_visits"
            ],
            4_194_304,
        )

    def test_target_stable_work_is_independent_of_stable_frame_count(self):
        baseline = run(frame_count=4_096)
        longer = run(frame_count=8_192)

        self.assertEqual(
            baseline["target_image_dependency_product"][
                "texture_dependency_checks"
            ],
            longer["target_image_dependency_product"][
                "texture_dependency_checks"
            ],
        )
        self.assertEqual(
            baseline["target_text_persistent_delta"][
                "delta_dependency_entry_visits"
            ],
            longer["target_text_persistent_delta"][
                "delta_dependency_entry_visits"
            ],
        )
        self.assertEqual(
            longer["current_image_prepare"]["texture_dependency_checks"],
            baseline["current_image_prepare"]["texture_dependency_checks"],
        )
        self.assertEqual(
            longer["current_upstream_ui_texture_prepare"][
                "dependency_prepare_visits"
            ],
            baseline["current_upstream_ui_texture_prepare"][
                "dependency_prepare_visits"
            ],
        )

    def test_target_delta_work_depends_on_changed_not_unrelated_segments(self):
        smaller = run(segment_count=64)
        larger = run(segment_count=128)

        self.assertEqual(
            smaller["target_image_dependency_product"][
                "delta_texture_dependency_checks"
            ],
            larger["target_image_dependency_product"][
                "delta_texture_dependency_checks"
            ],
        )
        self.assertEqual(
            smaller["target_text_persistent_delta"][
                "delta_dependency_entry_visits"
            ],
            larger["target_text_persistent_delta"][
                "delta_dependency_entry_visits"
            ],
        )
        self.assertEqual(
            larger["current_text_delta_composition"][
                "delta_dependency_entry_visits"
            ],
            smaller["current_text_delta_composition"][
                "delta_dependency_entry_visits"
            ],
        )
        self.assertEqual(
            smaller["target_downstream_text_prepare"],
            larger["target_downstream_text_prepare"],
        )

    def test_model_rejects_invalid_state_partition_or_cardinality(self):
        invalid_calls = (
            {"frame_count": 0},
            {"delta_frame_count": -1},
            {"delta_frame_count": 0},
            {"text_affecting_delta_frame_count": -1},
            {"delta_frame_count": 2, "text_affecting_delta_frame_count": 3},
            {"text_paint_only_delta_frame_count": -1},
            {"text_resident_glyph_delta_frame_count": -1},
            {"text_new_glyph_delta_frame_count": -1},
            {
                "text_paint_only_delta_frame_count": 1,
                "text_resident_glyph_delta_frame_count": 1,
            },
            {
                "text_paint_only_delta_frame_count": 1,
                "text_resident_glyph_delta_frame_count": 1,
                "text_new_glyph_delta_frame_count": 1,
            },
            {"resource_generation_frame_count": 4_097},
            {"changed_segments_per_delta_frame": 0},
            {"segment_count": 2, "changed_segments_per_delta_frame": 3},
            {"image_dependencies_per_segment": 0},
            {"text_dependencies_per_segment": 0},
            {"text_run_spans_per_segment": 0},
            {"font_dependencies_per_segment": 0},
            {"ui_commands_per_frame": 0},
            {"binding_cache_entry_count": 0},
            {"compiled_vertices_per_text_dependency": 0},
            {"sdf_vertex_byte_len": 0},
            {"material_uniform_stride": 0},
        )
        for kwargs in invalid_calls:
            with self.subTest(kwargs=kwargs):
                with self.assertRaises(ValueError):
                    run(**kwargs)

        no_text_delta = run(text_affecting_delta_frame_count=0)
        self.assertIsNone(
            no_text_delta["delta"][
                "implemented_text_delta_dependency_entry_reduction_ratio"
            ]
        )

    def test_output_artifacts_are_restricted_to_d_e_or_f(self):
        for path in (
            Path("D:/profiles/render.json"),
            Path("E:/profiles/render.json"),
            Path("F:/profiles/render.json"),
        ):
            with self.subTest(path=path):
                self.assertEqual(validate_output_path(path), path)
        for path in (Path("C:/profiles/render.json"), Path("render.json")):
            with self.subTest(path=path):
                with self.assertRaises(ValueError):
                    validate_output_path(path)

    def test_model_is_bound_to_current_image_and_text_residual_shapes(self):
        image_source = IMAGE.read_text(encoding="utf-8")
        image_prepare = image_source.split("pub(super) fn prepare", 1)[1].split(
            "fn rebuild_segment_geometry", 1
        )[0]
        text_source = TEXT_SEGMENT_CACHE.read_text(encoding="utf-8")
        text_prepare = text_source.split(
            "pub(super) fn prepare_frame_product", 1
        )[1].split("pub(super) fn invalidate_frame_product", 1)[0]

        self.assertIn("render_segments.iter().zip(image_segments.iter_mut())", image_prepare)
        self.assertIn("Self::refresh_segment_dependencies(", image_prepare)
        self.assertIn("prepared.change_journal()", image_prepare)
        self.assertIn("self.frame_generation == Some(prepared.generation())", image_prepare)
        self.assertIn(
            "self.image_bindings.retain_prepare_epoch(prepare_epoch)",
            image_prepare,
        )
        self.assertIn(
            "binding_product: Option<Arc<ScreenSpaceUiImageBindingProduct>>",
            image_source,
        )
        self.assertIn("for &index in journal.changed_segment_indices()", text_prepare)
        self.assertIn("self.frame_aggregate_index.patch(index, &product)", text_prepare)
        self.assertIn("self.native_glyph_dependency_ref_counts", text_prepare)
        self.assertNotIn("NativeBitmapAtlasFrameDependencyIndex", text_prepare)
        self.assertNotIn("ScreenSpaceUiTextFrameRunIndex", text_prepare)
        self.assertIn("refresh_font_dependencies(&mut self, prepared", text_source)
        self.assertIn(
            "self.font_dependency_generation == Some(prepared.generation())",
            text_source,
        )
        self.assertIn("self.frame_matches(prepared.generation()", text_prepare)
        self.assertIn("self.frame_generation = Some(prepared.generation())", text_prepare)

        prepared_atlas_source = SDF_PREPARED_ATLAS.read_text(encoding="utf-8")
        self.assertIn("self.slots.as_slice() != slots", prepared_atlas_source)
        unreal_font_cache_source = UNREAL_FONT_CACHE.read_text(encoding="utf-8")
        self.assertIn("ShapedGlyphToAtlasData.Find(GlyphKey)", unreal_font_cache_source)
        self.assertIn("SdfGlyphToAtlasData.Find(GlyphKey)", unreal_font_cache_source)
        self.assertIn("SdfGlyphToAtlasData.Add(GlyphKey", unreal_font_cache_source)
        slint_item_cache_source = SLINT_ITEM_CACHE.read_text(encoding="utf-8")
        self.assertIn("pub struct ItemCache<T>", slint_item_cache_source)
        self.assertIn("evaluate_if_dirty(update_fn)", slint_item_cache_source)
        compiled_source = SDF_COMPILED_SEGMENTS.read_text(encoding="utf-8")
        local_patch = compiled_source.split("fn patch_retained_frame", 1)[1].split(
            "fn rebuild", 1
        )[0]
        self.assertIn("journal.changed_segment_indices()", local_patch)
        self.assertNotIn("for segment in frame.segment_products()", local_patch)
        self.assertIn("fn apply_replacements", compiled_source)
        vertex_buffer_source = SDF_VERTEX_BUFFER.read_text(encoding="utf-8")
        self.assertIn("write_sdf_vertex_buffer_ranges", vertex_buffer_source)
        self.assertIn("coalesced_vertex_ranges", vertex_buffer_source)

    def test_source_binding_covers_current_authorities_and_reference_lifetimes(self):
        binding = build_source_binding(ROOT)

        self.assertEqual(
            {entry["relative_path"] for entry in binding["critical_source_files"]},
            set(CRITICAL_SOURCE_FILES),
        )
        self.assertEqual(
            {entry["relative_path"] for entry in binding["reference_source_files"]},
            set(REFERENCE_SOURCE_FILES),
        )
        self.assertEqual(len(binding["source_set_sha256"]), 64)
        self.assertTrue(all(entry["sha256"] for entry in binding["critical_source_files"]))
        self.assertTrue(all(entry["sha256"] for entry in binding["reference_source_files"]))

    def test_interpretation_excludes_timing_and_marks_typed_full_fallback(self):
        result = run()

        self.assertFalse(result["interpretation"]["timing_claim"])
        self.assertIn("CPU", result["interpretation"]["excluded"])
        self.assertEqual(
            result["typed_full_fallback"]["resource_generation_frame_count"], 4
        )


if __name__ == "__main__":
    unittest.main()
