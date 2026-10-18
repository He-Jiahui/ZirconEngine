"""Guard the client fixed-step commit ordering around timeline publication."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
FRAME_DRIVER = (
    ROOT
    / "examples"
    / "woc"
    / "native"
    / "apps"
    / "woc_client"
    / "src"
    / "presentation"
    / "frame_driver.rs"
)


class WocClientTimelineCommitContractTests(unittest.TestCase):
    def test_timeline_publish_failure_rolls_back_before_consuming_input(self):
        source = FRAME_DRIVER.read_text(encoding="utf-8")
        push = source.index("let push_result = match self.timeline.push(snapshot)")
        commit = source.index("self.accumulator_ns -= step_ns", push)
        rollback = source.index("self.authority.rollback(authority_checkpoint)", push)
        self.assertLess(rollback, commit)
        self.assertIn("ClientFrameDriverError::TimelineRollback", source[push:commit])

    def test_successful_publish_is_the_commit_fence(self):
        source = FRAME_DRIVER.read_text(encoding="utf-8")
        push = source.index("let push_result = match self.timeline.push(snapshot)")
        commit = source.index("self.accumulator_ns -= step_ns", push)
        self.assertIn("match self.timeline.push(snapshot)", source[push - 20 : push + 80])
        self.assertIn("Ok(result) => result", source[push : commit])
        self.assertNotIn("self.pending_commands.clear()", source[push : commit])


if __name__ == "__main__":
    unittest.main()
