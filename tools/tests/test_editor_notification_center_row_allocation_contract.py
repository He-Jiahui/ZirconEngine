from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
NOTIFICATION_CENTER = (
    ROOT
    / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection/notification_center"
)


class EditorNotificationCenterRowAllocationContractTests(unittest.TestCase):
    def test_entry_constructor_does_not_clone_default_title_eagerly(self) -> None:
        source = (NOTIFICATION_CENTER / "entry.rs").read_text(encoding="utf-8")
        constructor = source.split("pub(super) fn new", 1)[1].split(
            "pub(super) fn matches_id", 1
        )[0]

        self.assertIn("title: String::new()", constructor)
        self.assertNotIn("title: id.clone()", constructor)

    def test_pipe_parser_falls_back_only_when_title_field_is_absent(self) -> None:
        source = (NOTIFICATION_CENTER / "parse.rs").read_text(encoding="utf-8")
        function = source.split("fn notification_entry_from_string", 1)[1].split(
            "fn notification_entry_from_table", 1
        )[0]

        self.assertIn("let mut has_explicit_title = false", function)
        self.assertIn("has_explicit_title = true", function)
        self.assertIn("if !has_explicit_title", function)
        self.assertIn("entry.title = entry.id.clone()", function)

    def test_tone_normalization_uses_static_case_insensitive_names(self) -> None:
        source = (NOTIFICATION_CENTER / "attributes.rs").read_text(encoding="utf-8")
        function = source.split("pub(super) fn normalized_tone", 1)[1]

        self.assertIn("-> &'static str", function)
        self.assertIn("eq_ignore_ascii_case", function)
        self.assertNotIn("to_ascii_lowercase", function)


if __name__ == "__main__":
    unittest.main()
