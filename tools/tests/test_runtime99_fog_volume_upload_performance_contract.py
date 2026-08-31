from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MEDIA_INJECT = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject.rs"
)
EXECUTOR = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/media_inject.rs"
)
PERFORMANCE_TESTS = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject/performance_tests.rs"
)


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime99FogVolumeUploadPerformanceContractTests(unittest.TestCase):
    def test_executor_passes_borrowed_volumes_and_layers_to_encoder(self) -> None:
        source = EXECUTOR.read_text(encoding="utf-8")

        self.assertNotIn("fog_volumes_for_layers", source)
        self.assertIn("local_volumes: &advanced.fog_volumes", source)
        self.assertIn("FroxelMediaInjectPipeline::prepare_for_layers(", source)
        self.assertIn(".encode_prepared(", source)
        self.assertIn("outcome.uploaded_bytes", source)

    def test_gpu_volume_collection_fuses_filter_and_conversion(self) -> None:
        source = MEDIA_INJECT.read_text(encoding="utf-8")
        collection = function_body(
            source,
            "fn collect_gpu_volumes(",
            "#[repr(C)]",
        )

        self.assertIn("if !include_local_volumes {", collection)
        self.assertIn("return Vec::new();", collection)
        self.assertIn("Vec::with_capacity(local_volumes.len())", collection)
        self.assertIn("volume.layer_mask.intersects(render_layers)", collection)
        self.assertIn(".map(GpuFogVolume::from)", collection)
        self.assertNotIn(".cloned()", collection)

    def test_release_benchmark_covers_enabled_and_disabled_paths(self) -> None:
        self.assertTrue(PERFORMANCE_TESTS.is_file())
        benchmark = PERFORMANCE_TESTS.read_text(encoding="utf-8")

        self.assertIn("RUNTIME99_FOG_VOLUME_FILTER_PERF", benchmark)
        self.assertIn("legacy_volume_clones=32768", benchmark)
        self.assertIn("optimized_volume_clones=0", benchmark)
        self.assertIn("RUNTIME99_FOG_VOLUME_DISABLED_PERF", benchmark)
        self.assertIn("legacy_volume_visits=65536", benchmark)
        self.assertIn("optimized_volume_visits=0", benchmark)


if __name__ == "__main__":
    unittest.main()
