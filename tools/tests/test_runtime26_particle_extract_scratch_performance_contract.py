from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/scene/world/render_particles.rs"


def source() -> str:
    return SOURCE.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def production_extract() -> str:
    text = source().split("pub(super) fn collect_render_particles", 1)[1]
    return text.split("#[derive(Clone, Debug, PartialEq)]", 1)[0]


def performance_test() -> str:
    return source().split(
        "fn optimization_wave_20260825vw_runtime26_particle_extract_scratch_evidence()",
        1,
    )[1]


class Runtime26ParticleExtractScratchPerformanceContract(unittest.TestCase):
    def test_production_extract_reuses_frame_and_stack_storage(self) -> None:
        body = production_extract()

        self.assertIn("let sprite_start = sprites.len();", body)
        self.assertIn("let entity_sprites = &sprites[sprite_start..];", body)
        self.assertIn("std::array::from_fn(|_| None)", body)
        self.assertNotIn("let mut entity_sprites = Vec::new();", body)
        self.assertNotIn("let mut entity_gpu_bounds = Vec::new();", body)
        self.assertNotIn("sprites.extend(entity_sprites);", body)

    def test_release_evidence_uses_the_managed_sampling_contract(self) -> None:
        benchmark = performance_test()

        self.assertIn("const ENTITY_COUNT: usize = 100_000;", benchmark)
        self.assertIn("const WARMUP_PAIRS: usize = 4;", benchmark)
        self.assertIn("const SAMPLE_PAIRS: usize = 21;", benchmark)
        self.assertIn("legacy_particle_extract_scratch_workload", benchmark)
        self.assertIn("optimized_particle_extract_scratch_workload", benchmark)

    def test_release_evidence_compares_real_legacy_and_optimized_storage(self) -> None:
        text = source()

        self.assertIn("let mut entity_sprites = Vec::new();", text)
        self.assertIn("let mut entity_gpu_bounds = Vec::new();", text)
        self.assertIn("let sprite_start = sprites.len();", text)
        self.assertIn("legacy_ns_raw", text)
        self.assertIn("optimized_ns_raw", text)

    def test_release_evidence_gates_latency_and_emits_machine_data(self) -> None:
        benchmark = compact(performance_test())

        self.assertIn("p50_reduction_percent>=15.0", benchmark)
        self.assertIn("p95_reduction_percent>=5.0", benchmark)
        self.assertIn("legacy_transient_buffers=", source())
        self.assertIn("optimized_transient_buffers=0", source())
        self.assertIn("RUNTIME26_PARTICLE_EXTRACT_SCRATCH_BENCH_V1", source())


if __name__ == "__main__":
    unittest.main()
