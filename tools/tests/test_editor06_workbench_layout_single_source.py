import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
LAYOUT_ROOT = REPO_ROOT / "zircon_editor/src/ui/workbench/layout"
WORKBENCH_LAYOUT = LAYOUT_ROOT / "workbench_layout.rs"
MAIN_HOST_PAGE_LAYOUT = LAYOUT_ROOT / "main_host_page_layout.rs"
DEFAULT_LAYOUT_FIXTURE = (
    REPO_ROOT / "zircon_editor/fixtures/workbench/default-layout.json"
)
EDITOR_SOURCE_ROOT = REPO_ROOT / "zircon_editor/src"
RETIRED_ROOT_FIELDS = {"drawers", "region_overrides", "view_overrides"}
LIKELY_WORKBENCH_ROOT_ACCESS = re.compile(
    r"(?s)\b(?:fixture\s*\.\s*layout|restored(?:_layout)?|current_layout|layout)"
    r"\s*\.\s*(?:drawers|region_overrides|view_overrides)\b"
)


def rust_brace_block(source: str, opening_brace: int) -> str:
    depth = 0
    for index in range(opening_brace, len(source)):
        token = source[index]
        if token == "{":
            depth += 1
        elif token == "}":
            depth -= 1
            if depth == 0:
                return source[opening_brace + 1 : index]
    raise AssertionError("unclosed Rust brace block")


def top_level_field_names(block: str) -> set[str]:
    depth = 0
    fields = set()
    for line in block.splitlines():
        if depth == 0:
            field = re.match(r"\s*([A-Za-z_][A-Za-z0-9_]*)\s*:", line)
            if field:
                fields.add(field.group(1))
        depth += line.count("{") - line.count("}")
    return fields


class WorkbenchLayoutSingleSourceContract(unittest.TestCase):
    def test_workbench_layout_has_no_top_level_drawer_mirror(self) -> None:
        source = WORKBENCH_LAYOUT.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", maxsplit=1)[0]

        self.assertIsNone(
            re.search(r"\bpub\s+drawers\s*:", production),
            "WorkbenchLayout must store drawers only under ActivityWindowLayout",
        )
        for mirrored_field in ("region_overrides", "view_overrides"):
            self.assertIsNone(
                re.search(rf"\bpub\s+{mirrored_field}\s*:", production),
                f"WorkbenchLayout must store {mirrored_field} only under ActivityWindowLayout",
            )
        self.assertNotIn(
            "sync_legacy_drawers_from_active_activity_window",
            production,
            "layout commands must not maintain a second drawer state tree",
        )
        self.assertNotIn(
            "activity_windows_from_legacy_drawers",
            production,
            "the current layout type must not synthesize its owner tree from retired fields",
        )
        self.assertIn(
            "#[serde(deny_unknown_fields)]",
            production,
            "retired serialized fields must fail instead of being silently ignored",
        )
        activity_windows_field = re.search(
            r"(?P<attrs>(?:\s*#\[[^\]]+\]\s*)*)pub\s+activity_windows\s*:",
            production,
        )
        self.assertIsNotNone(activity_windows_field)
        self.assertNotIn(
            "serde(default)",
            activity_windows_field.group("attrs"),
            "the canonical activity-window tree must be present in persisted layouts",
        )

    def test_production_layout_code_has_no_legacy_drawer_sync_calls(self) -> None:
        offenders = []
        for path in LAYOUT_ROOT.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            production = source.split("#[cfg(test)]", maxsplit=1)[0]
            if "sync_legacy_drawers_from_active_activity_window" in production:
                offenders.append(path.relative_to(REPO_ROOT).as_posix())

        self.assertEqual(
            offenders,
            [],
            f"retired WorkbenchLayout drawer mirror is still synchronized by {offenders}",
        )

    def test_all_workbench_layout_literals_exclude_retired_root_fields(self) -> None:
        offenders = []
        for path in EDITOR_SOURCE_ROOT.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            for layout in re.finditer(r"\bWorkbenchLayout\s*\{", source):
                opening_brace = source.find("{", layout.start())
                retired = top_level_field_names(
                    rust_brace_block(source, opening_brace)
                ) & RETIRED_ROOT_FIELDS
                if retired:
                    line = source.count("\n", 0, layout.start()) + 1
                    offenders.append(
                        f"{path.relative_to(REPO_ROOT).as_posix()}:{line}:"
                        f"{','.join(sorted(retired))}"
                    )

        self.assertEqual(
            offenders,
            [],
            f"WorkbenchLayout literals still initialize retired root fields: {offenders}",
        )

    def test_workbench_layout_consumers_do_not_read_retired_root_fields(self) -> None:
        offenders = []
        for path in EDITOR_SOURCE_ROOT.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            for access in LIKELY_WORKBENCH_ROOT_ACCESS.finditer(source):
                line = source.count("\n", 0, access.start()) + 1
                offenders.append(
                    f"{path.relative_to(REPO_ROOT).as_posix()}:{line}:"
                    f"{' '.join(access.group(0).split())}"
                )

        self.assertEqual(
            offenders,
            [],
            f"WorkbenchLayout consumers still read retired root fields: {offenders}",
        )

    def test_persisted_fixture_uses_required_activity_window_ownership(self) -> None:
        page_source = MAIN_HOST_PAGE_LAYOUT.read_text(encoding="utf-8")
        self.assertNotIn(
            '#[serde(default = "ActivityWindowId::workbench")]',
            page_source,
            "workbench pages must name their activity-window owner explicitly",
        )

        fixture = DEFAULT_LAYOUT_FIXTURE.read_text(encoding="utf-8")
        self.assertNotIn('"drawers":', fixture)
        self.assertIn('"activity_windows":', fixture)
        self.assertIn('"activity_window": "window:workbench"', fixture)


if __name__ == "__main__":
    unittest.main()
