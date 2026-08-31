from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class Editor06BindingRouteAuthorityContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_runtime_and_editor_publish_binding_routes_without_stub_api(self) -> None:
        runtime_registration = self.read(
            "zircon_runtime/src/ui/event_ui/manager/registration.rs"
        )
        editor_service = self.read("zircon_editor/src/ui/control/service.rs")

        self.assertIn("pub fn register_binding_route", runtime_registration)
        self.assertIn("self.register_route_entry(binding, None)", runtime_registration)
        self.assertIn("pub fn register_binding_route", editor_service)
        self.assertIn("self.event_manager.register_binding_route(binding)", editor_service)
        self.assertNotIn("register_route_stub", runtime_registration)
        self.assertNotIn("register_route_stub", editor_service)

    def test_workbench_reflection_uses_one_binding_route_owner(self) -> None:
        reflection_root = ROOT / "zircon_editor/src/ui/workbench/reflection"
        production = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(reflection_root.rglob("*.rs"))
        )
        module = self.read(
            "zircon_editor/src/ui/workbench/reflection/route_registration/mod.rs"
        )
        owner = self.read(
            "zircon_editor/src/ui/workbench/reflection/route_registration/binding_route.rs"
        )

        self.assertIn("mod binding_route;", module)
        self.assertIn("register_binding_route", owner)
        self.assertNotIn("register_stub_route", production)
        self.assertNotIn("mod stub_route", production)

    def test_editor_host_invokes_the_typed_binding_behind_a_route(self) -> None:
        control_requests = self.read(
            "zircon_editor/src/ui/host/editor_event_control_requests.rs"
        )
        route_body = control_requests.split("fn invoke_route(", 1)[1].split(
            "fn call_action(", 1
        )[0]

        self.assertIn("control_service.route_binding(route_id)", route_body)
        self.assertIn("EditorUiBinding::from_ui_binding", route_body)
        self.assertIn("self.invoke_editor_binding(Some(route_id), editor_binding)", route_body)


if __name__ == "__main__":
    unittest.main()
