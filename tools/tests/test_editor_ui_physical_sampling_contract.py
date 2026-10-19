# 核对原生界面尺寸变化采用物理采样契约，并在计划中保留双线性要求。
import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/zircon_editor/editor_ui/12/2026-08-15-m6-device-pixel-aa-and-local-supersampling.md': 'docs/plans/zircon_editor/editor_ui/12/2026-08-15-m6-device-pixel-aa-and-local-supersampling.md'}
_HISTORY_IDENTITIES = {'docs/plans/zircon_editor/editor_ui/12/2026-08-15-m6-device-pixel-aa-and-local-supersampling.md': (167735, '692349de857d6306c1e9311fe57bb02c1a41f76d4a2bc4d33cb9cbbd7e07498a')}
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



REPO_ROOT = Path(__file__).resolve().parents[2]
COPY = REPO_ROOT / (
    "zircon_editor/src/ui/retained/contract/presenter/"
    "softbuffer/surface_io/copy.rs"
)
PLAN = REPO_ROOT / (
    "docs/plans/zircon_editor/editor_ui/12/"
    "2026-08-15-m6-device-pixel-aa-and-local-supersampling.md"
)


class EditorUiPhysicalSamplingContractTests(unittest.TestCase):
    def test_native_resize_does_not_use_nearest_neighbor_source_selection(self):
        source = _read_contract_input(COPY, encoding="utf-8")

        self.assertNotIn("EndpointNearestAxis", source)
        self.assertNotIn("Vec<AxisSample>", source)
        self.assertIn("axis_sample", source)
        self.assertIn("bilinear_channel", source)
        self.assertIn("srgb_byte_to_linear", source)
        self.assertIn("linear_to_srgb_byte", source)
        self.assertNotIn("powf", source)
        self.assertIn("copy_rgba_to_softbuffer(source, buffer, None, target_size)", source)

    def test_m6_plan_records_bilinear_resize_sampling(self):
        plan = _read_contract_input(PLAN, encoding="utf-8")

        self.assertIn("physical-pixel-center bilinear sampling", plan)
        self.assertIn("linear-light color interpolation", plan)
        self.assertNotIn("software scaled images use nearest-neighbor", plan)


if __name__ == "__main__":
    unittest.main()
