import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EXTENSION_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/extensions"
)
FEEDBACK_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/extension_module_feedback"
)
FEEDBACK_ENTRY = FEEDBACK_ROOT.parent / "extension_module_feedback.rs"
EXCLUDED_WORKSPACES = {"blend_space", "ui_asset_editor"}


def extension_slug(path: Path) -> str:
    prefix = "workbench_extension_"
    suffix = "_workspace"
    return path.stem[len(prefix) : -len(suffix)]


class EditorZuiExtensionCommandLiveInputProjectionContractTests(unittest.TestCase):
    def test_command_input_scope_is_exact_and_excludes_scan_only_workspace(self):
        controls = []
        workspaces = set()
        for path in sorted(EXTENSION_ROOT.rglob("workbench_extension_*_workspace.zui")):
            slug = extension_slug(path)
            if slug in EXCLUDED_WORKSPACES:
                continue
            with path.open("rb") as source:
                nodes = tomllib.load(source)["nodes"]
            for node in nodes.values():
                if any(
                    event.get("event") == "Change"
                    for event in node.get("events", [])
                ):
                    controls.append(node["control_id"])
                    workspaces.add(slug)

        self.assertEqual(42, len(workspaces))
        self.assertEqual(126, len(controls))
        self.assertEqual(126, len(set(controls)))

    def test_extension_command_feedback_projects_authored_live_inputs(self):
        entry = FEEDBACK_ENTRY.read_text(encoding="utf-8")
        projection = (FEEDBACK_ROOT / "live_input_summary.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("mod live_input_summary;", entry)
        self.assertIn("live_input_summary::for_command(self, action_id)", entry)
        self.assertIn("route.action_id.ends_with(\".edit\")", projection)
        self.assertIn("node.value_text.as_deref()", projection)
        self.assertIn('action_id.strip_suffix(".invoke")', projection)
        self.assertIn("workbench.extension.ui_asset_editor", projection)
        self.assertNotIn("binding_id_for_action_id", projection)


if __name__ == "__main__":
    unittest.main()
