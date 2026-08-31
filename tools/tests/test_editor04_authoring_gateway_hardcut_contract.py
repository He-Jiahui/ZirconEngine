import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]


class Editor04AuthoringGatewayHardcutContractTests(unittest.TestCase):
    def test_context_and_builder_name_the_stable_edit_domain_owner(self) -> None:
        context = (REPO_ROOT / "zircon_editor/src/core/context/editor_context.rs").read_text(
            encoding="utf-8"
        )
        builder = (REPO_ROOT / "zircon_editor/src/core/context/builder.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("authoring_gateway: EditorRuntimeGatewayHandle", context)
        self.assertIn("pub fn authoring_gateway(&self)", context)
        self.assertNotIn("pub fn gateway(&self)", context)
        self.assertIn("authoring_gateway: EditorRuntimeGatewayHandle", builder)
        self.assertIn("pub fn with_authoring_gateway(", builder)
        self.assertNotIn("pub fn with_gateway(", builder)

    def test_ui_routes_edit_and_play_through_distinct_named_owners(self) -> None:
        controller = (
            REPO_ROOT / "zircon_editor/src/ui/host/editor_host_event_controller.rs"
        ).read_text(encoding="utf-8")
        construction = (
            REPO_ROOT
            / "zircon_editor/src/ui/workbench/startup/editor_state_construction.rs"
        ).read_text(encoding="utf-8")
        render = (
            REPO_ROOT / "zircon_editor/src/ui/workbench/state/editor_state_render.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "WorldDomain::Edit => Some(self.context.authoring_gateway().clone())",
            controller,
        )
        self.assertIn(
            "WorldDomain::Play(instance) => self.play_sessions.play_gateway(instance)",
            controller,
        )
        self.assertNotIn("context.gateway()", construction)
        self.assertIn("context.authoring_gateway()", construction)
        self.assertNotIn("self.context.gateway()", render)
        self.assertIn(".authoring_gateway()", render)


if __name__ == "__main__":
    unittest.main()
