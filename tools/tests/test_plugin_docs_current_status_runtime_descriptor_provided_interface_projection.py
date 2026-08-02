import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusRuntimeDescriptorProvidedInterfaceProjectionTests(
    unittest.TestCase
):
    def test_current_status_records_runtime_descriptor_interface_projection(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        status_id = (
            "plugins_13_m5_t1_runtime_descriptor_provided_interface_projection"
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
        package_manifest_text = (
            repo_root / "docs/zircon_runtime/plugin/package_manifest.md"
        ).read_text(encoding="utf-8")
        physics_text = (
            repo_root / "docs/zircon_plugins/physics/runtime.md"
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
            "runtime package manifest docs": package_manifest_text,
            "physics runtime docs": physics_text,
            "structure convention": structure_text,
            "review findings": review_text,
        }
        required_phrases = [
            status_id,
            "RuntimePluginDescriptorBuilder::with_provided_interface_id",
            "RuntimePluginDescriptor::package_manifest()",
            "provided_interfaces",
            "PHYSICS_QUERY_INTERFACE_ID",
            "zircon_plugin_physics_runtime",
            "runtime_plugin_descriptor_projects_public_metadata_to_package_manifest",
            "cargo check --manifest-path zircon_plugins\\Cargo.toml -p zircon_plugin_physics_runtime",
            "Rust 单测两次超时未采信",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record runtime descriptor provided-interface projection:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
