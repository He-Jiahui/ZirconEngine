from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
VERTEX_SOURCE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs"
)
BATCH_SOURCE = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs"
)
RENDERER_SOURCE = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs"
)
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_runtime/99e/2026-08-27-streaming-sprite-batch-projection.md"
)


def production_source(path: Path) -> str:
    return path.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]


class Runtime99EStreamingSpriteBatchProjectionContract(unittest.TestCase):
    def test_stage_projection_streams_phase_indices(self) -> None:
        source = production_source(VERTEX_SOURCE)
        start = source.index("pub(super) fn visit_stage_sprites(")
        end = source.index("fn sprite_vertex_source", start)
        body = source[start:end]

        self.assertIn("let first_phase_item = phase_items.next();", body)
        self.assertIn("std::iter::once(first_phase_item).chain(phase_items)", body)
        self.assertNotIn("collect::<Vec", body)

    def test_slice_projection_appends_without_intermediate_vectors(self) -> None:
        source = production_source(VERTEX_SOURCE)
        start = source.index("pub(super) fn append_sprite_image_vertices(")
        end = source.index("pub(crate) fn build_sprite_vertices(", start)
        body = source[start:end]

        self.assertIn("visit_sprite_image_slices(", body)
        self.assertIn("append_sprite_quad_vertices(", body)
        self.assertNotIn("Vec::", body)
        self.assertNotIn("fn sprite_image_slices(", source)
        self.assertIn("fn emit_sprite_image_slice(", source)

    def test_2d_batch_projection_writes_final_batch_storage_directly(self) -> None:
        source = production_source(BATCH_SOURCE)
        start = source.index("fn prepare_sprite_draw_batches(")
        end = source.index("pub(crate) fn prepare_sprite_queue_stats(", start)
        body = source[start:end]
        renderer = production_source(RENDERER_SOURCE)

        self.assertIn("visit_stage_sprites(frame, stage", body)
        self.assertIn("append_sprite_image_vertices(sprite, size", body)
        self.assertNotIn("batch_sprite_draw_items(", body)
        self.assertNotIn("Vec<(usize, Vec<SpriteVertex>)>", body)
        self.assertIn("prepare_sprite_draw_batches(frame, stage)", renderer)
        self.assertNotIn("build_sprite_vertices(frame, stage)", renderer)

    def test_behavior_and_performance_evidence_are_recorded(self) -> None:
        tests = (
            ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs"
        ).read_text(encoding="utf-8")
        record = RECORD.read_text(encoding="utf-8")

        self.assertIn(
            "prepared_sprite_batches_project_adjacent_vertices_directly_into_final_storage",
            tests,
        )
        self.assertIn("99.98%", record)
        self.assertIn("40.00%", record)
        self.assertIn("60.45%", record)


if __name__ == "__main__":
    unittest.main()
