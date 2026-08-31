from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorTestInfrastructurePerformanceContractTests(unittest.TestCase):
    def test_retired_symbol_guard_does_not_aggregate_all_editor_sources(self) -> None:
        source = (ROOT / "zircon_editor/src/tests/commands/registry.rs").read_text(
            encoding="utf-8"
        )

        self.assertNotIn("let mut source = String::new()", source)
        self.assertNotIn("source.push_str", source)
        self.assertIn("retired_symbol_hits", source)

    def test_structure_audit_is_cached_once_per_test_process(self) -> None:
        source = (ROOT / "zircon_editor/src/tests/structure_convention/mod.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("OnceLock<Value>", source)
        self.assertIn("get_or_init", source)
        self.assertEqual(source.count("run_python_audit("), 2)

    def test_plugin_manifest_inventory_is_cached_once_per_test_process(self) -> None:
        source = (
            ROOT / "zircon_editor/src/tests/editor_plugin_catalog_consistency.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "OnceLock<BTreeMap<String, PluginPackageManifest>>", source
        )
        self.assertIn("MANIFESTS.get_or_init", source)
        self.assertEqual(source.count("fs::read_dir("), 1)
        self.assertEqual(source.count("fs::read_to_string("), 1)

    def test_material_lab_fixtures_are_cached_once_per_test_process(self) -> None:
        support = (
            ROOT
            / "zircon_editor/src/tests/ui/boundary/material_component_lab/support.rs"
        ).read_text(encoding="utf-8")
        inventory_root = (
            ROOT
            / "zircon_editor/src/tests/ui/boundary/material_component_lab/inventory"
        )
        inventory = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(inventory_root.glob("*.rs"))
        )

        self.assertIn("OnceLock<Vec<PathBuf>>", support)
        self.assertIn(
            "OnceLock<BTreeMap<String, MaterialPrototypeFixture>>", support
        )
        self.assertRegex(support, r"MATERIAL_PROTOTYPE_FILES\s*\.get_or_init")
        self.assertIn("MATERIAL_PROTOTYPE_FIXTURES.get_or_init", support)
        self.assertEqual(support.count("fs::read_dir(&domain_dir)"), 1)
        self.assertEqual(
            support.count("UiZuiAssetLoader::load_zui_str(&source)"), 1
        )
        self.assertNotIn("UiZuiAssetLoader::load_zui_str", inventory)

    def test_material_lab_theme_projection_is_cached_once_per_test_process(self) -> None:
        support = (
            ROOT
            / "zircon_editor/src/tests/ui/boundary/material_component_lab/support.rs"
        ).read_text(encoding="utf-8")
        lab_theme_root = (
            ROOT
            / "zircon_editor/src/tests/ui/boundary/material_component_lab/lab_theme"
        )
        lab_theme = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(lab_theme_root.glob("*.rs"))
        )
        mui_x_theme = (
            ROOT
            / "zircon_editor/src/tests/ui/boundary/material_component_lab/mui_x_theme.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("OnceLock<String>", support)
        self.assertIn("OnceLock<BTreeSet<String>>", support)
        self.assertRegex(
            support, r"EDITOR_MATERIAL_THEME_SOURCE\s*\.get_or_init"
        )
        self.assertIn("EDITOR_MATERIAL_THEME_SELECTORS.get_or_init", support)
        self.assertNotIn("fn theme_selectors", lab_theme)
        self.assertNotIn("fn theme_selectors", mui_x_theme)
        self.assertNotIn("fs::read_to_string(editor_asset", lab_theme)

    def test_zui_governance_documents_are_parsed_once_per_test_process(self) -> None:
        governance_root = (
            ROOT / "zircon_editor/src/tests/ui/boundary/zui_asset_governance"
        )
        support = (governance_root / "support.rs").read_text(encoding="utf-8")

        self.assertIn(
            "OnceLock<BTreeMap<PathBuf, UiV2AssetDocument>>", support
        )
        self.assertRegex(
            support, r"PRODUCTION_ZUI_DOCUMENTS\s*\.get_or_init"
        )
        self.assertEqual(
            support.count("UiZuiAssetLoader::load_zui_str(&source)"), 1
        )

        direct_parser_users = []
        governance_files = list(governance_root.glob("*.rs"))
        governance_files.append(governance_root.parent / "zui_asset_governance.rs")
        for path in governance_files:
            if path.name == "support.rs":
                continue
            source = path.read_text(encoding="utf-8")
            if "UiZuiAssetLoader::load_zui_str" in source:
                direct_parser_users.append(path.name)
        self.assertEqual(direct_parser_users, [])

    def test_material_surface_import_graph_uses_set_membership(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("let mut visited = BTreeSet::new()", source)
        self.assertIn("if !visited.insert(current.clone())", source)
        self.assertNotIn("visited.contains(&current)", source)

    def test_material_meta_fixture_is_cached_once_per_test_process(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/tests/ui/boundary/material_meta_component_contracts/fixture.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("static MATERIAL_META_DOCUMENT: OnceLock<Value>", source)
        self.assertRegex(source, r"MATERIAL_META_DOCUMENT\s*\.get_or_init")

    def test_template_showcase_bindings_are_projected_once_per_test_process(self) -> None:
        root = ROOT / "zircon_editor/src/tests/host/template_runtime"
        support = (root / "support.rs").read_text(encoding="utf-8")

        self.assertIn(
            "OnceLock<BTreeMap<String, EditorUiBinding>>", support
        )
        self.assertRegex(support, r"SHOWCASE_BINDINGS\s*\.get_or_init")
        self.assertEqual(
            support.count('.project_document(COMPONENT_SHOWCASE_DOCUMENT_ID)'), 1
        )

        showcase_sources = [
            root / "component_showcase_category.rs",
            root / "component_showcase_selection.rs",
            root / "dual_host_parity.rs",
            *(root / "component_showcase_state").rglob("*.rs"),
        ]
        for source_path in showcase_sources:
            source = source_path.read_text(encoding="utf-8")
            self.assertNotIn("fn showcase_binding(", source)

    def test_pane_body_specs_do_not_rebuild_the_editor_runtime_per_lookup(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "OnceLock<BTreeMap<String, PaneBodySpec>>", source
        )
        self.assertRegex(source, r"PANE_BODY_SPECS\s*\.get_or_init")
        self.assertEqual(source.count("let runtime = editor_runtime();"), 1)

    def test_workbench_contract_node_lookup_borrows_wide_model_rows(self) -> None:
        for relative in [
            "workbench_projection/support.rs",
            "status_bar.rs",
        ]:
            source = (
                ROOT
                / "zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge"
                / relative
            ).read_text(encoding="utf-8")
            start = source.index("fn template_contract_node")
            body = source[start : source.index("\n}\n", start) + 3]

            self.assertIn("nodes.get(row)", body)
            self.assertNotIn("nodes.row_data(row)", body)


if __name__ == "__main__":
    unittest.main()
