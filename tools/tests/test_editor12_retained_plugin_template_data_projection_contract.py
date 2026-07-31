"""Static contract coverage for plugin V2 pane data reaching the retained host."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative_path: str) -> str:
    return (ROOT / relative_path).read_text(encoding="utf-8")


class EditorPluginV2PaneDataProjectionTests(unittest.TestCase):
    def test_enabled_extension_data_sources_are_snapshotted_outside_the_shell_lock(self) -> None:
        runtime_access = source(
            "zircon_editor/src/ui/host/editor_event_runtime_access.rs"
        )
        registry = source("zircon_editor/src/core/editor_extension.rs")

        self.assertIn("ui_template_pane_data_sources", registry)
        self.assertIn("ui_template_pane_data_snapshots", runtime_access)
        self.assertIn("source.snapshot()", runtime_access)
        self.assertIn("is_enabled_by(&enabled_capabilities)", runtime_access)

    def test_template_v2_snapshots_flow_to_all_pane_locations(self) -> None:
        payloads = source(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs"
        )
        presentation = source(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/presentation.rs"
        )
        apply = source("zircon_editor/src/ui/retained_host/ui/apply_presentation.rs")
        shell = source(
            "zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs"
        )
        floating = source(
            "zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs"
        )
        panes = source(
            "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs"
        )
        context = source(
            "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_presentation.rs"
        )
        builders = source(
            "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/mod.rs"
        )

        for text, name in [
            (payloads, "payload collection"),
            (presentation, "recompute presentation"),
            (apply, "presentation application"),
            (shell, "dock shell"),
            (floating, "floating pane"),
            (panes, "pane projection"),
        ]:
            self.assertIn("template_v2_data", text, name)
        self.assertIn("with_template_v2_snapshot", context)
        self.assertIn("template_v2_snapshot", builders)
        self.assertIn("TemplateV2PanePayload", builders)
        self.assertIn("component_patches", builders)

        implementation = "\n".join(
            [payloads, presentation, apply, shell, floating, panes, context, builders]
        ).lower()
        self.assertNotIn("navigation.", implementation)


if __name__ == "__main__":
    unittest.main()
