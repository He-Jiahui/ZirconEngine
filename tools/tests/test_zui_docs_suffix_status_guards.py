import unittest
from pathlib import Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


class ZuiDocsSuffixStatusGuardTests(unittest.TestCase):
    def test_structure_and_review_status_include_latest_zui_authority_guards(self):
        repo_root = Path(__file__).resolve().parents[2]
        structure_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        review_text = (
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")
        status_sections = {
            "engine-code-structure-convention": _section(
                structure_text,
                "## 2026-06-28 Plan 11 layout metadata `.zui` reference guard 结构补记",
                "## 2026-06-28 Plan 08 Non-Base mesh variant-aware cache owner 结构补记",
            ),
            "engine-code-review-findings": _section(
                review_text,
                "## 2026-06-28 Plan 11 layout metadata `.zui` reference guard 审查补记",
                "## 2026-06-28 Plan 08 Non-Base mesh variant-aware cache owner 审查补记",
            ),
        }

        required_status_anchors = [
            "editor_ui_11_m5_workbench_shell_zui_authority_guard_passed",
            "editor_ui_11_m5_plugin_export_wizard_zui_report_template_guard_passed",
            "editor_ui_11_m5_editor_command_workflow_component_drawer_zui_guard_passed",
            "editor_ui_11_m5_editor_workbench_shell_zui_host_asset_guard_passed",
            "editor_ui_11_m5_editor_build_export_desktop_zui_doc_guard_passed",
        ]

        failures: list[str] = []
        for section_name, section in status_sections.items():
            for anchor in required_status_anchors:
                if anchor not in section:
                    failures.append(f"{section_name}: missing {anchor}")

        if failures:
            self.fail(
                "Structure/review status docs are missing latest .zui authority guard anchors:\n"
                + "\n".join(failures)
            )

    def test_production_zui_asset_text_suffix_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_production_zui_asset_text_suffix_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "Asset Browser docs": (
                repo_root / "docs/zircon_editor/ui/layouts/views/asset_browser.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_production_zui_assets_do_not_display_retired_suffixes",
            "zircon_editor/assets/ui/editor/asset_browser.zui",
            "workbench_page_chrome.zui",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Production .zui asset text suffix guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_editor_ui_asset_editing_fixture_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_ui_asset_editing_test_fixture_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "UI Asset Editor host session docs": (
                repo_root / "docs/editor-and-tooling/ui-asset-editor-host-session.md"
            ).read_text(encoding="utf-8"),
            "UI asset protocol docs": (
                repo_root / "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_editor_ui_asset_editing_tests_use_zui_suffix",
            "zircon_editor/src/tests/editing/ui_asset",
            "zircon_editor/src/tests/editing/ui_asset_replay.rs",
            "zircon_editor/src/tests/editing/ui_asset_theme_authoring.rs",
            "301 retired suffix references",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Editor UI asset editing fixture .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_editor_host_manager_ui_asset_fixture_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_host_manager_ui_asset_fixture_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "UI Asset Editor host session docs": (
                repo_root / "docs/editor-and-tooling/ui-asset-editor-host-session.md"
            ).read_text(encoding="utf-8"),
            "UI asset protocol docs": (
                repo_root / "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_editor_host_manager_ui_asset_tests_use_zui_suffix",
            "zircon_editor/src/tests/host/manager",
            "zircon_editor/src/tests/host/manager/ui_asset_reference_and_promotion.rs",
            "zircon_editor/src/tests/host/manager/ui_asset_session_preview.rs",
            "zircon_editor/src/tests/host/manager/ui_asset_style_and_inspector.rs",
            "94 retired suffix references",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Editor host manager UI asset fixture .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_editor_ui_asset_editor_ui_tests_fixture_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_ui_asset_editor_ui_tests_fixture_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "UI Asset Editor host session docs": (
                repo_root / "docs/editor-and-tooling/ui-asset-editor-host-session.md"
            ).read_text(encoding="utf-8"),
            "UI asset protocol docs": (
                repo_root / "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_editor_ui_asset_editor_tests_use_zui_suffix",
            "zircon_editor/src/tests/ui/ui_asset_editor",
            "zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs",
            "zircon_editor/src/tests/ui/ui_asset_editor/resource_dependency_view.rs",
            "zircon_editor/src/tests/ui/ui_asset_editor/action_localization_reports.rs",
            "26 retired suffix references",
            "editor_widgets.ui.toml",
            "editor_base.ui.toml",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Editor UI Asset Editor UI tests fixture .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_editor_host_theme_tooling_fixture_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_host_theme_tooling_fixture_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "UI Asset Editor host session docs": (
                repo_root / "docs/editor-and-tooling/ui-asset-editor-host-session.md"
            ).read_text(encoding="utf-8"),
            "UI asset protocol docs": (
                repo_root / "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_editor_host_theme_tooling_tests_use_zui_suffix",
            "zircon_editor/src/tests/host/ui_asset_editor_theme_tooling",
            "zircon_editor/src/tests/host/ui_asset_editor_theme_tooling/batch_helpers.rs",
            "zircon_editor/src/tests/host/ui_asset_editor_theme_tooling/support.rs",
            "15 retired suffix references",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Editor host theme tooling fixture .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_runtime_ui_prototype_store_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_runtime_ui_prototype_store_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "UI asset protocol docs": (
                repo_root / "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_runtime_ui_active_tests_use_zui_suffix",
            "zircon_runtime/src/ui/tests/asset_prototype_store.rs",
            "14 retired suffix references",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Runtime UI prototype store .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_editor_ui_component_adapter_fixture_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_component_adapter_fixture_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "UI Asset Editor host session docs": (
                repo_root / "docs/editor-and-tooling/ui-asset-editor-host-session.md"
            ).read_text(encoding="utf-8"),
            "UI asset protocol docs": (
                repo_root / "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_editor_ui_component_adapter_tests_use_zui_suffix",
            "zircon_editor/src/tests/ui/component_adapter.rs",
            "3 retired suffix references",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Editor UI component adapter fixture .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_editor_retained_host_projection_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_retained_host_projection_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "UI Asset Editor host session docs": (
                repo_root / "docs/editor-and-tooling/ui-asset-editor-host-session.md"
            ).read_text(encoding="utf-8"),
            "UI asset protocol docs": (
                repo_root / "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_editor_retained_host_projection_tests_use_zui_suffix",
            "zircon_editor/src/tests/host/retained_window/native_host_contract.rs",
            "zircon_editor/src/ui/retained_host/ui/tests/host_scene_projection.rs",
            "4 retired suffix references",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Editor retained host projection .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_editor_extension_contract_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_editor_extension_contract_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "Editor command workflow docs": (
                repo_root / "docs/editor-and-tooling/editor-command-workflow.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_editor_extension_contract_tests_use_zui_suffix",
            "zircon_editor/src/tests/editor_authoring_extension_descriptors.rs",
            "zircon_editor/src/tests/editor_event/runtime.rs",
            "register_ui_template",
            "register_component_drawer",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Editor extension contract .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_editor_view_projection_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_view_projection_test_fixture_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "UI asset protocol docs": (
                repo_root / "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_editor_view_projection_tests_use_zui_suffix",
            "zircon_editor/src/ui/layouts/views/view_projection/tests.rs",
            "view_template_projection_rejects_non_zui_asset_paths",
            "ViewTemplateProjectionError::NonV2AssetPath",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Editor view projection .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_runtime_extension_component_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_runtime_extension_component_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "runtime/editor pluginized export docs": (
                repo_root / "docs/engine-architecture/runtime-editor-pluginized-export.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_runtime_extension_component_tests_use_zui_suffix",
            "zircon_runtime/src/tests/plugin_extensions/extension_registry_components.rs",
            "runtime_extension_registry_rejects_non_zui_ui_component_documents",
            "UiComponentDescriptor::new",
            "register_ui_component",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Runtime extension component .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

    def test_runtime_asset_ui_reference_zui_guard_status_is_recorded(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "editor_ui_11_m5_runtime_asset_ui_reference_fixture_zui_guard_passed"
        sections = {
            "Plan 11 status": (
                repo_root
                / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
            ).read_text(encoding="utf-8"),
            "runtime UI asset docs": (
                repo_root / "docs/zircon_runtime/asset/assets/ui.md"
            ).read_text(encoding="utf-8"),
            "structure convention": (
                repo_root / "docs/plans/engine-code-structure-convention.md"
            ).read_text(encoding="utf-8"),
            "review findings": (
                repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
            ).read_text(encoding="utf-8"),
            "active session": (
                repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
            ).read_text(encoding="utf-8"),
        }
        required_phrases = [
            status_id,
            "test_runtime_asset_ui_reference_tests_use_zui_suffix",
            "zircon_runtime/src/asset/tests/assets/ui.rs",
            "zircon_runtime/src/asset/tests/assets/ui/references.rs",
            "ui_asset_direct_references_include_collected_resource_dependencies",
            "ui_v2_asset_direct_references_include_imports_and_resources",
            "ui_asset_references",
            "ui_v2_asset_references",
            "retired `.ui.toml` / `.v2.ui.toml` suffixes",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Runtime asset UI reference .zui guard status is incomplete:\n"
                + "\n".join(failures)
            )

if __name__ == "__main__":
    unittest.main()
