from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMPILED_RS = ROOT / "zircon_runtime/src/animation/sequence/compiled.rs"
SEQUENCE_TESTS_RS = ROOT / "zircon_runtime/src/animation/sequence/tests/cases.rs"


def block_region(source: str, marker: str) -> str:
    """Return the balanced Rust block that starts at *marker*.

    The production function has been through harmless formatter/ownership
    changes. Matching the balanced block keeps
    this contract tied to the control-flow boundary instead of indentation or
    a particular closing expression.
    """
    offset = source.index(marker)
    opening = source.index("{", offset)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[offset : index + 1]
    raise AssertionError(f"unterminated Rust block: {marker}")


class Runtime08cLazyMissingTrackPathPerformanceContractTests(unittest.TestCase):
    def test_resolved_tracks_do_not_materialize_a_discarded_diagnostic_path(self) -> None:
        source = COMPILED_RS.read_text(encoding="utf-8")
        compile_body = block_region(source, "pub fn compile_sequence_for_world(")
        track_loop = block_region(compile_body, "for (track_index, track)")

        self.assertNotIn("let track_path =", track_loop)
        self.assertLess(
            track_loop.index("let Some(writer)"),
            track_loop.index("AnimationTrackPath::new"),
        )

    def test_missing_writer_constructs_and_retains_the_track_path(self) -> None:
        source = COMPILED_RS.read_text(encoding="utf-8")

        self.assertRegex(
            source,
            r"missing_tracks\s*\.\s*extend\s*\(\s*binding\.tracks\(\)\.iter\(\)\.map",
        )
        self.assertRegex(
            source,
            r"missing_tracks\s*\.\s*push\s*\(\s*AnimationTrackPath::new\(",
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
