from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


class Editor04FocusWorldDomainContractTests(unittest.TestCase):
    def test_scene_selection_messages_require_an_explicit_world_domain(self) -> None:
        selection_domain = read(
            "zircon_editor/src/core/editor_message/ids/selection_domain.rs"
        )

        self.assertIn("Scene(WorldDomain)", selection_domain)
        self.assertNotIn("Scene,", selection_domain)

    def test_focus_object_messages_require_an_explicit_world_domain(self) -> None:
        focus = read("zircon_editor/src/core/editor_message/message/focus.rs")

        self.assertIn("FocusObject {", focus)
        self.assertIn("domain: WorldDomain", focus)

    def test_focus_retention_is_partitioned_by_world_identity(self) -> None:
        retention = read("zircon_editor/src/core/editor_message/retention.rs")

        self.assertIn("FocusObject(WorldDomain)", retention)
        self.assertIn("Selection(SelectionDomain)", retention)
        self.assertIn("FocusMessage::FocusObject { domain, .. }", retention)
        self.assertIn("FocusObject(*domain)", retention)

    def test_message_contract_exercises_distinct_play_instances(self) -> None:
        focus = read("zircon_editor/src/core/editor_message/message/focus.rs")

        self.assertIn("selection_domains_preserve_play_instance_identity", focus)
        self.assertIn("WorldDomain::Play(first)", focus)
        self.assertIn("WorldDomain::Play(second)", focus)


if __name__ == "__main__":
    unittest.main()
