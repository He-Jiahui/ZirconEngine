import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusHybridGiRenderLayerSchemaV1MaskCallerTests(
    unittest.TestCase
):
    def test_current_status_records_hybrid_gi_render_layer_mask_api_convergence(
        self,
    ) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        status_id = (
            "plugins_13_m5_t1_hybrid_gi_render_layer_schema_v1_mask_callers"
        )

        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        camera_text = (
            repo_root / "docs/zircon_runtime/core/framework/render/camera.md"
        ).read_text(encoding="utf-8")
        hybrid_gi_text = (
            repo_root
            / "docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md"
        ).read_text(encoding="utf-8")
        structure_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        review_text = (
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
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
            "camera docs": camera_text,
            "hybrid gi docs": hybrid_gi_text,
            "structure convention": structure_text,
            "review findings": review_text,
            "active session": session_text,
        }
        required_phrases = [
            status_id,
            "RenderLayerSet::from_scene_schema_v1_mask",
            "from_extract_mask",
            "zircon_plugin_hybrid_gi_runtime",
            "cargo check --manifest-path zircon_plugins\\Cargo.toml -p zircon_plugin_hybrid_gi_runtime",
            "test_plugin_render_layer_schema_v1_mask_callers",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record Hybrid GI render-layer schema-v1 mask convergence:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
