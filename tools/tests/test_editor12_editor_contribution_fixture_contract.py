"""Static contract tests for the Editor12 native contribution fixture."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_ROOT = REPOSITORY_ROOT / "zircon_plugins" / "editor_contribution_fixture"
NATIVE_SOURCE = PLUGIN_ROOT / "native" / "src" / "lib.rs"


class EditorContributionFixtureContractTests(unittest.TestCase):
    def test_workspace_and_plugin_manifest_describe_a_native_editor_fixture(self) -> None:
        workspace = (REPOSITORY_ROOT / "zircon_plugins" / "Cargo.toml").read_text(
            encoding="utf-8"
        )
        manifest = (PLUGIN_ROOT / "plugin.toml").read_text(encoding="utf-8")
        crate = (PLUGIN_ROOT / "native" / "Cargo.toml").read_text(encoding="utf-8")

        self.assertIn('"editor_contribution_fixture/native"', workspace)
        self.assertIn('id = "editor_contribution_fixture"', manifest)
        self.assertIn('kind = "editor"', manifest)
        self.assertIn(
            'editor_entry = "zircon_editor_contribution_fixture_entry_v3"', manifest
        )
        self.assertIn('crate-type = ["cdylib"]', crate)
        self.assertIn('features = ["native"]', crate)

    def test_native_entry_exposes_only_the_versioned_editor_payload(self) -> None:
        source = NATIVE_SOURCE.read_text(encoding="utf-8")

        self.assertIn("native_dist_editor_plugin_v3!", source)
        self.assertIn(
            "registration_manifest_schema: Some(EDITOR_CONTRIBUTION_BATCH_SCHEMA)",
            source,
        )
        self.assertIn(
            "registration_manifest: Some(EDITOR_CONTRIBUTION_BATCH)", source
        )
        self.assertIn("on_host_ready: None", source)
        self.assertNotIn("ZrHostApi", source)
        self.assertNotIn("HostApiV", source)

    def test_embedded_payload_is_canonical_and_covers_every_host_safe_kind(self) -> None:
        source = NATIVE_SOURCE.read_text(encoding="utf-8")
        payload_match = re.search(
            r'const EDITOR_CONTRIBUTION_BATCH_TEXT: &str = concat!\(\s*r#"(.*?)"#,\s*"\\0"\s*\);',
            source,
            flags=re.DOTALL,
        )
        self.assertIsNotNone(payload_match, "fixture must expose a NUL-terminated JSON payload")
        payload = json.loads(payload_match.group(1))

        self.assertEqual(payload["package_id"], "editor_contribution_fixture")
        contributions = payload["contributions"]
        self.assertEqual(
            {contribution["kind"] for contribution in contributions},
            {
                "view",
                "drawer",
                "menu",
                "command",
                "asset_type",
                "localization_bundle",
                "settings_page",
            },
        )
        expected_schema_by_kind = {
            "view": "zircon.editor.view/1",
            "drawer": "zircon.editor.drawer/1",
            "menu": "zircon.editor.menu/1",
            "command": "zircon.editor.command/1",
            "asset_type": "zircon.editor.asset-type/1",
            "localization_bundle": "zircon.editor.localization-bundle/1",
            "settings_page": "zircon.editor.settings-page/2",
        }
        self.assertEqual(len(contributions), len(expected_schema_by_kind))
        for contribution in contributions:
            self.assertEqual(
                contribution["schema"], expected_schema_by_kind[contribution["kind"]]
            )


if __name__ == "__main__":
    unittest.main()
