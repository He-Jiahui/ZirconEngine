from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]


class Plugins05NavigationOverlayTests(unittest.TestCase):
    def test_navigation_overlay_has_one_runtime_frame_and_real_provider(self) -> None:
        runtime_frame = (
            REPO_ROOT / "zircon_plugins/navigation/runtime/src/overlay_frame.rs"
        ).read_text(encoding="utf-8")
        runtime_manager = (
            REPO_ROOT / "zircon_plugins/navigation/runtime/src/manager.rs"
        ).read_text(encoding="utf-8")
        runtime_plugin = (
            REPO_ROOT / "zircon_plugins/navigation/runtime/src/plugin.rs"
        ).read_text(encoding="utf-8")
        editor_mirror = (
            REPO_ROOT / "zircon_plugins/navigation/editor/src/runtime_mirror.rs"
        ).read_text(encoding="utf-8")
        editor_provider = (
            REPO_ROOT
            / "zircon_plugins/navigation/editor/src/viewport_overlay_provider.rs"
        ).read_text(encoding="utf-8")
        editor_plugin = (
            REPO_ROOT / "zircon_plugins/navigation/editor/src/plugin.rs"
        ).read_text(encoding="utf-8")
        plugin_manifest = (
            REPO_ROOT / "zircon_plugins/navigation/plugin.toml"
        ).read_text(encoding="utf-8")

        for contract in (
            "pub struct NavigationOverlayFrame",
            "pub owner_generation: u64",
            "pub nav_mesh: NavigationGizmoSnapshot",
            "pub tick_report: NavAgentTickReport",
        ):
            self.assertIn(contract, runtime_frame)
        self.assertIn(
            'NAVIGATION_OVERLAY_FRAME_EVENT_ID: &str = "navigation.events.overlay_frame"',
            runtime_frame,
        )
        self.assertIn("pub fn navigation_overlay_frame", runtime_manager)
        self.assertIn("event(NAVIGATION_OVERLAY_FRAME_EVENT_ID)", runtime_plugin)
        self.assertIn("world.send_event(overlay_frame)", runtime_plugin)

        self.assertIn("type Payload = NavigationOverlayFrame", editor_mirror)
        self.assertIn("impl ViewportOverlayProvider for NavigationViewportOverlayProvider", editor_provider)
        self.assertIn("ViewportOverlayProviderRegistration::new", editor_provider)
        self.assertIn("impl EditorPlugin for NavigationEditorPlugin", editor_plugin)
        self.assertIn(
            "navigation_runtime_event_consumers_with_mirror(pie_mirror.clone())",
            editor_plugin,
        )
        self.assertIn(
            "register_navigation_extensions(registry, self.pie_mirror())",
            editor_plugin,
        )
        self.assertNotIn("pub fn navigation_runtime_event_consumers()", editor_mirror)
        self.assertNotIn("authoring_plugin!", editor_plugin)
        self.assertIn('consumer_id = "navigation.editor.overlay_frame"', plugin_manifest)
        self.assertIn('event_id = "navigation.events.overlay_frame"', plugin_manifest)
        self.assertIn('payload_schema = "navigation.events.overlay_frame.v1"', plugin_manifest)
        self.assertNotIn('consumer_id = "navigation.editor.agent_tick"', plugin_manifest)


if __name__ == "__main__":
    unittest.main()
