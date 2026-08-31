import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EXTENSION_WORKSPACE_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/extensions"
)
NAVIGATION_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/extension_module_navigation"
)
SPEC_ROOT = NAVIGATION_ROOT / "specs"
BINDING_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_extension_module_template_bindings"
)
REFERENCE_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/reference_menu_actions.rs"
)
PREVIEW_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs"
)
EXTENSION_TAB_ACTION = re.compile(
    r"workbench\.extension\.[a-z_]+\.[a-z_]+_tab\.select"
)
EXTENSION_TAB_CONTROL = re.compile(
    r"WorkbenchExtension[A-Za-z]+Tab\b"
)


class EditorZuiExtensionWorkspaceModeAuthorityContractTests(unittest.TestCase):
    def test_extension_workspaces_do_not_offer_tabs_without_content_stacks(self):
        workspaces = sorted(
            EXTENSION_WORKSPACE_ROOT.rglob("workbench_extension_*_workspace.zui")
        )
        self.assertEqual(44, len(workspaces))

        for path in workspaces:
            with self.subTest(workspace=path.name):
                with path.open("rb") as source:
                    document = tomllib.load(source)
                self.assertFalse(
                    any(
                        node.get("component") == "WorkbenchTab"
                        for node in document["nodes"].values()
                    )
                )
                self.assertFalse(
                    any(
                        "workbench_tab.zui#WorkbenchTab" in item
                        for item in document["imports"]["widgets"]
                    )
                )

    def test_extension_tab_state_is_absent_from_navigation_and_bindings(self):
        sources = [
            NAVIGATION_ROOT.with_suffix(".rs"),
            *sorted(SPEC_ROOT.rglob("*.rs")),
            *sorted(BINDING_ROOT.rglob("*.rs")),
            REFERENCE_ACTIONS,
            PREVIEW_ACTIONS,
        ]
        for path in sources:
            with self.subTest(source=path.relative_to(REPO_ROOT).as_posix()):
                text = path.read_text(encoding="utf-8")
                self.assertIsNone(EXTENSION_TAB_ACTION.search(text))
                self.assertIsNone(EXTENSION_TAB_CONTROL.search(text))

        navigation = NAVIGATION_ROOT.with_suffix(".rs").read_text(encoding="utf-8")
        spec_types = (SPEC_ROOT / "types.rs").read_text(encoding="utf-8")
        reference_actions = REFERENCE_ACTIONS.read_text(encoding="utf-8")
        for retired in (
            "workbench_extension_panel_tab_control_id",
            "workbench_extension_panel_tab_group",
            "workbench_extension_default_panel_tabs",
            "tab_controls",
            "tab_actions",
        ):
            self.assertNotIn(retired, navigation)
            self.assertNotIn(retired, spec_types)
            self.assertNotIn(retired, reference_actions)


if __name__ == "__main__":
    unittest.main()
