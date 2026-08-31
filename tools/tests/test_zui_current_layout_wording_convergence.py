import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

CURRENT_LAYOUT_WORDING_TARGETS = {
    "zircon_editor/src/tests/ui/boundary/template_assets.rs": [
        "hard_cut_to_v2",
        "v2-replaced editor production asset",
        "missing v2 asset",
        "through v2 asset",
        "v2 authoring asset",
        "v2 fixture marker",
        "v2 marker",
        "component showcase v2 asset",
    ],
    "zircon_editor/src/tests/ui/boundary/view_projection_cutover.rs": [
        "asset browser v2 asset",
        "v2 asset browser",
    ],
    "zircon_editor/src/tests/ui/boundary/workbench_projection_cutover": [
        "hard_cut_to_v2",
    ],
    "zircon_editor/src/tests/host/retained_window/native_host_contract": [
        "v2 asset search fields",
        "v2 asset browser buttons",
        "v2 asset browser text fields",
    ],
    "zircon_editor/src/tests/ui/assets_activity/window_template.rs": [
        "asset window v2 asset should parse",
    ],
    "zircon_editor/src/tests/ui/boundary/runtime_ui_golden.rs": [
        "runtime_v2_asset",
        "runtime_v2_fixtures",
        "build_v2_surface",
        "v2 surface",
        "ui v2 asset",
        "runtime v2 semantic",
    ],
    "zircon_runtime/src/asset/assets/ui.rs": [
        "failed to parse ui v2 asset document",
        "expected ui v2 asset kind",
        "ui v2 component documents",
    ],
    "zircon_runtime/src/asset/artifact/cache_payload/ui.rs": [
        "ui v2 asset document cache",
        "ui v2 view document cache",
        "ui v2 component document cache",
        "ui v2 style document cache",
    ],
    "zircon_runtime/src/asset/importer/error.rs": [
        "ui v2 asset document failed",
    ],
    "zircon_runtime/src/ui/tests/boundary/asset_fixture_projection.rs": [
        "v2 heap-resident file cache",
        "ui_v2_surface_projection",
        "ui v2 surface_builder.rs",
        "ui v2 surface tree file",
        "ui v2 surface projection",
    ],
    "zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs": [
        "expected ui v2 asset kind",
    ],
    "zircon_editor/tests/integration_contracts/activity_drawer_window_template.rs": [
        "v2 asset should parse",
    ],
    "zircon_editor/tests/integration_contracts/editor_main_frame_template.rs": [
        "v2 asset should parse",
    ],
    "zircon_editor/tests/integration_contracts/ui_layout_editor_window_template.rs": [
        "v2 asset should parse",
    ],
    "zircon_editor/tests/integration_contracts/workbench_window_template.rs": [
        "v2 asset should parse",
    ],
}

RETIRED_SUFFIX_NEGATIVE_MARKERS = {
    "zircon_editor/src/tests/ui/boundary/template_assets.rs": [
        "activity_drawer_window.v2.ui.toml",
        "workbench_shell.ui.toml",
        "runtime_hud.ui.toml",
        "component_widgets.ui.toml#ShowcaseSection",
    ],
    "zircon_editor/src/tests/ui/boundary/view_projection_cutover.rs": [
        "res://ui/theme/editor_base.ui.toml",
    ],
}


class ZuiCurrentLayoutWordingConvergenceTests(unittest.TestCase):
    def test_current_layout_positive_wording_uses_zui_authority(self):
        failures: list[str] = []
        for relative_path, stale_phrases in CURRENT_LAYOUT_WORDING_TARGETS.items():
            target = REPO_ROOT / relative_path
            sources = sorted(target.rglob("*.rs")) if target.is_dir() else [target]
            self.assertTrue(
                sources and all(source.is_file() for source in sources),
                f"expected current layout sources under {relative_path}",
            )
            text = "\n".join(source.read_text(encoding="utf-8") for source in sources)
            for phrase in stale_phrases:
                if phrase in text:
                    failures.append(f"{relative_path}: {phrase}")

        if failures:
            self.fail(
                "Current .zui layout tests still describe active assets as v2 wording:\n"
                + "\n".join(failures)
            )

    def test_retired_suffix_negative_assertions_remain_explicit(self):
        failures: list[str] = []
        for relative_path, required_markers in RETIRED_SUFFIX_NEGATIVE_MARKERS.items():
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for marker in required_markers:
                if marker not in text:
                    failures.append(f"{relative_path}: {marker}")

        if failures:
            self.fail(
                "Retired .ui.toml / .v2.ui.toml negative assertions were removed:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
