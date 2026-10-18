import re
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
AUTH_FIXTURE = (
    ROOT / "zircon_runtime/tests/fixtures/ui/reactbits_auth_onboarding_components.zui"
)


class AuthOnboardingScrollContractTests(unittest.TestCase):
    def test_compact_stack_can_grow_to_expose_both_cards_to_its_scroll_owner(self):
        source = AUTH_FIXTURE.read_text(encoding="utf-8")

        def node(node_id: str):
            match = re.search(
                rf"(?ms)^\[nodes\.{re.escape(node_id)}\]\r?\n(.*?)(?=^\[|\Z)",
                source,
            )
            self.assertIsNotNone(match, node_id)
            return tomllib.loads("[node]\n" + match.group(1))["node"]

        body = node("body")
        main = node("main")
        self.assertEqual("ScrollableBox", body["component"])
        self.assertEqual("Vertical", body["layout"]["container"]["axis"])
        self.assertEqual([{"node": "main"}], body["children"])
        self.assertEqual({"xs": "column", "md": "row"}, main["props"]["direction"])
        self.assertEqual({"xs": 2, "md": 2}, main["props"]["spacing"])
        self.assertEqual(
            ["auth_card", "onboarding_card"],
            [mount["node"] for mount in main["children"]],
        )
        height = main["layout"]["height"]
        self.assertEqual(560, height["min"])
        self.assertNotIn("preferred", height)
        self.assertNotIn("max", height)
        self.assertNotEqual("Stretch", height.get("stretch"))

        for card_id in ("auth_card", "onboarding_card"):
            card_height = node(card_id)["layout"]["height"]
            self.assertEqual(560, card_height["min"])
            self.assertNotIn("preferred", card_height)
            self.assertNotIn("max", card_height)
            self.assertNotEqual("Stretch", card_height.get("stretch"))

        self.assertEqual("start", node("onboarding_back")["props"]["icon_placement"])
        self.assertEqual("end", node("onboarding_continue")["props"]["icon_placement"])

        action_mounts = node("onboarding_actions")["children"]
        self.assertEqual(
            ["onboarding_back", "onboarding_action_gap", "onboarding_continue"],
            [mount["node"] for mount in action_mounts],
        )
        gap = node("onboarding_action_gap")
        self.assertEqual("Space", gap["component"])
        self.assertEqual(0, gap["layout"]["width"]["min"])
        self.assertEqual("Stretch", gap["layout"]["width"]["stretch"])


if __name__ == "__main__":
    unittest.main()
