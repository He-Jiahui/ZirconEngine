from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CONTROLLER = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_host_event_controller.rs"
)


class PlayLogRoutingContractTests(unittest.TestCase):
    def test_backend_diagnostics_use_the_attached_play_identity(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")

        self.assertIn("fn play_backend_log_source", source)
        self.assertIn(
            "Some(WorldDomain::Play(instance)) => LogSource::play(instance)", source
        )
        self.assertIn("Some(WorldDomain::Edit) | None => LogSource::runtime()", source)
        self.assertIn("let source = play_backend_log_source(&self.play_sessions);", source)
        self.assertIn("source.clone(),", source)

    def test_backend_log_regressions_cover_attached_and_unattached_routes(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")

        self.assertIn("unattached_play_backend_output_enters_the_runtime_log_channel", source)
        self.assertIn(
            "attached_play_backend_output_enters_its_play_instance_log_channel", source
        )
        self.assertIn("&LogSource::play(instance)", source)


if __name__ == "__main__":
    unittest.main()
