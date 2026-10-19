import unittest
from pathlib import Path

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/zircon_runtime/text/07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md': 'docs/plans/zircon_runtime/text/07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md'}
_HISTORY_IDENTITIES = {'docs/plans/zircon_runtime/text/07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md': (10455, '971e9406345974b85ebc465e40df4a8d06c90ff659d4b8a423a23d079ed58f07')}
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
SHAPING_TESTS = ROOT / "zircon_runtime/src/text/shaping/tests/cases.rs"
PROFILE = (
    ROOT / "zircon_runtime/src/text/shaping/tests/letter_spacing_profile.rs"
)
REVIEW = (
    ROOT
    / "docs/plans/zircon_runtime/text/07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md"
)

# 读取字距剖析配置，确认发布版样本覆盖脚本方向和跨距通道且使用受管执行入口。
class RuntimeTextLetterSpacingProfileContractTests(unittest.TestCase):
    def test_release_profile_is_profiling_only_and_managed(self) -> None:
        root = _read_contract_input(SHAPING_TESTS, encoding="utf-8")
        profile = _read_contract_input(PROFILE, encoding="utf-8")

        self.assertIn('#[cfg(feature = "profiling")]', root)
        self.assertIn("mod letter_spacing_profile;", root)
        self.assertIn("const SAMPLE_COUNT: usize = 31;", profile)
        self.assertIn("const CLUSTER_COUNTS: [usize; 3] = [32, 256, 4_096];", profile)
        self.assertIn("#[ignore =", profile)
        self.assertIn("!cfg!(debug_assertions)", profile)

    def test_profile_covers_required_script_direction_and_span_lanes(self) -> None:
        profile = _read_contract_input(PROFILE, encoding="utf-8")

        for lane in [
            "LatinLigature",
            "Cjk",
            "CombiningMark",
            "EmojiZwj",
            "ArabicRtl",
            "MixedBidi",
            "VerticalCjk",
            "Single",
            "Alternating",
        ]:
            self.assertIn(lane, profile)

    def test_candidate_uses_one_cluster_flag_pass_and_forces_liga_off(self) -> None:
        profile = _read_contract_input(PROFILE, encoding="utf-8")
        start = profile.index("fn apply_candidate_tracking")
        end = profile.index("fn glyph_wrap_line_count", start)
        candidate = profile[start:end]

        self.assertIn('OpenTypeFeature::new(*b"liga", 0)', profile)
        self.assertIn("cluster_flags.cluster_start", candidate)
        self.assertNotIn("graphemes(", candidate)
        self.assertNotIn("cosmic_text", profile)
        self.assertNotIn("letter_spacing(", profile)
        self.assertIn("candidate_cache_identity_supported=false", profile)

    def test_profile_reports_raw_distribution_work_route_and_memory(self) -> None:
        profile = _read_contract_input(PROFILE, encoding="utf-8")

        for field in [
            "p50_ns={p50_ns}",
            "p95_ns={p95_ns}",
            "p99_ns={p99_ns}",
            "samples_ns={samples_ns:?}",
            "rss_delta_bytes={rss_delta_bytes:?}",
            "backend_shape_calls={backend_shape_calls}",
            "glyph_count={glyph_count}",
            "cluster_count={cluster_count}",
            "glyph_wrap_line_count={glyph_wrap_line_count}",
            "requested_bytes={requested_bytes}",
            "route={route}",
        ]:
            self.assertIn(field, profile)
        self.assertNotIn("target/", profile)
        self.assertNotIn("File::create", profile)
        self.assertNotIn("fs::write", profile)

    def test_plan_does_not_claim_unrun_measurements_or_implementation(self) -> None:
        review = _read_contract_input(REVIEW, encoding="utf-8")

        self.assertIn("letter_spacing_release_profile_harness_static_implemented", review)
        self.assertIn("managed_31_sample_baseline_pending", review)
        self.assertIn("letter_spacing_implementation_not_started", review)
        self.assertIn("forced Cosmic fallback lane remains pending", review)


if __name__ == "__main__":
    unittest.main()
