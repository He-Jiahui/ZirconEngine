from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/scene/level_system_render_extract.rs"


def source_text() -> str:
    return SOURCE.read_text(encoding="utf-8")


def compact_source() -> str:
    return "".join(source_text().split())


class Runtime99JSinglePassAnimationPoseExtractPerformanceContract(unittest.TestCase):
    def test_animation_pose_extract_projects_final_rows_from_the_sealed_map(self) -> None:
        compact = compact_source()

        self.assertIn("frame_state.animation_poses().iter().filter_map", compact)
        self.assertIn("RenderSkeletalPoseExtract{", compact)
        self.assertIn("pose:pose.clone()", compact)

    def test_animation_pose_extract_has_no_candidate_or_skeleton_staging_vectors(self) -> None:
        compact = compact_source()

        self.assertNotIn("letcandidate_entities", compact)
        self.assertNotIn("letskeletons", compact)
        self.assertNotIn("frame_state.animation_poses().get(&entity)", compact)

    def test_animation_pose_extract_keeps_only_the_final_vector_materialization(self) -> None:
        source = source_text()

        self.assertLessEqual(source.count("collect::<Vec<_>>();"), 1)
        self.assertNotIn(".keys()", source)
        self.assertNotIn(".copied()", source)


if __name__ == "__main__":
    unittest.main()
