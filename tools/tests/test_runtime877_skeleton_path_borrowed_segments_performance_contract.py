"""Source and cost contract for borrowed skeleton path segments."""

import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/animation/manager/pose.rs"
LOWER = ROOT / "zircon_runtime/src/animation/manager/pose/tests/borrowed_path_tests.rs"


class Runtime877SkeletonPathBorrowedSegmentsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.owner = cls.source.split("fn skeleton_bone_path(", 1)[1].split("\n}\n", 1)[0]

    def test_bone_and_parent_names_are_borrowed(self) -> None:
        self.assertIn("bone.name.as_str()", self.owner)
        self.assertIn("parent_bone.name.as_str()", self.owner)
        self.assertIn("segments.reverse()", self.owner)
        self.assertIn('segments.join("/")', self.owner)
        self.assertNotIn(".clone()", self.owner)

    def test_missing_parent_keeps_the_original_none_propagation(self) -> None:
        self.assertIn("skeleton.bones.get(parent_index as usize)?", self.owner)

    def test_lower_parity_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod borrowed_path_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("runtime877_skeleton_path_borrowed_segments_preserves_exact_paths", lower)
        self.assertIn("RUNTIME877_SKELETON_PATH_BORROWED_SEGMENTS_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_deep_skeleton_clone_model(self) -> None:
        paths, ancestor_depth = 4_096, 32
        self.assertEqual(paths * ancestor_depth, 131_072)
        optimized_clones = paths * ancestor_depth if ".clone()" in self.owner else 0
        self.assertEqual(optimized_clones, 0)


if __name__ == "__main__":
    unittest.main()
