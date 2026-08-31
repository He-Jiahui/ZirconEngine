from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMPILED_RS = ROOT / "zircon_runtime/src/animation/sequence/compiled.rs"
SEQUENCE_TESTS_RS = ROOT / "zircon_runtime/src/animation/sequence/tests.rs"


def compact(source: str) -> str:
    return re.sub(r"\s+", "", source)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime08cLazyMissingTrackPathPerformanceContractTests(unittest.TestCase):
    def test_resolved_tracks_do_not_materialize_a_discarded_diagnostic_path(self) -> None:
        source = COMPILED_RS.read_text(encoding="utf-8")
        compile_body = function_region(
            source,
            "pub fn compile_sequence_for_world(",
            "impl CompiledAnimationSequence",
        )
        track_loop = function_region(
            compile_body,
            "        for (track_index, track)",
            "    Ok(compiled)",
        )

        self.assertNotIn("let track_path =", track_loop)
        self.assertLess(
            track_loop.index("let Some(writer)"),
            track_loop.index("AnimationTrackPath::new"),
        )

    def test_missing_writer_constructs_and_retains_the_track_path(self) -> None:
        source = compact(COMPILED_RS.read_text(encoding="utf-8"))

        self.assertIn(
            "else{compiled.missing_tracks.push(AnimationTrackPath::new("
            "binding.entity_path.clone(),track.property_path.clone(),));continue;}",
            source,
        )

    def test_existing_success_and_missing_track_oracles_remain_present(self) -> None:
        tests = SEQUENCE_TESTS_RS.read_text(encoding="utf-8")

        self.assertIn(
            "fn compiled_sequence_resolves_numeric_target_once_and_writes_through_compiled_property()",
            tests,
        )
        self.assertIn(
            "fn compiled_sequence_retries_missing_target_only_after_topology_catalog_changes()",
            tests,
        )


if __name__ == "__main__":
    unittest.main()
