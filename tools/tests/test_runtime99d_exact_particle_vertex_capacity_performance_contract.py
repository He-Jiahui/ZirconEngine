from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
PARTICLE_VERTICES = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/particle/"
    "build_particle_vertices/build_particle_vertices.rs"
)


class Runtime99DExactParticleVertexCapacityPerformanceContractTests(unittest.TestCase):
    def test_vertex_builder_allocates_once_for_the_exact_renderable_count(self) -> None:
        source = PARTICLE_VERTICES.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]
        normalized = " ".join(production.split())

        self.assertIn(
            "const PARTICLE_VERTICES_PER_SPRITE: usize = 6;",
            production,
        )
        self.assertIsNotNone(
            re.search(
                r"\.filter\(\|sprite\| is_renderable\(sprite\)\)\s*\.count\(\)",
                normalized,
            )
        )
        self.assertIsNotNone(
            re.search(
                r"\.count\(\)\s*\.saturating_mul\(PARTICLE_VERTICES_PER_SPRITE\)",
                normalized,
            )
        )
        self.assertIn("Vec::with_capacity(vertex_capacity)", production)
        self.assertNotIn("let mut vertices = Vec::new();", production)

    def test_capacity_count_and_vertex_build_share_one_eligibility_predicate(self) -> None:
        source = PARTICLE_VERTICES.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]

        self.assertIn(
            "let is_renderable = |sprite: &RenderParticleSpriteSnapshot|",
            production,
        )
        self.assertEqual(production.count("is_renderable(sprite)"), 2)
        self.assertEqual(production.count("camera_layers.intersects"), 1)
        self.assertEqual(production.count("sprite.depth_test != depth_test"), 1)
        self.assertEqual(production.count("sprite.size <= f32::EPSILON"), 1)
        self.assertEqual(production.count("sprite.color.w <= f32::EPSILON"), 1)

    def test_rust_coverage_checks_both_particle_passes_use_exact_capacity(self) -> None:
        source = PARTICLE_VERTICES.read_text(encoding="utf-8")

        self.assertIn(
            "assert_eq!(depth_tested.capacity(), depth_tested.len());",
            source,
        )
        self.assertIn("assert_eq!(overlay.capacity(), overlay.len());", source)


if __name__ == "__main__":
    unittest.main()
