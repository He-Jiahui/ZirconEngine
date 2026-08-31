import json
import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EDITOR_SOURCE_ROOT = REPO_ROOT / "zircon_editor/src"
UI_SLOT = EDITOR_SOURCE_ROOT / "ui/workbench/layout/activity_drawer_slot.rs"
CORE_SLOT = EDITOR_SOURCE_ROOT / "core/editor_event/workbench/activity_drawer_slot.rs"
SLOT_PREFERENCE = EDITOR_SOURCE_ROOT / "ui/activity/slot.rs"
WORKBENCH_LAYOUT = EDITOR_SOURCE_ROOT / "ui/workbench/layout/workbench_layout.rs"
VIEW_INSTANCES = REPO_ROOT / "zircon_editor/fixtures/workbench/view-instances.json"


class ActivityDrawerSlotHardCutContract(unittest.TestCase):
    def test_runtime_and_core_slots_expose_one_bottom_position(self) -> None:
        for path in (UI_SLOT, CORE_SLOT):
            source = path.read_text(encoding="utf-8")
            self.assertNotIn("BottomLeft", source, path.as_posix())
            self.assertNotIn("BottomRight", source, path.as_posix())
            self.assertIsNone(
                re.search(r"pub\s+fn\s+canonical\s*\(", source),
                f"{path.as_posix()} must not retain legacy slot canonicalization",
            )

    def test_editor_runtime_has_no_legacy_bottom_slot_branches(self) -> None:
        offenders = []
        legacy_variant = re.compile(
            r"(?:Core)?ActivityDrawerSlot::Bottom(?:Left|Right)"
        )
        for path in EDITOR_SOURCE_ROOT.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            if legacy_variant.search(source):
                offenders.append(path.relative_to(REPO_ROOT).as_posix())

        self.assertEqual(
            offenders,
            [],
            f"runtime code still branches on retired bottom drawer slots: {offenders}",
        )

    def test_serialized_inputs_require_the_current_bottom_slot(self) -> None:
        preference = SLOT_PREFERENCE.read_text(encoding="utf-8")
        self.assertNotIn('alias = "BottomLeft"', preference)
        self.assertNotIn('alias = "BottomRight"', preference)

        fixtures = json.loads(VIEW_INSTANCES.read_text(encoding="utf-8"))
        drawer_slots = [
            instance["host"]["Drawer"]
            for instance in fixtures
            if "Drawer" in instance["host"]
        ]
        self.assertNotIn("BottomLeft", drawer_slots)
        self.assertNotIn("BottomRight", drawer_slots)
        self.assertIn("Bottom", drawer_slots)

    def test_layout_normalization_does_not_merge_retired_slots(self) -> None:
        source = WORKBENCH_LAYOUT.read_text(encoding="utf-8")
        self.assertNotIn("canonical_activity_drawers", source)
        self.assertNotIn("merge_drawer_layout", source)


if __name__ == "__main__":
    unittest.main()
