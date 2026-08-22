import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.non_network_server_naming import (  # noqa: E402
    non_network_server_references,
)


class NonNetworkServerNamingTests(unittest.TestCase):
    def test_test_owned_sources_are_reported_but_not_migration_debt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = (
                root
                / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/render_stats.rs"
            )
            source.parent.mkdir(parents=True)
            source.write_text(
                "fn fixture() {\n    let warm_server = ();\n}\n",
                encoding="utf-8",
            )

            report = non_network_server_references(root, [source])

        self.assertEqual(1, report["test_owned_source_file_count"])
        self.assertEqual(1, report["test_owned_server_reference_count"])
        self.assertEqual([], report["unclassified_locations"])
        self.assertEqual([], report["non_network_server_migration_debt"])

    def test_canonical_production_server_contexts_are_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sources = {
                "zircon_app/src/entry/runtime_library/library_path.rs": (
                    'let path = r"\\\\server\\share\\zircon_runtime.dll";\n'
                ),
                "zircon_editor/src/ui/workbench/startup/display_project_path.rs": (
                    'display_project_path("\\\\\\\\?\\\\UNC\\\\server\\\\share\\\\ZirconProject");\n'
                ),
                "zircon_editor/src/ui/retained_host/app/module_plugin_actions/"
                "project_policy/status.rs": (
                    'RuntimeTargetMode::ServerRuntime => "server",\n'
                ),
                "zircon_plugins/animation/runtime/src/capability.rs": (
                    "targets: [client_runtime, server_runtime, editor_host],\n"
                ),
                "zircon_runtime/src/bin/zircon_export_validate/run.rs": (
                    'let profile = OsString::from("server");\n'
                ),
                "zircon_runtime/src/platform/capability/matrix/mod.rs": (
                    'let message = "server or headless runtime requires a backend";\n'
                ),
            }
            paths = []
            for relative_path, source_text in sources.items():
                source = root / relative_path
                source.parent.mkdir(parents=True, exist_ok=True)
                source.write_text(source_text, encoding="utf-8")
                paths.append(source)

            report = non_network_server_references(root, paths)

        self.assertEqual(6, report["allowed_context_count"])
        self.assertEqual([], report["unclassified_locations"])
        self.assertEqual([], report["non_network_server_migration_debt"])

    def test_allowed_contexts_do_not_hide_unrelated_server_tokens(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sources = {
                "zircon_app/src/entry/runtime_library/library_path.rs": (
                    'let render_server = r"\\\\server\\share\\zircon_runtime.dll";\n'
                ),
                "zircon_plugins/animation/runtime/src/capability.rs": (
                    "targets: [server_runtime, render_server],\n"
                ),
                "zircon_runtime/src/bin/zircon_export_validate/run.rs": (
                    'let render_server = OsString::from("server");\n'
                ),
                "zircon_runtime/src/platform/capability/matrix/mod.rs": (
                    'let render_server = "server or headless runtime";\n'
                ),
                "zircon_editor/src/ui/retained_host/app/module_plugin_projection/"
                "rows/labels.rs": (
                    'render_server; RuntimeTargetMode::ServerRuntime => "server",\n'
                ),
            }
            paths = []
            for relative_path, source_text in sources.items():
                source = root / relative_path
                source.parent.mkdir(parents=True, exist_ok=True)
                source.write_text(source_text, encoding="utf-8")
                paths.append(source)

            report = non_network_server_references(root, paths)

        self.assertEqual(5, report["unclassified_location_count"])
        self.assertEqual(
            [["render_server"]] * 5,
            [location["tokens"] for location in report["unclassified_locations"]],
        )
        self.assertEqual("migration-debt-present", report["m1_gate_status"])

    def test_scene_server_comment_does_not_classify_unrelated_tokens(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "zircon_editor/src/ui/workbench/scene_status.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                "let render_server = (); // stale scene server owner\n",
                encoding="utf-8",
            )

            report = non_network_server_references(root, [source])

        self.assertEqual(
            {
                "editor-scene-comment-debt": 1,
                "unclassified-non-network-server": 1,
            },
            report["classification_counts"],
        )
        decisions = {
            decision["classification"]: decision
            for decision in report["reference_decisions"]
        }
        self.assertEqual(
            ["render_server"],
            decisions["unclassified-non-network-server"]["tokens"],
        )
        self.assertEqual(
            ["server"],
            decisions["editor-scene-comment-debt"]["tokens"],
        )
        self.assertEqual(1, report["count"])
        self.assertEqual(1, report["sample_location_count"])
        self.assertEqual(2, report["reference_decision_count"])
        self.assertEqual(1, report["unclassified_location_count"])

    def test_asset_owner_path_does_not_override_token_classification(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "zircon_editor/src/ui/host/resource_access.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                "asset_server; render_server; // stale scene server owner\n",
                encoding="utf-8",
            )

            report = non_network_server_references(root, [source])

        self.assertEqual(
            {
                "editor-asset-resource-owner-debt": 1,
                "editor-scene-comment-debt": 1,
                "unclassified-non-network-server": 1,
            },
            report["classification_counts"],
        )
        decisions = {
            decision["classification"]: decision
            for decision in report["reference_decisions"]
        }
        self.assertEqual(
            ["asset_server"],
            decisions["editor-asset-resource-owner-debt"]["tokens"],
        )
        self.assertEqual(
            ["server"],
            decisions["editor-scene-comment-debt"]["tokens"],
        )
        self.assertEqual(
            ["render_server"],
            decisions["unclassified-non-network-server"]["tokens"],
        )
        self.assertEqual(1, report["count"])
        self.assertEqual(1, report["sample_location_count"])
        self.assertEqual(3, report["reference_decision_count"])
        sample = report["sample_locations"][0]
        self.assertNotIn("target_owner", sample)
        self.assertNotIn("required_action", sample)
        self.assertEqual(
            [decision["target_owner"] for decision in report["reference_decisions"]],
            sample["target_owners"],
        )
        self.assertEqual(
            [
                decision["required_action"]
                for decision in report["reference_decisions"]
            ],
            sample["required_actions"],
        )

    def test_runtime_profile_server_definition_is_a_server_runtime_owner(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = (
                root
                / "zircon_runtime/src/core/framework/project/runtime_profile_id.rs"
            )
            source.parent.mkdir(parents=True)
            source.write_text(
                "pub enum RuntimeProfileId {\n    Server,\n}\n",
                encoding="utf-8",
            )

            report = non_network_server_references(root, [source])

        self.assertEqual([], report["unclassified_locations"])
        self.assertEqual([], report["non_network_server_migration_debt"])
        self.assertEqual("classified-and-clear", report["m1_gate_status"])

    def test_runtime_profile_owner_does_not_hide_unrelated_server_names(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = (
                root
                / "zircon_runtime/src/core/framework/project/runtime_profile_id.rs"
            )
            source.parent.mkdir(parents=True)
            source.write_text(
                "pub fn build() {\n    let render_server = ();\n}\n",
                encoding="utf-8",
            )

            report = non_network_server_references(root, [source])

        self.assertEqual(1, report["unclassified_location_count"])
        self.assertEqual("render_server", report["unclassified_locations"][0]["tokens"][0])


if __name__ == "__main__":
    unittest.main()
