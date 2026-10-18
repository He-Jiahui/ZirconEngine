import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
REGISTRATION = (
    ROOT / "zircon_plugins/ai/runtime/src/plugin/registration.rs"
)


class AiRuntimeRegistrationContractTests(unittest.TestCase):
    def test_scene_system_uses_the_typed_tick_context(self):
        source = REGISTRATION.read_text(encoding="utf-8")

        self.assertIn(
            "let delta_seconds = context.tick().delta_seconds();",
            source,
        )
        self.assertNotIn("context.delta_seconds", source)

    def test_event_readers_share_the_cursor_and_event_store_lifetime(self):
        source = REGISTRATION.read_text(encoding="utf-8")

        self.assertEqual(source.count("        &'events mut self,"), 2)
        self.assertIn("let hearing_events = self", source)
        self.assertIn("for event in hearing_events", source)
        self.assertIn("let animation_events = self", source)
        self.assertIn("for event in animation_events", source)


if __name__ == "__main__":
    unittest.main()
