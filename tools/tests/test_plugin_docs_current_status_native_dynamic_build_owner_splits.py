import unittest
from pathlib import Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


class PluginDocsCurrentStatusNativeDynamicBuildOwnerSplitTests(unittest.TestCase):
    def test_current_export_plan_reflects_native_dynamic_materialize_owner_split(self):
        repo_root = Path(__file__).resolve().parents[2]

        export_plan_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        export_plan_status = _section(
            export_plan_text,
            "## 状态与产出记录",
            "## 5. 里程碑与任务分解",
        )

        standalone_plan_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_plan_status = standalone_plan_text[
            standalone_plan_text.index("## 9. 审查和验收记录") :
        ]

        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_current_contract_section = _section(
            standalone_text,
            "## 6. 注册跨 ABI 编组",
            "## 9. 当前落地状态",
        )

        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root
            / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        required_by_section = {
            "09 export status": [
                "plugins_13_m5_t1_native_dynamic_materialize_owner_split",
                "native_dynamic_materialize.py",
                "NativeDynamic materialization owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_native_dynamic_materialize_owner_split",
                "native_dynamic_materialize.py",
                "NativeDynamic materialization owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_native_dynamic_materialize_owner_split",
                "native_dynamic_materialize.py",
                "NativeDynamic materialization owner",
            ],
            "export tooling docs": [
                "native_dynamic_materialize.py",
                "NativeDynamic materialization owner",
                "package copy/finalization helpers",
            ],
            "active session notes": [
                "plugins_13_m5_t1_native_dynamic_materialize_owner_split",
                "native_dynamic_materialize.py",
                "NativeDynamic materialization owner",
            ],
        }
        sections = {
            "09 export status": export_plan_status,
            "13 standalone status": standalone_plan_status,
            "standalone current contract": standalone_current_contract_section,
            "export tooling docs": export_tool_text,
            "active session notes": session_text,
        }

        failures: list[str] = []
        for section_name, required_phrases in required_by_section.items():
            section = sections[section_name]
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current export/plugin docs do not reflect NativeDynamic materialize owner split:\n"
                + "\n".join(failures)
            )

    def test_current_export_plan_reflects_native_build_workspace_owner_split(
        self,
    ):
        repo_root = Path(__file__).resolve().parents[2]

        export_plan_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        export_plan_status = _section(
            export_plan_text,
            "## 状态与产出记录",
            "## 5. 里程碑与任务分解",
        )

        standalone_plan_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_plan_status = standalone_plan_text[
            standalone_plan_text.index("## 9. 审查和验收记录") :
        ]

        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_current_contract_section = _section(
            standalone_text,
            "## 6. 注册跨 ABI 编组",
            "## 9. 当前落地状态",
        )

        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root
            / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        required_by_section = {
            "09 export status": [
                "plugins_13_m5_t1_native_build_workspace_owner_split",
                "native_build_workspace.py",
                "NativeBuild workspace metadata owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_native_build_workspace_owner_split",
                "native_build_workspace.py",
                "NativeBuild workspace metadata owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_native_build_workspace_owner_split",
                "native_build_workspace.py",
                "NativeBuild TOML/workspace crate diagnostics",
            ],
            "export tooling docs": [
                "native_build_workspace.py",
                "NativeBuild workspace metadata owner",
                "NativeBuild TOML/workspace crate diagnostics",
            ],
            "active session notes": [
                "plugins_13_m5_t1_native_build_workspace_owner_split",
                "native_build_workspace.py",
                "NativeBuild workspace metadata owner",
            ],
        }
        sections = {
            "09 export status": export_plan_status,
            "13 standalone status": standalone_plan_status,
            "standalone current contract": standalone_current_contract_section,
            "export tooling docs": export_tool_text,
            "active session notes": session_text,
        }

        failures: list[str] = []
        for section_name, required_phrases in required_by_section.items():
            section = sections[section_name]
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current export/plugin docs do not reflect NativeBuild "
                "workspace metadata owner split:\n"
                + "\n".join(failures)
            )

    def test_current_export_plan_reflects_native_build_cargo_command_owner_split(
        self,
    ):
        repo_root = Path(__file__).resolve().parents[2]

        export_plan_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        export_plan_status = _section(
            export_plan_text,
            "## 状态与产出记录",
            "## 5. 里程碑与任务分解",
        )

        standalone_plan_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_plan_status = standalone_plan_text[
            standalone_plan_text.index("## 9. 审查和验收记录") :
        ]

        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_current_contract_section = _section(
            standalone_text,
            "## 6. 注册跨 ABI 编组",
            "## 9. 当前落地状态",
        )

        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root
            / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        required_by_section = {
            "09 export status": [
                "plugins_13_m5_t1_native_build_cargo_command_owner_split",
                "native_build_command.py",
                "NativeBuild Cargo command owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_native_build_cargo_command_owner_split",
                "native_build_command.py",
                "NativeBuild Cargo command owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_native_build_cargo_command_owner_split",
                "native_build_command.py",
                "Cargo profile/features/command/artifact naming",
            ],
            "export tooling docs": [
                "native_build_command.py",
                "NativeBuild Cargo command owner",
                "Cargo profile/features/command/artifact naming",
            ],
            "active session notes": [
                "plugins_13_m5_t1_native_build_cargo_command_owner_split",
                "native_build_command.py",
                "NativeBuild Cargo command owner",
            ],
        }
        sections = {
            "09 export status": export_plan_status,
            "13 standalone status": standalone_plan_status,
            "standalone current contract": standalone_current_contract_section,
            "export tooling docs": export_tool_text,
            "active session notes": session_text,
        }

        failures: list[str] = []
        for section_name, required_phrases in required_by_section.items():
            section = sections[section_name]
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current export/plugin docs do not reflect NativeBuild "
                "Cargo command owner split:\n"
                + "\n".join(failures)
            )

    def test_current_export_plan_reflects_native_dynamic_cli_options_owner_split(
        self,
    ):
        repo_root = Path(__file__).resolve().parents[2]

        export_plan_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        export_plan_status = _section(
            export_plan_text,
            "## 状态与产出记录",
            "## 5. 里程碑与任务分解",
        )

        standalone_plan_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_plan_status = standalone_plan_text[
            standalone_plan_text.index("## 9. 审查和验收记录") :
        ]

        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_current_contract_section = _section(
            standalone_text,
            "## 6. 注册跨 ABI 编组",
            "## 9. 当前落地状态",
        )

        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root
            / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        required_by_section = {
            "09 export status": [
                "plugins_13_m5_t1_native_dynamic_cli_options_owner_split",
                "native_dynamic_cli_options.py",
                "NativeDynamic CLI options owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_native_dynamic_cli_options_owner_split",
                "native_dynamic_cli_options.py",
                "NativeDynamic CLI options owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_native_dynamic_cli_options_owner_split",
                "native_dynamic_cli_options.py",
                "CLI option normalization for build/signing/notarization",
            ],
            "export tooling docs": [
                "native_dynamic_cli_options.py",
                "NativeDynamic CLI options owner",
                "CLI option normalization for build/signing/notarization",
            ],
            "active session notes": [
                "plugins_13_m5_t1_native_dynamic_cli_options_owner_split",
                "native_dynamic_cli_options.py",
                "NativeDynamic CLI options owner",
            ],
        }
        sections = {
            "09 export status": export_plan_status,
            "13 standalone status": standalone_plan_status,
            "standalone current contract": standalone_current_contract_section,
            "export tooling docs": export_tool_text,
            "active session notes": session_text,
        }

        failures: list[str] = []
        for section_name, required_phrases in required_by_section.items():
            section = sections[section_name]
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current export/plugin docs do not reflect NativeDynamic "
                "CLI options owner split:\n"
                + "\n".join(failures)
            )

    def test_current_export_plan_reflects_native_dynamic_materialize_io_owner_split(
        self,
    ):
        repo_root = Path(__file__).resolve().parents[2]

        export_plan_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        export_plan_status = _section(
            export_plan_text,
            "## 状态与产出记录",
            "## 5. 里程碑与任务分解",
        )

        standalone_plan_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_plan_status = standalone_plan_text[
            standalone_plan_text.index("## 9. 审查和验收记录") :
        ]

        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_current_contract_section = _section(
            standalone_text,
            "## 6. 注册跨 ABI 编组",
            "## 9. 当前落地状态",
        )

        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root
            / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        required_by_section = {
            "09 export status": [
                "plugins_13_m5_t1_native_dynamic_materialize_io_owner_split",
                "native_dynamic_materialize_io.py",
                "NativeDynamic materialization IO/path owner",
            ],
            "13 standalone status": [
                "plugins_13_m5_t1_native_dynamic_materialize_io_owner_split",
                "native_dynamic_materialize_io.py",
                "NativeDynamic materialization IO/path owner",
            ],
            "standalone current contract": [
                "plugins_13_m5_t1_native_dynamic_materialize_io_owner_split",
                "native_dynamic_materialize_io.py",
                "NativeDynamic materialization IO/path owner",
            ],
            "export tooling docs": [
                "native_dynamic_materialize_io.py",
                "NativeDynamic materialization IO/path owner",
                "directory reset/list/remove, file/tree copy, and stage-child path resolution",
            ],
            "active session notes": [
                "plugins_13_m5_t1_native_dynamic_materialize_io_owner_split",
                "native_dynamic_materialize_io.py",
                "NativeDynamic materialization IO/path owner",
            ],
        }
        sections = {
            "09 export status": export_plan_status,
            "13 standalone status": standalone_plan_status,
            "standalone current contract": standalone_current_contract_section,
            "export tooling docs": export_tool_text,
            "active session notes": session_text,
        }

        failures: list[str] = []
        for section_name, required_phrases in required_by_section.items():
            section = sections[section_name]
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current export/plugin docs do not reflect NativeDynamic "
                "materialization IO/path owner split:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
