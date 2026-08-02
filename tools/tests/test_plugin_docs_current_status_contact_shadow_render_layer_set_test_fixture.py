import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusContactShadowRenderLayerSetTests(unittest.TestCase):
    def test_current_status_records_contact_shadow_render_layer_set_fixture(
        self,
    ) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_13_m5_t1_contact_shadow_render_layer_set_test_fixture"

        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        render_framework_text = (
            repo_root / "docs/assets-and-rendering/render-framework-architecture.md"
        ).read_text(encoding="utf-8")
        structure_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        review_text = (
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")

        sections = {
            "Plugins 09 status": _section(
                plan_09_text, "## 状态与产出记录", "## 5. 里程碑与任务分解"
            ),
            "Plugins 13 status": _tail_section(
                plan_13_text, "## 9. 审查和验收记录"
            ),
            "standalone current status": _tail_section(
                standalone_text, "## 9. 当前落地状态"
            ),
            "render framework docs": render_framework_text,
            "structure convention": structure_text,
            "review findings": review_text,
        }
        required_phrases = [
            status_id,
            "RenderLayerSet",
            "default_render_layer_set",
            "DEFAULT_RENDER_LAYER_MASK",
            "zircon_plugin_rendering_contact_shadow_runtime",
            "test_contact_shadow_render_layer_set_test_fixture",
            "cargo check --manifest-path zircon_plugins\\Cargo.toml -p zircon_plugin_rendering_contact_shadow_runtime --locked --all-targets",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record contact-shadow RenderLayerSet test fixture support:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
