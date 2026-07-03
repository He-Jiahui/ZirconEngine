from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
SLUG = "plugins_09_e1_library_embed_compile_host_manifest_path_command_gate"
REQUIRED_MARKERS = (
    SLUG,
    "library_embed_compile_plan.rs",
    "test_runtime_export_library_embed_command_policy.py",
    "--manifest-path",
)


def _section(text: str, start_marker: str, end_marker: str | None = None) -> str:
    start = text.index(start_marker)
    if end_marker is None:
        return text[start:]
    return text[start : text.index(end_marker, start + len(start_marker))]


class PluginDocsCurrentStatusLibraryEmbedManifestPathCommandGateTests(
    unittest.TestCase
):
    def test_current_status_records_library_embed_manifest_path_command_gate(
        self,
    ) -> None:
        export_plan = (
            REPO_ROOT / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        standalone_plan = (
            REPO_ROOT / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_doc = (
            REPO_ROOT / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        export_tool = (
            REPO_ROOT / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        structure = (
            REPO_ROOT / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        review = (
            REPO_ROOT / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")
        session = (
            REPO_ROOT
            / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        sections = {
            "09 export status": _section(
                export_plan,
                "## 状态与产出记录",
                "## 5. 里程碑与任务分解",
            ),
            "13 standalone status": _section(
                standalone_plan,
                "## 9. 审查和验收记录",
            ),
            "standalone current status": _section(
                standalone_doc,
                "## 9. 当前落地状态",
            ),
            "export tooling docs": export_tool,
            "structure convention": structure,
            "review findings": review,
            "active session notes": session,
        }

        failures: list[str] = []
        for section_name, section in sections.items():
            for marker in REQUIRED_MARKERS:
                if marker not in section:
                    failures.append(f"{section_name}: missing {marker}")

        if failures:
            self.fail(
                "Current plugin docs do not record LibraryEmbed manifest-path "
                "command gate:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
