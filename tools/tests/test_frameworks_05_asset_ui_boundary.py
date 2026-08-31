from __future__ import annotations

import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSET_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "asset"


def production_rust_sources(root: Path) -> list[Path]:
    sources: list[Path] = []
    for path in root.rglob("*.rs"):
        relative = path.relative_to(root)
        if "tests" in relative.parts:
            continue
        if path.name in {"test.rs", "tests.rs"}:
            continue
        if path.name.startswith("test_") or path.name.endswith("_tests.rs"):
            continue
        sources.append(path)
    return sources


class Frameworks05AssetUiBoundaryTests(unittest.TestCase):
    def test_asset_production_has_no_ui_domain_references(self) -> None:
        offenders: list[str] = []
        for path in production_rust_sources(ASSET_ROOT):
            for line_number, line in enumerate(
                path.read_text(encoding="utf-8").splitlines(), start=1
            ):
                if "crate::ui::" in line:
                    offenders.append(
                        f"{path.relative_to(REPO_ROOT).as_posix()}:{line_number}: {line.strip()}"
                    )

        self.assertEqual([], offenders)

    def test_zui_loader_registration_is_owned_by_ui_document_plugin(self) -> None:
        retired_importer = (
            ASSET_ROOT / "importer" / "ingest" / "import_ui_v2_asset.rs"
        )
        asset_importer = (
            ASSET_ROOT / "importer" / "ingest" / "asset_importer.rs"
        ).read_text(encoding="utf-8")
        asset_ingest_mod = (
            ASSET_ROOT / "importer" / "ingest" / "mod.rs"
        ).read_text(encoding="utf-8")
        plugin = (
            REPO_ROOT
            / "zircon_plugins"
            / "ui_document_importer"
            / "runtime"
            / "src"
            / "plugin.rs"
        ).read_text(encoding="utf-8")

        self.assertFalse(retired_importer.exists())
        self.assertNotIn("import_ui_v2_asset", asset_importer)
        self.assertNotIn("import_ui_v2_asset", asset_ingest_mod)
        self.assertNotIn("import_ui_zui_asset", asset_importer)
        self.assertNotIn("import_ui_zui_asset", asset_ingest_mod)
        self.assertNotIn("zircon.builtin.ui.zui", asset_importer)
        self.assertIn("registry.register_asset_importer", plugin)
        self.assertIn("import_ui_zui_document", plugin)

    def test_ui_products_link_the_ui_document_importer_provider(self) -> None:
        runtime_manifest = (
            REPO_ROOT
            / "zircon_runtime"
            / "src"
            / "builtin"
            / "runtime_modules"
            / "manifest.rs"
        ).read_text(encoding="utf-8")
        catalog_manifest_path = (
            REPO_ROOT / "zircon_plugins" / "first_party_runtime_catalog" / "Cargo.toml"
        )
        catalog_manifest = tomllib.loads(
            catalog_manifest_path.read_text(encoding="utf-8")
        )
        catalog_source = (
            catalog_manifest_path.parent / "src" / "lib.rs"
        ).read_text(encoding="utf-8")
        app_manifest = tomllib.loads(
            (REPO_ROOT / "zircon_app" / "Cargo.toml").read_text(encoding="utf-8")
        )
        app_projection = (
            REPO_ROOT / "zircon_app" / "src" / "entry" / "first_party_runtime_plugins.rs"
        ).read_text(encoding="utf-8")
        app_selection = (
            REPO_ROOT / "zircon_app" / "src" / "entry" / "builtin_modules.rs"
        ).read_text(encoding="utf-8")
        app_entry = (
            REPO_ROOT / "zircon_app" / "src" / "entry" / "engine_entry.rs"
        ).read_text(encoding="utf-8")
        app_profile_tests = (
            REPO_ROOT
            / "zircon_app"
            / "src"
            / "entry"
            / "tests"
            / "profile_bootstrap"
            / "first_party_runtime_plugins.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("RuntimePluginId::UiDocumentImporter", runtime_manifest)
        self.assertEqual(
            ["dep:zircon_plugin_ui_document_importer_runtime"],
            catalog_manifest["features"]["ui-document-importer"],
        )
        self.assertEqual(
            {"path": "../ui_document_importer/runtime", "optional": True},
            catalog_manifest["dependencies"][
                "zircon_plugin_ui_document_importer_runtime"
            ],
        )
        self.assertIn('feature = "ui-document-importer"', catalog_source)
        self.assertIn(
            "zircon_plugin_ui_document_importer_runtime::plugin_registration()",
            catalog_source,
        )
        self.assertEqual(
            [
                "dep:zircon_first_party_runtime_catalog",
                "zircon_first_party_runtime_catalog/ui-document-importer",
            ],
            app_manifest["features"]["first-party-ui-document-importer"],
        )
        self.assertIn(
            "first-party-ui-document-importer", app_manifest["features"]["ui"]
        )
        self.assertIn(
            "effective_project_plugin_manifest(config)", app_projection
        )
        self.assertIn(
            "pub(super) fn effective_project_plugin_manifest", app_selection
        )
        self.assertEqual(
            3,
            app_selection.count(
                "let effective_manifest = effective_project_plugin_manifest(config);"
            ),
        )
        self.assertNotIn(
            "pub(super) fn builtin_modules_for_config(",
            app_selection,
        )
        self.assertIn("Some(&effective_manifest)", app_selection)
        self.assertIn("manifest_with_mode_baseline", app_selection)
        self.assertIn(
            "pub(super) fn render_profile_runtime_plugin_overlay", app_selection
        )
        self.assertNotIn(
            "fn render_profile_runtime_plugin_overlay", app_projection
        )
        self.assertIn(
            "Self::for_config_with_first_party_runtime_plugin_registrations(config)",
            app_entry,
        )
        self.assertEqual(
            1,
            app_entry.count(
                "let effective_manifest = effective_project_plugin_manifest(config);"
            ),
        )
        self.assertIn(
            "first_party_runtime_plugin_registrations_for_manifest(", app_entry
        )
        self.assertIn(
            "builtin_modules_for_config_with_effective_manifest_and_runtime_plugin_registrations(",
            app_entry,
        )
        self.assertGreaterEqual(app_entry.count("&effective_manifest"), 2)
        self.assertIn(
            "runtime_profile_bootstrap_reports_missing_required_ui_document_importer",
            app_profile_tests,
        )
        self.assertIn(
            "BuiltinEngineEntry::for_config(&config)", app_profile_tests
        )

    def test_root_module_order_has_no_asset_ui_semantic_comment(self) -> None:
        crate_root = (REPO_ROOT / "zircon_runtime" / "src" / "lib.rs").read_text(
            encoding="utf-8"
        )

        self.assertNotIn("must be declared before", crate_root)


if __name__ == "__main__":
    unittest.main()
