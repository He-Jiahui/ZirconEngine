import unittest
from pathlib import Path


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusAuditTargetCountConvergenceTests(unittest.TestCase):
    def test_current_status_records_39_dist_targets_audit_count(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_13_m5_t1_structure_audit_dist_target_count_converged"

        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
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
            "Plugins 13 status": _tail_section(
                plan_13_text, "## 9. 审查和验收记录"
            ),
            "Plugins 09 status": _tail_section(
                plan_09_text, "## 状态与产出记录"
            ),
            "standalone current status": _tail_section(
                standalone_text, "## 9. 当前落地状态"
            ),
            "structure convention": structure_text,
            "review findings": review_text,
            "active session": session_text,
        }
        required_phrases = [
            status_id,
            "37 个 root dist-capable plugin + 2 个 Sound feature-provider distribution targets",
            "dist_capable_plugin_count=39",
            "dist_build_matrix_count=39",
            "plugin validate --all",
            "target_count=39",
            "failed_count=0",
            "diagnostics=0",
            "历史 37/37 记录只表示 root plugin rollout 快照",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin status docs do not record the 39-target "
                "structure-audit count:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
