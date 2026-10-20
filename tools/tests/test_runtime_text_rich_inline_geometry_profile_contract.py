import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md': 'versions/reconciliation-515/d9589b2ad2f6a3bd60b8176fe14baeb6ecf71abd960f9c7890148eaeb846a014/docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md'}
_HISTORY_IDENTITIES = {'versions/reconciliation-515/d9589b2ad2f6a3bd60b8176fe14baeb6ecf71abd960f9c7890148eaeb846a014/docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md': (20861, 'd9589b2ad2f6a3bd60b8176fe14baeb6ecf71abd960f9c7890148eaeb846a014')}
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



ROOT = Path(__file__).resolve().parents[2]
RICH_RENDER = (
    ROOT
    / "zircon_runtime/src/graphics/scene/renderer/ui/render/rich_text.rs"
)
RICH_INLINE_TESTS = (
    ROOT
    / "zircon_runtime/src/graphics/scene/renderer/ui/render/tests/rich_inline.rs"
)
RICH_INLINE_PROFILE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/ui/render/tests/rich_inline_profile.rs"
)
PROFILE_PLAN = (
    ROOT
    / "docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md"
)

# 读取富文本内联几何剖析配置，确认低基数计数器只在剖析构建中出现。

def _mounted_test_source(parent: Path, relative: str, module: str, cfg: str = "test") -> str:
    """Read the actual explicit Rust test mount; missing or changed mounting fails closed."""
    source = _read_contract_input(parent, encoding="utf-8")
    declaration = f'#[cfg({cfg})]\n#[path = "{relative}"]\nmod {module};'
    if declaration not in source:
        raise AssertionError(f"Rust test mount changed: {parent}::{module}")
    child = (parent.parent / relative).resolve()
    if not child.is_relative_to(_HISTORY_REPO_ROOT):
        raise AssertionError(f"Rust test mount escapes the checkout: {child}")
    return _read_contract_input(child, encoding="utf-8")



class RuntimeTextRichInlineGeometryProfileContractTests(unittest.TestCase):
    def test_profile_uses_fixed_low_cardinality_counter_names(self) -> None:
        source = _read_contract_input(RICH_RENDER, encoding="utf-8")
        names = [
            "rich_inline_run_count",
            "rich_inline_line_probe_count",
            "rich_inline_line_run_probe_count",
            "rich_inline_prefix_grapheme_count",
            "rich_inline_prefix_advance_count",
            "rich_inline_paint_frame_match_count",
            "rich_inline_paint_frame_mismatch_count",
        ]

        for name in names:
            self.assertEqual(source.count(f'"{name}"'), 1, name)
        self.assertNotIn("format!(", source[source.index("fn publish(&self)") :])

    def test_profile_fields_are_absent_from_ordinary_builds(self) -> None:
        source = _read_contract_input(RICH_RENDER, encoding="utf-8")
        profile_start = source.index("struct RichInlineGeometryProfile")
        profile_end = source.index("pub(super) struct RichTextRunPresentation", profile_start)
        profile = source[profile_start:profile_end]

        self.assertGreaterEqual(profile.count('#[cfg(feature = "profiling")]'), 9)
        self.assertIn('#[cfg(not(feature = "profiling"))]', profile)
        self.assertNotIn("String", profile)
        self.assertNotIn("Vec<", profile)

    def test_real_layout_regression_checks_work_and_frame_agreement(self) -> None:
        tests = _read_contract_input(RICH_INLINE_TESTS, encoding="utf-8")

        self.assertIn(
            "rich_inline_geometry_profile_reports_existing_prefix_reconstruction_work",
            tests,
        )
        self.assertIn("expected_line_run_probes", tests)
        self.assertIn("expected_prefix_graphemes", tests)
        self.assertIn('counter("rich_inline_paint_frame_match_count")', tests)
        self.assertIn('counter("rich_inline_paint_frame_mismatch_count"), 0', tests)

    def test_release_profile_harness_covers_direction_and_line_search_lanes(self) -> None:
        tests = _read_contract_input(RICH_INLINE_TESTS, encoding="utf-8")
        profile = _mounted_test_source(RICH_INLINE_TESTS, "../../../../ui/render/tests/rich_inline_profile.rs", "rich_inline_profile", 'all(feature = "profiling", windows)')

        self.assertIn('path = "../../../../ui/render/tests/rich_inline_profile.rs"', tests)
        self.assertIn("mod rich_inline_profile;", tests)
        self.assertIn("const SAMPLE_COUNT: usize = 31;", profile)
        self.assertIn("DenseLtr", profile)
        self.assertIn("DenseRtl", profile)
        self.assertIn("DenseVerticalRl", profile)
        self.assertIn("WrappedLines", profile)
        self.assertIn("samples_ns={samples_ns:?}", profile)
        self.assertIn("rss_delta_bytes={rss_delta_bytes:?}", profile)

    def test_release_profile_harness_reports_all_geometry_work_counters(self) -> None:
        profile = _mounted_test_source(RICH_INLINE_TESTS, "../../../../ui/render/tests/rich_inline_profile.rs", "rich_inline_profile", 'all(feature = "profiling", windows)')
        names = [
            "rich_inline_run_count",
            "rich_inline_line_probe_count",
            "rich_inline_line_run_probe_count",
            "rich_inline_prefix_grapheme_count",
            "rich_inline_prefix_advance_count",
            "rich_inline_paint_frame_match_count",
            "rich_inline_paint_frame_mismatch_count",
        ]

        for name in names:
            self.assertEqual(profile.count(f'"{name}"'), 1, name)
        self.assertNotIn("target/", profile)
        self.assertNotIn("File::create", profile)

    def test_plan_keeps_baseline_and_cutover_pending(self) -> None:
        plan = _read_contract_input(PROFILE_PLAN, encoding="utf-8")

        self.assertIn("baseline_profile_pending_no_optimization", plan)
        self.assertIn("Do not change geometry ownership before the baseline", plan)
        self.assertIn("31 raw samples", plan)
        self.assertIn("FSlateImageRun", plan)
        self.assertIn("FSlateWidgetRun", plan)


if __name__ == "__main__":
    unittest.main()
