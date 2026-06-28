import unittest
import tomllib
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

PRODUCTION_UI_ASSET_ROOTS = [
    "zircon_editor/assets",
    "zircon_runtime/assets",
    "zircon_plugins",
]

RETIRED_IMPORTER_FILES = [
    "zircon_runtime/src/asset/importer/ingest/import_ui_asset.rs",
    "zircon_runtime/src/asset/importer/ingest/import_ui_v2_asset.rs",
]

ACTIVE_SUPPORT_FILES = [
    "zircon_editor/src/ui/asset_editor/promote_widget.rs",
    "zircon_editor/src/ui/host/ui_asset_promotion.rs",
    "zircon_editor/src/ui/host/asset_editor_sessions/mod.rs",
    "zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs",
    "zircon_editor/src/ui/host/editor_event_execution/asset_event.rs",
    "zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs",
    "zircon_editor/src/ui/layouts/views/view_projection.rs",
    "zircon_editor/src/ui/asset_editor/session/promotion_state.rs",
    "zircon_editor/src/ui/asset_editor/style/theme_authoring.rs",
    "zircon_editor/src/ui/asset_editor/style/theme_authoring/merge.rs",
]

USER_VISIBLE_SAMPLE_FILES = [
    "zircon_editor/src/ui/layouts/views/asset_browser/summary_layout.rs",
    "zircon_editor/src/ui/layouts/views/asset_browser/summary_nodes.rs",
    "zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs",
    "zircon_editor/src/ui/layouts/views/asset_browser/tests.rs",
    "zircon_editor/src/tests/ui/asset_browser/bootstrap_assets.rs",
    "zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot.rs",
]

EDITOR_LAYOUT_METADATA_FILES = [
    "zircon_editor/assets/ui/editor/layout/page_templates.toml",
    "zircon_editor/assets/ui/editor/layout/presets.toml",
    "zircon_editor/assets/ui/editor/layout/shell_regions.toml",
]

RETIRED_ACTIVE_SUFFIXES = [
    ".ui.toml",
    ".v2.ui.toml",
]


def _string_values(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for child in value.values():
            yield from _string_values(child)
    elif isinstance(value, list):
        for child in value:
            yield from _string_values(child)


class ZuiStaticSuffixConvergenceTests(unittest.TestCase):
    def test_production_ui_asset_trees_have_no_legacy_suffix_files(self):
        legacy_files: list[str] = []
        for root in PRODUCTION_UI_ASSET_ROOTS:
            root_path = REPO_ROOT / root
            if not root_path.exists():
                continue
            for suffix in RETIRED_ACTIVE_SUFFIXES:
                legacy_files.extend(
                    str(path.relative_to(REPO_ROOT))
                    for path in root_path.rglob(f"*{suffix}")
                )

        if legacy_files:
            self.fail(
                "Production UI asset trees still contain retired suffix files:\n"
                + "\n".join(sorted(legacy_files))
            )

    def test_retired_runtime_importer_files_stay_deleted(self):
        existing = [
            path
            for path in RETIRED_IMPORTER_FILES
            if (REPO_ROOT / path).exists()
        ]
        if existing:
            self.fail(
                "Retired UI importer files should not be restored:\n"
                + "\n".join(existing)
            )

    def test_active_editor_ui_support_paths_accept_only_zui_suffix(self):
        failures: list[str] = []
        for relative_path in ACTIVE_SUPPORT_FILES:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for suffix in RETIRED_ACTIVE_SUFFIXES:
                if suffix in text:
                    failures.append(f"{relative_path}: {suffix}")

        if failures:
            self.fail(
                "Active editor UI support paths still accept or generate retired suffixes:\n"
                + "\n".join(failures)
            )

    def test_asset_browser_user_visible_samples_use_zui_suffix(self):
        failures: list[str] = []
        for relative_path in USER_VISIBLE_SAMPLE_FILES:
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            for suffix in RETIRED_ACTIVE_SUFFIXES:
                if suffix in text:
                    failures.append(f"{relative_path}: {suffix}")

        if failures:
            self.fail(
                "Asset Browser user-visible resource samples still show retired suffixes:\n"
                + "\n".join(failures)
            )

    def test_editor_layout_metadata_references_only_zui_ui_assets(self):
        failures: list[str] = []
        for relative_path in EDITOR_LAYOUT_METADATA_FILES:
            data = tomllib.loads((REPO_ROOT / relative_path).read_text(encoding="utf-8"))
            for value in _string_values(data):
                for suffix in RETIRED_ACTIVE_SUFFIXES:
                    if suffix in value:
                        failures.append(f"{relative_path}: {value}")
                if value.startswith("res://ui/"):
                    asset_path = value.split("#", maxsplit=1)[0]
                    if not asset_path.endswith(".zui"):
                        failures.append(f"{relative_path}: {value}")

        if failures:
            self.fail(
                "Editor layout metadata must reference only .zui UI assets:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
