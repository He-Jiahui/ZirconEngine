from __future__ import annotations

import unittest
from pathlib import Path
from types import SimpleNamespace

from tools.zircon_export.plugin_validate_report import (
    plugin_validate_all_report,
    plugin_validate_report,
    render_plugin_validate_all_report,
    render_plugin_validate_report,
)


class PluginValidateReportTests(unittest.TestCase):
    def test_single_report_text_includes_workspace_and_dist_crate_manifests(
        self,
    ) -> None:
        args = SimpleNamespace(form="dist")
        repo_root = Path("E:/repo")
        workspace_manifest = repo_root / "zircon_plugins" / "Cargo.toml"
        plugin_manifest = (
            repo_root / "zircon_plugins" / "native_dynamic_fixture" / "plugin.toml"
        )
        dist_crate_manifest = (
            repo_root
            / "zircon_plugins"
            / "native_dynamic_fixture"
            / "native"
            / "Cargo.toml"
        )

        report = plugin_validate_report(
            args=args,
            requested_plugin_id="native_dynamic_fixture",
            repo_root=repo_root,
            workspace_manifest=workspace_manifest,
            plugin_manifest_path=plugin_manifest,
            engine_version="0.1.0",
            package_id="native_dynamic_fixture",
            source_kind="root",
            dist_crate="zircon_plugin_native_dynamic_fixture_native",
            dist_crate_manifest=dist_crate_manifest,
            abi_version=3,
            diagnostics=[],
        )

        text = render_plugin_validate_report(report)

        self.assertIn(f"workspace_manifest={workspace_manifest}", text)
        self.assertIn(f"dist_crate_manifest={dist_crate_manifest}", text)
        self.assertIn("status=ok", text)

    def test_all_report_counts_failed_items_and_renders_item_diagnostics(self) -> None:
        args = SimpleNamespace(form="dist")
        items = [
            {
                "package_id": "sound",
                "source_kind": "root",
                "dist_crate": "zircon_plugin_sound_dist",
                "fatal": False,
                "diagnostics": [],
            },
            {
                "package_id": "sound_timeline_animation_track",
                "source_kind": "feature_provider",
                "dist_crate": "zircon_plugin_sound_timeline_animation_dist",
                "fatal": True,
                "diagnostics": ["feature provider diagnostic"],
            },
        ]

        report = plugin_validate_all_report(
            args=args,
            repo_root=Path("E:/repo"),
            workspace_manifest=Path("E:/repo/zircon_plugins/Cargo.toml"),
            engine_version="0.1.0",
            diagnostics=[],
            items=items,
        )
        text = render_plugin_validate_all_report(report)

        self.assertTrue(report["fatal"])
        self.assertEqual(report["target_count"], 2)
        self.assertEqual(report["failed_count"], 1)
        self.assertIn("failed_count=1", text)
        self.assertIn("status=failed", text)
        self.assertIn(
            "item_diagnostic=sound_timeline_animation_track: "
            "feature provider diagnostic",
            text,
        )


if __name__ == "__main__":
    unittest.main()
