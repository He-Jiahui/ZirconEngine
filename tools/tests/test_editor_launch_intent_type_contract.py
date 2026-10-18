import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
LAUNCH_ARGS = ROOT / "zircon_app/src/entry/cli/launch_args.rs"


class EditorLaunchIntentTypeContractTests(unittest.TestCase):
    def test_project_launch_intent_option_is_explicitly_typed(self):
        source = LAUNCH_ARGS.read_text(encoding="utf-8")

        self.assertIn(
            "let mut project_launch_intent: Option<ProjectLaunchIntent> = None;",
            source,
        )


if __name__ == "__main__":
    unittest.main()
