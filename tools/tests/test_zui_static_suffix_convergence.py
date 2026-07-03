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

EDITOR_UI_ASSET_EDITING_TEST_PATHS = [
    "zircon_editor/src/tests/editing/ui_asset",
    "zircon_editor/src/tests/editing/ui_asset_palette_drop.rs",
    "zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs",
    "zircon_editor/src/tests/editing/ui_asset_replay.rs",
    "zircon_editor/src/tests/editing/ui_asset_theme_authoring.rs",
]

EDITOR_HOST_MANAGER_UI_ASSET_TEST_PATHS = [
    "zircon_editor/src/tests/host/manager",
]

EDITOR_UI_ASSET_EDITOR_TEST_PATHS = [
    "zircon_editor/src/tests/ui/ui_asset_editor",
]

EDITOR_UI_ASSET_EDITOR_ALLOWED_RETIRED_SUFFIX_LINES = {
    "zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs": [
        "editor_widgets.ui.toml",
        "editor_base.ui.toml",
    ],
}

EDITOR_HOST_THEME_TOOLING_TEST_PATHS = [
    "zircon_editor/src/tests/host/ui_asset_editor_theme_tooling",
]

RUNTIME_UI_ACTIVE_TEST_FILES = [
    "zircon_runtime/src/ui/tests/asset_prototype_store.rs",
]

EDITOR_UI_COMPONENT_ADAPTER_TEST_FILES = [
    "zircon_editor/src/tests/ui/component_adapter.rs",
]

EDITOR_RETAINED_HOST_PROJECTION_TEST_FILES = [
    "zircon_editor/src/tests/host/retained_window/native_host_contract.rs",
    "zircon_editor/src/ui/retained_host/ui/tests/host_scene_projection.rs",
    "zircon_editor/src/ui/retained_host/ui/tests/host_scene_projection/assertions.rs",
]

EDITOR_EXTENSION_CONTRACT_TEST_FILES = [
    "zircon_editor/src/tests/editor_authoring_extension_descriptors.rs",
    "zircon_editor/src/tests/editor_event/runtime.rs",
]

EDITOR_VIEW_PROJECTION_TEST_FILES = [
    "zircon_editor/src/ui/layouts/views/view_projection/tests.rs",
]

RUNTIME_EXTENSION_COMPONENT_TEST_FILES = [
    "zircon_runtime/src/tests/plugin_extensions/extension_registry_components.rs",
]

RUNTIME_ASSET_UI_REFERENCE_TEST_FILES = [
    "zircon_runtime/src/asset/tests/assets/ui.rs",
    "zircon_runtime/src/asset/tests/assets/ui/references.rs",
]

GLOBAL_MATERIAL_SURFACE_TEST_FILE = (
    "zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs"
)

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

    def test_production_zui_assets_do_not_display_retired_suffixes(self):
        failures: list[str] = []
        for root in PRODUCTION_UI_ASSET_ROOTS:
            root_path = REPO_ROOT / root
            if not root_path.exists():
                continue
            for zui_asset in root_path.rglob("*.zui"):
                text = zui_asset.read_text(encoding="utf-8")
                for suffix in RETIRED_ACTIVE_SUFFIXES:
                    if suffix in text:
                        failures.append(
                            f"{zui_asset.relative_to(REPO_ROOT)}: {suffix}"
                        )

        if failures:
            self.fail(
                "Production .zui assets still display retired UI suffixes:\n"
                + "\n".join(failures)
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

    def test_editor_ui_asset_editing_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_root in EDITOR_UI_ASSET_EDITING_TEST_PATHS:
            path = REPO_ROOT / relative_root
            files = sorted(path.rglob("*.rs")) if path.is_dir() else [path]
            for rust_file in files:
                text = rust_file.read_text(encoding="utf-8")
                for suffix in RETIRED_ACTIVE_SUFFIXES:
                    if suffix in text:
                        failures.append(
                            f"{rust_file.relative_to(REPO_ROOT)}: {suffix}"
                        )

        if failures:
            self.fail(
                "Editor UI asset editing tests still use retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_editor_host_manager_ui_asset_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_root in EDITOR_HOST_MANAGER_UI_ASSET_TEST_PATHS:
            path = REPO_ROOT / relative_root
            files = sorted(path.rglob("*.rs")) if path.is_dir() else [path]
            for rust_file in files:
                text = rust_file.read_text(encoding="utf-8")
                for suffix in RETIRED_ACTIVE_SUFFIXES:
                    if suffix in text:
                        failures.append(
                            f"{rust_file.relative_to(REPO_ROOT)}: {suffix}"
                        )

        if failures:
            self.fail(
                "Editor host manager UI asset tests still use retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_editor_ui_asset_editor_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_root in EDITOR_UI_ASSET_EDITOR_TEST_PATHS:
            path = REPO_ROOT / relative_root
            files = sorted(path.rglob("*.rs")) if path.is_dir() else [path]
            for rust_file in files:
                relative_path = rust_file.relative_to(REPO_ROOT).as_posix()
                allowed_lines = EDITOR_UI_ASSET_EDITOR_ALLOWED_RETIRED_SUFFIX_LINES.get(
                    relative_path, []
                )
                for line_number, line in enumerate(
                    rust_file.read_text(encoding="utf-8").splitlines(),
                    start=1,
                ):
                    if any(allowed in line for allowed in allowed_lines):
                        continue
                    for suffix in RETIRED_ACTIVE_SUFFIXES:
                        if suffix in line:
                            failures.append(
                                f"{rust_file.relative_to(REPO_ROOT)}:{line_number}: {suffix}"
                            )

        if failures:
            self.fail(
                "Editor UI Asset Editor tests still use retired UI suffixes outside explicit legacy-import guards:\n"
                + "\n".join(failures)
            )

    def test_editor_host_theme_tooling_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_root in EDITOR_HOST_THEME_TOOLING_TEST_PATHS:
            path = REPO_ROOT / relative_root
            files = sorted(path.rglob("*.rs")) if path.is_dir() else [path]
            for rust_file in files:
                text = rust_file.read_text(encoding="utf-8")
                for suffix in RETIRED_ACTIVE_SUFFIXES:
                    if suffix in text:
                        failures.append(
                            f"{rust_file.relative_to(REPO_ROOT)}: {suffix}"
                        )

        if failures:
            self.fail(
                "Editor host theme tooling tests still use retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_runtime_ui_active_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_path in RUNTIME_UI_ACTIVE_TEST_FILES:
            rust_file = REPO_ROOT / relative_path
            text = rust_file.read_text(encoding="utf-8")
            for suffix in RETIRED_ACTIVE_SUFFIXES:
                if suffix in text:
                    failures.append(f"{relative_path}: {suffix}")

        if failures:
            self.fail(
                "Runtime UI active tests still use retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_editor_ui_component_adapter_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_path in EDITOR_UI_COMPONENT_ADAPTER_TEST_FILES:
            rust_file = REPO_ROOT / relative_path
            text = rust_file.read_text(encoding="utf-8")
            for suffix in RETIRED_ACTIVE_SUFFIXES:
                if suffix in text:
                    failures.append(f"{relative_path}: {suffix}")

        if failures:
            self.fail(
                "Editor UI component adapter tests still use retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_editor_retained_host_projection_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_path in EDITOR_RETAINED_HOST_PROJECTION_TEST_FILES:
            rust_file = REPO_ROOT / relative_path
            text = rust_file.read_text(encoding="utf-8")
            for suffix in RETIRED_ACTIVE_SUFFIXES:
                if suffix in text:
                    failures.append(f"{relative_path}: {suffix}")

        if failures:
            self.fail(
                "Editor retained host projection tests still use retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_editor_extension_contract_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_path in EDITOR_EXTENSION_CONTRACT_TEST_FILES:
            rust_file = REPO_ROOT / relative_path
            for line_number, line in enumerate(
                rust_file.read_text(encoding="utf-8").splitlines(),
                start=1,
            ):
                for suffix in RETIRED_ACTIVE_SUFFIXES:
                    if suffix in line:
                        failures.append(f"{relative_path}:{line_number}: {suffix}")

        if failures:
            self.fail(
                "Editor extension contract tests still use retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_editor_view_projection_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_path in EDITOR_VIEW_PROJECTION_TEST_FILES:
            rust_file = REPO_ROOT / relative_path
            for line_number, line in enumerate(
                rust_file.read_text(encoding="utf-8").splitlines(),
                start=1,
            ):
                for suffix in RETIRED_ACTIVE_SUFFIXES:
                    if suffix in line:
                        failures.append(f"{relative_path}:{line_number}: {suffix}")

        if failures:
            self.fail(
                "Editor view projection tests still use retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_runtime_extension_component_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_path in RUNTIME_EXTENSION_COMPONENT_TEST_FILES:
            rust_file = REPO_ROOT / relative_path
            for line_number, line in enumerate(
                rust_file.read_text(encoding="utf-8").splitlines(),
                start=1,
            ):
                for suffix in RETIRED_ACTIVE_SUFFIXES:
                    if suffix in line:
                        failures.append(f"{relative_path}:{line_number}: {suffix}")

        if failures:
            self.fail(
                "Runtime extension component tests still use retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_runtime_asset_ui_reference_tests_use_zui_suffix(self):
        failures: list[str] = []
        for relative_path in RUNTIME_ASSET_UI_REFERENCE_TEST_FILES:
            rust_file = REPO_ROOT / relative_path
            for line_number, line in enumerate(
                rust_file.read_text(encoding="utf-8").splitlines(),
                start=1,
            ):
                for suffix in RETIRED_ACTIVE_SUFFIXES:
                    if suffix in line:
                        failures.append(f"{relative_path}:{line_number}: {suffix}")

        if failures:
            self.fail(
                "Runtime asset UI reference tests still use retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_global_material_surface_assets_collect_zui_view_surfaces(self):
        source = (REPO_ROOT / GLOBAL_MATERIAL_SURFACE_TEST_FILE).read_text(
            encoding="utf-8"
        )
        stale_phrases = [
            'ends_with(".ui.toml")',
            'OsStr::new("toml")',
            "Milestone 3 inventory changed",
        ]
        required_phrases = [
            'OsStr::new("zui")',
            'zui_asset_kind(&document) == Some("view")',
            "Current .zui view surface inventory changed",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in source:
                failures.append(f"stale global material surface collector: {phrase}")
        for phrase in required_phrases:
            if phrase not in source:
                failures.append(f"missing zui view surface collector marker: {phrase}")

        if failures:
            self.fail(
                "Global Material surface asset conformance must collect current .zui view surfaces:\n"
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
