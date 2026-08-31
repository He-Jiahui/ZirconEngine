from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOST = ROOT / "zircon_editor/src/ui/retained_host/host_contract"
SURFACE_HIT_TEST = HOST / "surface_hit_test"
NATIVE_POINTER = HOST / "native_pointer"


class EditorPointerMoveBorrowedHitPerformanceContractTests(unittest.TestCase):
    def test_move_hit_is_a_narrow_borrowed_view(self) -> None:
        model = (
            SURFACE_HIT_TEST / "template_node/model.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("struct TemplateNodePointerMoveHit", model)
        move_hit = model.split("struct TemplateNodePointerMoveHit", 1)[1]
        move_hit = move_hit.split("}", 1)[0]

        self.assertIn("pub(crate) control_id: &'a str", move_hit)
        self.assertIn("pub(crate) action_id: &'a str", move_hit)
        self.assertIn("pub(crate) value_text: &'a str", move_hit)
        self.assertIn("pub(crate) kind: TemplateNodePointerMoveKind", move_hit)
        self.assertNotIn("SharedString", move_hit)
        self.assertNotIn("binding_id", move_hit)
        self.assertNotIn("component_role", move_hit)
        self.assertNotIn("table_row_", move_hit)

    def test_move_route_never_materializes_the_owned_button_hit(self) -> None:
        routing = (
            NATIVE_POINTER / "routing/workbench.rs"
        ).read_text(encoding="utf-8")
        move = (
            NATIVE_POINTER / "move_dispatch/workbench.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("route_pointer_move_to_workbench_generation", routing)
        self.assertIn("Option<TemplateNodePointerMoveHit<'_>>", routing)
        self.assertIn(
            "hit_test_workbench_window_template_node_for_pointer_move_with_index",
            routing,
        )
        self.assertIn("route_pointer_move_to_workbench_generation", move)
        self.assertNotIn("route_pointer_to_workbench_generation", move)
        self.assertNotIn("TemplateNodePointerHit", move)

    def test_button_route_keeps_owned_payload_authority(self) -> None:
        button = (
            NATIVE_POINTER / "button_dispatch/workbench/entry.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("route_pointer_to_workbench_generation", button)
        self.assertNotIn("route_pointer_move_to_workbench_generation", button)

    def test_primary_press_consumes_the_owned_hit_without_a_second_clone(self) -> None:
        activation = (
            NATIVE_POINTER / "button_dispatch/workbench/primary/activation.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("hit.clone()", activation)
        self.assertLess(
            activation.index("let damage = hit_damage"),
            activation.rindex("dispatch_template_node_primary_press"),
        )

    def test_lower_regression_proves_generation_owned_string_identity(self) -> None:
        tests = (
            SURFACE_HIT_TEST / "template_node_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "workbench_pointer_move_hit_borrows_generation_owned_node_and_popup_strings",
            tests,
        )
        self.assertGreaterEqual(tests.count("std::ptr::eq("), 3)


if __name__ == "__main__":
    unittest.main()
