from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
PRESENTATION_DIR = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "asset_editor"
    / "session"
    / "presentation"
)
PANE = PRESENTATION_DIR / "pane.rs"
PREVIEW = PRESENTATION_DIR / "preview.rs"
REFLECTION = PRESENTATION_DIR / "reflection.rs"
SOURCE = PRESENTATION_DIR / "source.rs"
INSPECTOR = PRESENTATION_DIR / "inspector.rs"
STYLE = PRESENTATION_DIR / "style.rs"
THEME = PRESENTATION_DIR / "theme.rs"
COMMANDS = PRESENTATION_DIR / "commands.rs"
PRESENTATION_MOD = PRESENTATION_DIR / "mod.rs"
SESSION_MOD = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "asset_editor"
    / "session"
    / "mod.rs"
)
UI_PERF = ROOT / "zircon_editor" / "src" / "ui" / "retained_host" / "ui_perf.rs"
UI_HOTSPOT = (
    ROOT
    / "zircon_runtime"
    / "src"
    / "core"
    / "runtime"
    / "diagnostics"
    / "profiling"
    / "ui_hotspot.rs"
)
PROFILE_INTERFACE = ROOT / "zircon_runtime_interface" / "src" / "profiling.rs"
SOURCE_OUTLINE_CACHE = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "asset_editor"
    / "session"
    / "lifecycle"
    / "source_outline_cache.rs"
)
NAVIGATION_STATE = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "asset_editor"
    / "session"
    / "navigation_state.rs"
)


def function_body(source: str, signature: str) -> str:
    function_start = source.index(signature)
    body_start = source.index("{", function_start)
    depth = 0
    for offset, character in enumerate(source[body_start:], start=body_start):
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[body_start : offset + 1]
    raise AssertionError(f"unterminated function: {signature}")


def profile_scope_block(source: str, span: str, occurrence: int = 0) -> str:
    marker = f'"asset_editor.presentation", "{span}",'
    search_start = 0
    for _ in range(occurrence + 1):
        scope_start = source.index(marker, search_start)
        search_start = scope_start + len(marker)
    block_start = source.rfind("{", 0, scope_start)
    depth = 0
    for offset, character in enumerate(source[block_start:], start=block_start):
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[block_start : offset + 1]
    raise AssertionError(f"unterminated profile scope: {span}")


class UiAssetPaneProfileContractTests(unittest.TestCase):
    def test_presentation_is_folder_backed_without_a_compatibility_facade(self) -> None:
        session_mod = SESSION_MOD.read_text(encoding="utf-8")
        presentation_mod = PRESENTATION_MOD.read_text(encoding="utf-8")

        self.assertTrue(PRESENTATION_DIR.is_dir())
        self.assertFalse((PRESENTATION_DIR.parent / "presentation_state.rs").exists())
        self.assertFalse((PRESENTATION_DIR / "assembly.rs").exists())
        self.assertIn("pub(crate) mod presentation;", session_mod)
        self.assertNotIn("presentation_state", session_mod)
        for module in (PANE, PREVIEW, REFLECTION, SOURCE, INSPECTOR, STYLE, THEME, COMMANDS):
            self.assertTrue(module.is_file(), module)
            self.assertIn(f"mod {module.stem};", presentation_mod)

    def test_pane_projection_enters_the_shared_profile_before_building_artifacts(self) -> None:
        source = PANE.read_text(encoding="utf-8")
        ui_perf = UI_PERF.read_text(encoding="utf-8")
        ui_hotspot = UI_HOTSPOT.read_text(encoding="utf-8")
        profile_interface = PROFILE_INTERFACE.read_text(encoding="utf-8")
        body = function_body(source, "    pub fn pane_presentation(&self)")

        profile_scope = (
            'zircon_runtime::profile_scope!("editor", "asset_editor.presentation", '
            '"pane_presentation",);'
        )
        counter = "UiPerfCounter::AssetEditorPanePresentationBuildCount"
        first_artifact = "let reflection = self.reflection_pane_presentation();"
        profiling_prefix = body[: body.index(first_artifact)]

        self.assertIn(profile_scope, body)
        self.assertIn("record_current_ui_perf_counter(", profiling_prefix)
        self.assertIn(counter, profiling_prefix)
        self.assertLess(body.index(profile_scope), body.index(first_artifact))
        self.assertLess(body.index(counter), body.index(first_artifact))
        self.assertNotIn("asset_editor_profile_counter", source)
        self.assertNotIn("UiPerfCounter::PresentationRebuildCount", body)
        self.assertIn("AssetEditorPanePresentationBuildCount", ui_perf)
        self.assertIn("asset_editor_pane_presentation_build_count", ui_perf)
        self.assertIn("asset_editor_pane_presentation_build_count", ui_hotspot)
        self.assertIn("asset_editor_pane_presentation_build_count", profile_interface)

    def test_pane_domains_are_individually_attributable_in_the_shared_profile(self) -> None:
        source = PANE.read_text(encoding="utf-8")
        ui_perf = UI_PERF.read_text(encoding="utf-8")
        ui_hotspot = UI_HOTSPOT.read_text(encoding="utf-8")
        profile_interface = PROFILE_INTERFACE.read_text(encoding="utf-8")
        source_outline_cache = SOURCE_OUTLINE_CACHE.read_text(encoding="utf-8")
        navigation_state = NAVIGATION_STATE.read_text(encoding="utf-8")
        body = function_body(source, "    pub fn pane_presentation(&self)")

        domains = {
            "reflection": (
                "AssetEditorPaneReflectionBuildCount",
                "asset_editor_pane_reflection_build_count",
            ),
            "preview": (
                "AssetEditorPanePreviewBuildCount",
                "asset_editor_pane_preview_build_count",
            ),
            "inspector": (
                "AssetEditorPaneInspectorBuildCount",
                "asset_editor_pane_inspector_build_count",
            ),
            "style": (
                "AssetEditorPaneStyleBuildCount",
                "asset_editor_pane_style_build_count",
            ),
            "theme": (
                "AssetEditorPaneThemeBuildCount",
                "asset_editor_pane_theme_build_count",
            ),
            "command_availability": (
                "AssetEditorPaneCommandAvailabilityBuildCount",
                "asset_editor_pane_command_availability_build_count",
            ),
        }

        domain_sources = {
            "reflection": REFLECTION.read_text(encoding="utf-8"),
            "preview": PREVIEW.read_text(encoding="utf-8"),
            "inspector": INSPECTOR.read_text(encoding="utf-8"),
            "style": STYLE.read_text(encoding="utf-8"),
            "theme": THEME.read_text(encoding="utf-8"),
            "command_availability": COMMANDS.read_text(encoding="utf-8"),
        }

        for span, (counter, metric) in domains.items():
            domain_source = domain_sources[span]
            self.assertRegex(
                domain_source,
                re.compile(
                    r'zircon_runtime::profile_scope!\(\s*"editor",\s*'
                    r'"asset_editor\.presentation",\s*'
                    + re.escape(f'"{span}",')
                    + r'\s*\);'
                ),
            )
            self.assertIn(f"UiPerfCounter::{counter}", domain_source)
            self.assertIn(counter, ui_perf)
            self.assertIn(metric, ui_perf)
            self.assertIn(metric, ui_hotspot)
            self.assertIn(metric, profile_interface)

        source_counter = "UiPerfCounter::AssetEditorPaneSourceBuildCount"
        self.assertIn(source_counter, source_outline_cache)
        self.assertEqual(source_outline_cache.count(source_counter), 2)
        self.assertEqual(navigation_state.count(source_counter), 2)
        for build_owner in (source_outline_cache, navigation_state):
            build_offsets = [
                offset
                for offset in range(len(build_owner))
                if build_owner.startswith("build_source_outline_index(", offset)
            ]
            counter_offsets = [
                offset
                for offset in range(len(build_owner))
                if build_owner.startswith(source_counter, offset)
            ]
            self.assertEqual(len(build_offsets), 2)
            self.assertEqual(len(counter_offsets), 2)
            for build_offset, counter_offset in zip(build_offsets, counter_offsets):
                self.assertLess(build_offset, counter_offset)
        self.assertNotIn(source_counter, body)
        self.assertNotIn(source_counter, SOURCE.read_text(encoding="utf-8"))
        self.assertIn("asset_editor_pane_source_build_count", ui_perf)
        self.assertIn("asset_editor_pane_source_build_count", ui_hotspot)
        self.assertIn("asset_editor_pane_source_build_count", profile_interface)

        preview_source = domain_sources["preview"]
        self.assertIn("fn preview_pane_presentation", preview_source)
        self.assertIn("let palette_drag_slot_target_items", preview_source)
        self.assertIn("let palette_drag_candidate_items", preview_source)

        inspector_artifacts_block = domain_sources["inspector"]
        self.assertIn("preview_mock_fields = build_preview_mock_fields", inspector_artifacts_block)
        self.assertIn("binding_fields = build_binding_fields", inspector_artifacts_block)
        self.assertIn("layout_semantic_selected_index", inspector_artifacts_block)

        inspector_rows_block = domain_sources["inspector"]
        self.assertIn("build_selected_node_prop_state_items", inspector_rows_block)
        self.assertIn("build_component_contract_items", inspector_rows_block)

        style_artifacts_block = domain_sources["style"]
        self.assertIn("selected_node_selector", style_artifacts_block)
        self.assertIn("build_stylesheet_items", style_artifacts_block)
        self.assertNotIn("selected_node_selector(", body)
        self.assertNotIn("build_stylesheet_items(", body)


if __name__ == "__main__":
    unittest.main()
