import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EVENT_SOURCE = ROOT / "zircon_runtime_interface/src/ui/component/event.rs"
PAYLOAD_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/template/asset/binding/schema.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[brace : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class RuntimeInterface03SchemaNameDispatchPerformanceContract(unittest.TestCase):
    def test_component_event_schema_names_use_direct_dispatch(self) -> None:
        source = EVENT_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn from_schema_name(value: &str)")
        mappings = {
            "ValueChanged": "ValueChanged",
            "Commit": "Commit",
            "KeyboardAction": "KeyboardAction",
            "KeyboardText": "KeyboardText",
            "TypeaheadExpired": "TypeaheadExpired",
            "Focus": "Focus",
            "Hover": "Hover",
            "Press": "Press",
            "BeginDrag": "BeginDrag",
            "DragDelta": "DragDelta",
            "LargeDragDelta": "LargeDragDelta",
            "EndDrag": "EndDrag",
            "DropHover": "DropHover",
            "ActiveDragTarget": "ActiveDragTarget",
            "OpenPopup": "OpenPopup",
            "OpenPopupAt": "OpenPopupAt",
            "ClosePopup": "ClosePopup",
            "SelectOption": "SelectOption",
            "ToggleExpanded": "ToggleExpanded",
            "AddElement": "AddElement",
            "SetElement": "SetElement",
            "RemoveElement": "RemoveElement",
            "MoveElement": "MoveElement",
            "AddMapEntry": "AddMapEntry",
            "SetMapEntry": "SetMapEntry",
            "RenameMapKey": "RenameMapKey",
            "RemoveMapEntry": "RemoveMapEntry",
            "DropReference": "DropReference",
            "ClearReference": "ClearReference",
            "LocateReference": "LocateReference",
            "OpenReference": "OpenReference",
            "SetVisibleRange": "SetVisibleRange",
            "SetPage": "SetPage",
            "SetWorldTransform": "SetWorldTransform",
            "SetWorldSurface": "SetWorldSurface",
        }
        self.assertIn("match value {", body)
        for schema_name, variant in mappings.items():
            self.assertIn(f'"{schema_name}" => Some(Self::{variant})', body)
        self.assertIn("_ => None", body)
        self.assertNotIn("Self::ALL", body)
        self.assertNotIn(".find(", body)
        self.assertIn(
            "RUNTIME_INTERFACE03_COMPONENT_EVENT_SCHEMA_NAME_DISPATCH_BENCH_V1",
            source,
        )

    def test_action_payload_schema_names_use_direct_dispatch(self) -> None:
        source = PAYLOAD_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn from_schema_name(value: &str)")
        mappings = {
            "additive": "Additive",
            "axis": "Axis",
            "channel": "Channel",
            "checked": "Checked",
            "committed": "Committed",
            "confirm": "Confirm",
            "context": "Context",
            "count": "Count",
            "delta": "Delta",
            "enabled": "Enabled",
            "fields": "Fields",
            "force_full_rebuild": "ForceFullRebuild",
            "index": "Index",
            "payload_kind": "PayloadKind",
            "primary": "Primary",
            "reference": "Reference",
            "scope": "Scope",
            "selection_ids": "SelectionIds",
            "source": "Source",
            "subject": "Subject",
            "surface_entity": "SurfaceEntity",
            "value": "Value",
            "visible": "Visible",
        }
        self.assertIn("match value {", body)
        for schema_name, variant in mappings.items():
            self.assertIn(f'"{schema_name}" => Some(Self::{variant})', body)
        self.assertIn("_ => None", body)
        self.assertNotIn("Self::ALL", body)
        self.assertNotIn(".find(", body)
        self.assertIn(
            "RUNTIME_INTERFACE03_ACTION_PAYLOAD_SCHEMA_NAME_DISPATCH_BENCH_V1",
            source,
        )


if __name__ == "__main__":
    unittest.main()
