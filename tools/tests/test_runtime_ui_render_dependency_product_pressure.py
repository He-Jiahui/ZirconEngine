from pathlib import Path
import unittest

from tools.runtime_ui_render_dependency_product_pressure import (
    run,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]
IMAGE = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs"
TEXT_SEGMENT_CACHE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs"
)


class RuntimeUiRenderDependencyProductPressureTests(unittest.TestCase):
    def test_default_model_quantifies_current_residual_and_target_delta_work(self):
        result = run()

        self.assertEqual(result["inputs"]["stable_frame_count"], 4_060)
        self.assertEqual(
            result["current_image_prepare"]["segment_visits"], 262_144
        )
        self.assertEqual(
            result["current_image_prepare"]["texture_dependency_checks"],
            1_048_576,
        )
        self.assertEqual(
            result["current_image_prepare"]["binding_retention_entry_visits"],
            2_097_152,
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
            65_536,
        )
        self.assertEqual(
            result["target_text_persistent_delta"][
                "delta_dependency_entry_visits"
            ],
            1_024,
        )
        self.assertEqual(
            result["target_text_persistent_delta"]["delta_run_entry_visits"],
            256,
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
            baseline["current_image_prepare"]["texture_dependency_checks"] * 2,
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
        self.assertGreater(
            larger["current_text_delta_composition"][
                "delta_dependency_entry_visits"
            ],
            smaller["current_text_delta_composition"][
                "delta_dependency_entry_visits"
            ],
        )

    def test_model_rejects_invalid_state_partition_or_cardinality(self):
        invalid_calls = (
            {"frame_count": 0},
            {"delta_frame_count": -1},
            {"delta_frame_count": 0},
            {"resource_generation_frame_count": 4_097},
            {"changed_segments_per_delta_frame": 0},
            {"segment_count": 2, "changed_segments_per_delta_frame": 3},
            {"image_dependencies_per_segment": 0},
            {"text_dependencies_per_segment": 0},
            {"text_run_spans_per_segment": 0},
            {"binding_cache_entry_count": 0},
        )
        for kwargs in invalid_calls:
            with self.subTest(kwargs=kwargs):
                with self.assertRaises(ValueError):
                    run(**kwargs)

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
        self.assertIn("image_bindings.retain_prepare_epoch(prepare_epoch)", image_prepare)
        self.assertIn("for product in &segment_products", text_prepare)
        self.assertIn(
            "NativeBitmapAtlasFrameDependencyIndex::from_segment_indexes", text_prepare
        )
        self.assertIn("ScreenSpaceUiTextFrameRunIndex::from_segment_run_counts", text_prepare)

    def test_interpretation_excludes_timing_and_marks_typed_full_fallback(self):
        result = run()

        self.assertFalse(result["interpretation"]["timing_claim"])
        self.assertIn("CPU", result["interpretation"]["excluded"])
        self.assertEqual(
            result["typed_full_fallback"]["resource_generation_frame_count"], 4
        )


if __name__ == "__main__":
    unittest.main()
