import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_ID = "editor_ui_11_m5_global_material_surface_zui_view_inventory_guard_passed"


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


class ZuiDocsGlobalMaterialSurfaceGuardTests(unittest.TestCase):
    def test_current_material_surface_docs_use_zui_view_inventory(self):
        showcase_text = (
            REPO_ROOT / "docs/ui-and-layout/runtime-ui-component-showcase.md"
        ).read_text(encoding="utf-8")
        shared_template_text = (
            REPO_ROOT / "docs/ui-and-layout/shared-ui-template-runtime.md"
        ).read_text(encoding="utf-8")

        sections = {
            "runtime-ui-component-showcase": _section(
                showcase_text,
                "## Global Material Surface Conformance",
                "## Host Contract Boundary",
            ),
            "shared-ui-template-runtime": _section(
                shared_template_text,
                "2026-05-07 Global UI Material M4",
                "2026-05-07 M6 text closure",
            ),
        }

        stale_phrases = [
            "repository-wide `.ui.toml` asset rule",
            "54-file global `.ui.toml` inventory",
            "`.ui.toml` template/runtime path",
            "console.ui.toml",
            "module_plugins_body.ui.toml",
            "runtime_diagnostics_body.ui.toml",
            "welcome.ui.toml",
            "editor_material.ui.toml",
            "editor_base.ui.toml",
            "material_meta_components.ui.toml",
        ]
        required_phrases = [
            "41-file global `.zui` view surface inventory",
            "component `.zui` libraries and theme/style/token `.zui` documents remain import-graph inputs",
            "global Material surface guard now collects `.zui` view documents by `asset.kind = \"view\"`",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in stale_phrases:
                if phrase in section:
                    failures.append(f"{section_name}: stale phrase {phrase}")
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current global Material surface docs must describe .zui view inventory:\n"
                + "\n".join(failures)
            )

    def test_plan_and_review_status_record_global_material_surface_zui_guard(self):
        plan_text = (
            REPO_ROOT
            / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
        ).read_text(encoding="utf-8")
        review_text = (
            REPO_ROOT / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")

        required_phrases = [
            STATUS_ID,
            "global Material surface guard now collects `.zui` view documents",
            "41-file global `.zui` view surface inventory",
        ]

        failures: list[str] = []
        for document_name, text in {
            "Plan 11": plan_text,
            "engine-code-review-findings": review_text,
        }.items():
            for phrase in required_phrases:
                if phrase not in text:
                    failures.append(f"{document_name}: missing {phrase}")

        if failures:
            self.fail(
                "Plan/review status must record the global Material surface .zui guard:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
