from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
LISTENER_CONTROL = (
    ROOT / "zircon_editor/src/core/editor_event/service/listener_control.rs"
)


def function_region(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for offset in range(opening, len(source)):
        character = source[offset]
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[start : offset + 1]
    raise AssertionError(f"unterminated function: {signature}")


class Editor122SinglePassDeliveryPageProjectionPerformanceContractTests(
    unittest.TestCase
):
    def test_control_branch_projects_shared_records_directly_to_json(self) -> None:
        source = LISTENER_CONTROL.read_text(encoding="utf-8")
        branch = source.split(
            "EditorEventListenerControlRequest::QueryDeliveriesPage", 1
        )[1].split("EditorEventListenerControlRequest::AckDeliveriesThrough", 1)[0]

        self.assertIn("listener_delivery_json(", branch)
        self.assertIn("record.payload.as_ref()", branch)
        self.assertNotIn("EditorEventListenerDelivery::from_shared", branch)
        self.assertNotIn("EditorEventListenerDeliveryPage", branch)
        self.assertNotIn("listener_deliveries(", branch)

    def test_json_projection_has_one_owned_output_stage(self) -> None:
        source = LISTENER_CONTROL.read_text(encoding="utf-8")
        projection = function_region(source, "fn listener_delivery_json")

        self.assertIn("let record = payload.record();", projection)
        for field in (
            "listener_id",
            "delivery_cursor",
            "event_id",
            "sequence",
            "source",
            "operation_id",
            "operation_display_name",
            "operation_arguments",
            "operation_group",
            "result",
        ):
            with self.subTest(field=field):
                self.assertIn(f'"{field}"', projection)
        self.assertNotIn("EditorEventListenerDelivery", projection)
        self.assertNotIn(".clone()", projection)
        self.assertNotIn("to_string()", projection)

    def test_source_guard_keeps_projection_outside_listener_lock(self) -> None:
        source = LISTENER_CONTROL.read_text(encoding="utf-8")

        self.assertIn(
            "fn delivery_page_json_projection_stays_outside_the_listener_lock_scope",
            source,
        )
        self.assertIn('find("listener_delivery_json")', source)
        self.assertIn("assert!(projection > lock_scope_end);", source)


if __name__ == "__main__":
    unittest.main()
