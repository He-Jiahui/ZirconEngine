import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SHAPING_TESTS = ROOT / "zircon_runtime/src/text/shaping/tests.rs"
PROFILE = (
    ROOT / "zircon_runtime/src/text/shaping/tests/letter_spacing_profile.rs"
)
REVIEW = (
    ROOT
    / "docs/plans/zircon_runtime/text/07/2026-08-30-rich-style-shaping-projection-and-letter-spacing-review.md"
)


class RuntimeTextLetterSpacingProfileContractTests(unittest.TestCase):
    def test_release_profile_is_profiling_only_and_managed(self) -> None:
        root = SHAPING_TESTS.read_text(encoding="utf-8")
        profile = PROFILE.read_text(encoding="utf-8")

        self.assertIn('#[cfg(feature = "profiling")]', root)
        self.assertIn("mod letter_spacing_profile;", root)
        self.assertIn("const SAMPLE_COUNT: usize = 31;", profile)
        self.assertIn("const CLUSTER_COUNTS: [usize; 3] = [32, 256, 4_096];", profile)
        self.assertIn("#[ignore =", profile)
        self.assertIn("!cfg!(debug_assertions)", profile)

    def test_profile_covers_required_script_direction_and_span_lanes(self) -> None:
        profile = PROFILE.read_text(encoding="utf-8")

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
        profile = PROFILE.read_text(encoding="utf-8")
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
        profile = PROFILE.read_text(encoding="utf-8")

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
        review = REVIEW.read_text(encoding="utf-8")

        self.assertIn("letter_spacing_release_profile_harness_static_implemented", review)
        self.assertIn("managed_31_sample_baseline_pending", review)
        self.assertIn("letter_spacing_implementation_not_started", review)
        self.assertIn("forced Cosmic fallback lane remains pending", review)


if __name__ == "__main__":
    unittest.main()
