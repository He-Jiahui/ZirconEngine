from pathlib import Path
import unittest

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/optimize/zircon_runtime/99e/2026-08-27-streaming-sprite-batch-projection.md': 'docs/plans/optimize/zircon_runtime/99e/2026-08-27-streaming-sprite-batch-projection.md'}
_HISTORY_IDENTITIES = {'docs/plans/optimize/zircon_runtime/99e/2026-08-27-streaming-sprite-batch-projection.md': (3527, '3d2be2274ace46005f02ae249f2fb22410f38b5e8fd5f10c819097b2290f0556')}
_CURRENT_POLICY_INPUTS = ()

def _read_contract_input(path, *args, **kwargs):
    candidate = _HistoryPath(path)
    try:
        relative = candidate.absolute().relative_to(_HISTORY_REPO_ROOT).as_posix()
    except ValueError:
        relative = None
    key = _HISTORY_PATHS.get(relative)
    if key is None:
        if relative and relative.startswith('docs/plans/') and relative not in _CURRENT_POLICY_INPUTS:
            raise AssertionError('Unregistered current/history contract input: ' + relative)
        return candidate.read_text(*args, **kwargs)
    if args or kwargs.get('encoding', 'utf-8').lower() not in ('utf-8', 'utf8') or kwargs.get('errors') not in (None, 'strict') or set(kwargs) - {'encoding', 'errors'}:
        raise AssertionError('Historical evidence requires strict UTF-8')
    reader = _ContractHistoryReader(identities=_HISTORY_IDENTITIES)
    text = reader.read(key)
    if text is None:
        raise AssertionError(reader.records)
    return text

# 精灵批次从阶段索引流式写入最终二维批次存储；切片投影直接追加，避免阶段间向量。

ROOT = Path(__file__).resolve().parents[2]
VERTEX_SOURCE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/renderer/sprite/build_sprite_vertices.rs"
)
BATCH_SOURCE = (
    ROOT / "zircon_runtime/src/graphics/scene/renderer/sprite/prepared_batches.rs"
)
RENDERER_SOURCE = (
    ROOT / "zircon_runtime/src/graphics/scene/renderer/sprite/sprite_renderer.rs"
)
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_runtime/99e/2026-08-27-streaming-sprite-batch-projection.md"
)


def production_source(path: Path) -> str:
    return _read_contract_input(path, encoding="utf-8").split("#[cfg(test)]", 1)[0]


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
        tests = _read_contract_input(ROOT / "zircon_runtime/src/graphics/scene/renderer/sprite/tests/cases.rs", encoding="utf-8")
        record = _read_contract_input(RECORD, encoding="utf-8")

        self.assertIn(
            "prepared_sprite_batches_project_adjacent_vertices_directly_into_final_storage",
            tests,
        )
        self.assertIn("99.98%", record)
        self.assertIn("40.00%", record)
        self.assertIn("60.45%", record)


if __name__ == "__main__":
    unittest.main()
