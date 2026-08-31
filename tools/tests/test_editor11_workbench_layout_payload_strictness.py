import json
import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
LAYOUT_ROOT = REPO_ROOT / "zircon_editor/src/ui/workbench/layout"
ACTIVITY_WINDOW_LAYOUT = LAYOUT_ROOT / "activity_window_layout.rs"
FLOATING_WINDOW_LAYOUT = LAYOUT_ROOT / "floating_window_layout.rs"
MAIN_HOST_PAGE_LAYOUT = LAYOUT_ROOT / "main_host_page_layout.rs"
ACTIVITY_DRAWER_LAYOUT = LAYOUT_ROOT / "activity_drawer_layout.rs"
DOCUMENT_NODE = LAYOUT_ROOT / "document_node.rs"
TAB_STACK_LAYOUT = LAYOUT_ROOT / "tab_stack_layout.rs"
PANE_CONSTRAINT_OVERRIDE = (
    REPO_ROOT
    / "zircon_editor/src/ui/workbench/autolayout/pane_constraint_override.rs"
)
AXIS_CONSTRAINT_OVERRIDE = (
    REPO_ROOT
    / "zircon_editor/src/ui/workbench/autolayout/axis_constraint_override.rs"
)
OWNERSHIP_TESTS = (
    REPO_ROOT
    / "zircon_editor/src/tests/workbench/layout/window_drawer_ownership.rs"
)
PAYLOAD_STRICTNESS_TESTS = (
    REPO_ROOT
    / "zircon_editor/src/tests/workbench/layout/payload_strictness.rs"
)
DEFAULT_LAYOUT_FIXTURE = (
    REPO_ROOT / "zircon_editor/fixtures/workbench/default-layout.json"
)


class WorkbenchLayoutPayloadStrictnessContract(unittest.TestCase):
    def test_nested_current_layout_types_reject_unknown_fields(self) -> None:
        for path in (
            ACTIVITY_WINDOW_LAYOUT,
            FLOATING_WINDOW_LAYOUT,
            MAIN_HOST_PAGE_LAYOUT,
            ACTIVITY_DRAWER_LAYOUT,
            DOCUMENT_NODE,
            TAB_STACK_LAYOUT,
            PANE_CONSTRAINT_OVERRIDE,
            AXIS_CONSTRAINT_OVERRIDE,
        ):
            source = path.read_text(encoding="utf-8")
            declaration = re.search(
                r"(?P<attrs>(?:#\[[^\n]+\]\s*)*)pub\s+(?:struct|enum)\s+",
                source,
            )
            self.assertIsNotNone(declaration, path)
            self.assertIn(
                "#[serde(deny_unknown_fields)]",
                declaration.group("attrs"),
                f"{path.name} must fail closed on unknown current-schema fields",
            )

    def test_floating_frame_uses_editor_owned_strict_wire_adapter(self) -> None:
        source = FLOATING_WINDOW_LAYOUT.read_text(encoding="utf-8")
        self.assertIn('#[serde(with = "strict_shell_frame")]', source)
        self.assertIn("mod strict_shell_frame", source)
        self.assertRegex(
            source,
            r"#\[serde\(deny_unknown_fields\)\]\s+struct ShellFrameWire",
        )

    def test_nested_current_layout_fields_are_required(self) -> None:
        activity_source = ACTIVITY_WINDOW_LAYOUT.read_text(encoding="utf-8")
        floating_source = FLOATING_WINDOW_LAYOUT.read_text(encoding="utf-8")

        self.assertNotIn(
            "#[serde(default)]",
            activity_source,
            "current activity-window fields must not be synthesized during decode",
        )
        self.assertNotIn(
            "#[serde(default)]",
            floating_source,
            "current floating-window frame must be present during decode",
        )

    def test_default_fixture_contains_complete_current_activity_window(self) -> None:
        fixture = json.loads(DEFAULT_LAYOUT_FIXTURE.read_text(encoding="utf-8"))
        window = fixture["activity_windows"]["window:workbench"]

        self.assertEqual(window["menu_overflow_mode"], "Auto")
        self.assertEqual(window["region_overrides"], {})
        self.assertEqual(window["view_overrides"], {})

    def test_rust_behavior_covers_missing_and_unknown_nested_fields(self) -> None:
        source = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (OWNERSHIP_TESTS, PAYLOAD_STRICTNESS_TESTS)
        )
        self.assertIn(
            "serialized_workbench_layout_rejects_missing_activity_window_fields",
            source,
        )
        self.assertIn(
            "serialized_workbench_layout_rejects_unknown_nested_fields",
            source,
        )
        self.assertIn(
            "serialized_workbench_layout_rejects_deep_unknown_fields",
            source,
        )


if __name__ == "__main__":
    unittest.main()
