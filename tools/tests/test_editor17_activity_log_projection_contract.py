from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ACTIVITY_VIEW = ROOT / "zircon_editor" / "src" / "ui" / "activity" / "view.rs"


class ActivityLogProjectionContractTests(unittest.TestCase):
    def test_activity_keeps_the_core_log_record_as_its_only_log_state(self) -> None:
        source = ACTIVITY_VIEW.read_text(encoding="utf-8")

        self.assertIn("struct ActivityLogView", source)
        self.assertIn("record: LogRecord", source)
        self.assertIn("fn activity_log_views(records: &[LogRecord])", source)
        self.assertIn(".map(ActivityLogView::new)", source)
        self.assertNotIn("message: String", source.split("struct ActivityLogView", 1)[1].split("impl ActivityLogView", 1)[0])

    def test_activity_log_view_preserves_typed_source_and_jump_dispatch(self) -> None:
        source = ACTIVITY_VIEW.read_text(encoding="utf-8")

        for accessor in (
            "fn sequence(&self) -> u64",
            "fn source(&self) -> &LogSource",
            "fn severity(&self) -> LogSeverity",
            "fn message(&self) -> &str",
            "fn timestamp_frame(&self) -> u64",
            "fn jump(&self) -> Option<&LogJump>",
            "activity_log_views_preserve_the_core_record_and_jump_target",
        ):
            self.assertIn(accessor, source)

        log_view_impl = source.split("impl ActivityLogView", 1)[1].split(
            "/// Builds a read-only Activity view", 1
        )[0]
        self.assertNotIn("const fn severity(&self)", log_view_impl)
        self.assertNotIn("const fn timestamp_frame(&self)", log_view_impl)


if __name__ == "__main__":
    unittest.main()
