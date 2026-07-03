import tempfile
import unittest
from pathlib import Path

from tools.plugin_structure_audits.retired_ui_assets import (
    audit_retired_ui_asset_conformance,
)


class PluginStructureAuditRetiredUiAssetsTests(unittest.TestCase):
    def test_retired_ui_asset_audit_reports_legacy_ui_toml_files(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            (repo_root / "zircon_editor/ui").mkdir(parents=True)
            (repo_root / "zircon_plugins/sound/editor").mkdir(parents=True)
            (repo_root / "zircon_runtime/ui").mkdir(parents=True)

            (repo_root / "zircon_editor/ui/legacy_panel.ui.toml").write_text(
                "",
                encoding="utf-8",
            )
            (repo_root / "zircon_plugins/sound/editor/report.v2.ui.toml").write_text(
                "",
                encoding="utf-8",
            )
            (repo_root / "zircon_runtime/ui/current.zui").write_text(
                "",
                encoding="utf-8",
            )

            audit = audit_retired_ui_asset_conformance(repo_root).to_json()

        self.assertEqual(2, audit["retired_ui_asset_files"])
        self.assertEqual(
            [
                "zircon_editor/ui/legacy_panel.ui.toml",
                "zircon_plugins/sound/editor/report.v2.ui.toml",
            ],
            audit["retired_ui_asset_file_paths"],
        )
        self.assertEqual(
            "retired-ui-assets-present",
            audit["zui_only_layout_status"],
        )

    def test_retired_ui_asset_audit_reports_zui_only_clean(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            (repo_root / "zircon_editor/ui").mkdir(parents=True)
            (repo_root / "zircon_plugins/sound/editor").mkdir(parents=True)
            (repo_root / "zircon_runtime/ui").mkdir(parents=True)
            (repo_root / "zircon_editor/ui/workbench.zui").write_text(
                "",
                encoding="utf-8",
            )
            (repo_root / "zircon_plugins/sound/editor/mixer.zui").write_text(
                "",
                encoding="utf-8",
            )

            audit = audit_retired_ui_asset_conformance(repo_root).to_json()

        self.assertEqual(0, audit["retired_ui_asset_files"])
        self.assertEqual([], audit["retired_ui_asset_file_paths"])
        self.assertEqual("zui-only-clean", audit["zui_only_layout_status"])


if __name__ == "__main__":
    unittest.main()
