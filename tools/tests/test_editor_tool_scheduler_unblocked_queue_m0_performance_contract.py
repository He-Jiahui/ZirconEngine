from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCHEDULER = ROOT / "zircon_editor/src/core/tools/scheduler.rs"
RESOURCE_SET = ROOT / "zircon_editor/src/core/tools/resource_set.rs"
SERVICE = ROOT / "zircon_editor/src/core/context/tool_scheduler.rs"


def source() -> str:
    return SCHEDULER.read_text(encoding="utf-8")


class EditorToolSchedulerUnblockedQueueContract(unittest.TestCase):
    def test_set_transition_paths_share_unblocked_single_queue_promotion(self) -> None:
        body = source()

        self.assertGreaterEqual(
            body.count("self.promote_available_claims(&mut events)"), 3
        )
        self.assertIn("activated.extend(self.promote_waiting_singles(events));", body)
        self.assertNotIn(
            "activated.is_none() && self.set_queue.is_empty()", body
        )

    def test_three_value_resource_set_dedup_does_not_build_a_tree(self) -> None:
        body = RESOURCE_SET.read_text(encoding="utf-8")

        self.assertNotIn("BTreeSet", body)
        self.assertIn("resources.sort_unstable();", body)
        self.assertIn("resources.dedup();", body)

    def test_service_parses_builtin_topic_once_at_construction(self) -> None:
        body = SERVICE.read_text(encoding="utf-8")
        publish = body.split("fn dispatch_outbox", maxsplit=1)[1]

        self.assertIn("topic: EditorTopic", body)
        self.assertEqual(body.count("topic: EditorTopic::tool()"), 1)
        self.assertNotIn("EditorTopic::parse(TOPIC_TOOL)", body)
        self.assertNotIn("EditorTopic::parse", publish)


if __name__ == "__main__":
    unittest.main()
