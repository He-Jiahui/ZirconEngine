#!/usr/bin/env python3
"""Static contract checks for Editor12 package admission before catalog publication."""

from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_ROOT = REPO_ROOT / "zircon_editor" / "src" / "core" / "plugin"
MANAGER_DISCOVERY = PLUGIN_ROOT / "manager" / "discovery.rs"
MANAGER_PUBLICATION = PLUGIN_ROOT / "manager" / "publication.rs"


class EditorPluginAdmissionContractTests(unittest.TestCase):
    def test_manager_validates_catalog_admission_before_discovery_publication(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        publication = MANAGER_PUBLICATION.read_text(encoding="utf-8")
        discovery = MANAGER_DISCOVERY.read_text(encoding="utf-8")

        self.assertIn("validate_catalog_admission", manager)
        self.assertIn("EditorPluginCatalogAdmissionError", discovery)
        self.assertEqual(manager.count("validate_catalog_admission(&catalog)?;"), 1)
        self.assertIn("validate_catalog_admission(&catalog)?;", publication)
        self.assertLess(
            publication.index("validate_catalog_admission(&catalog)?;"),
            publication.index("discovery_index(&catalog, discoveries)?"),
        )

    def test_admission_rejects_only_declared_package_dependency_cycles(self) -> None:
        admission = (PLUGIN_ROOT / "admission.rs").read_text(encoding="utf-8")
        catalog = (PLUGIN_ROOT / "catalog.rs").read_text(encoding="utf-8")

        self.assertIn("pub enum EditorPluginCatalogAdmissionError", admission)
        self.assertIn("DependencyCycle", admission)
        self.assertIn("admission_duplicate_package_ids", admission)
        self.assertIn("admission_duplicate_package_ids", catalog)
        self.assertIn("rejects_duplicate_runtime_manifest_input_for_one_editor_package", admission)
        self.assertIn("ignores_duplicate_runtime_only_manifest_input", admission)
        self.assertIn(".dependencies", admission)
        self.assertIn("find_dependency_cycle", admission)
        self.assertNotIn("NativePluginLoader", admission)
        self.assertNotIn("std::fs", admission)

    def test_plugin_module_exports_the_admission_error(self) -> None:
        module = (PLUGIN_ROOT / "mod.rs").read_text(encoding="utf-8")

        self.assertIn("mod admission;", module)
        self.assertIn("EditorPluginCatalogAdmissionError", module)

    def test_public_manager_factories_preserve_recoverable_admission_rejection(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        publication = MANAGER_PUBLICATION.read_text(encoding="utf-8")

        self.assertIn(
            "pub fn from_plugins(",
            manager,
        )
        self.assertIn("pub fn from_descriptors(", manager)
        self.assertIn("pub(crate) fn new(", manager)
        self.assertIn("pub(crate) fn publish_catalog(", publication)
        self.assertIn(
            ") -> Result<Arc<EditorPluginCatalogSnapshot>, EditorPluginDiscoveryError>",
            publication,
        )
        self.assertIn(
            "pub fn builtin_shared() -> Result<&'static Self, EditorPluginDiscoveryError>",
            manager,
        )
        self.assertNotIn(
            "Self::new_with_discoveries(catalog, std::iter::empty())\n            .expect(",
            manager,
        )
        self.assertNotIn(
            "self.publish_catalog_with_discoveries(catalog, std::iter::empty())\n            .expect(",
            publication,
        )

    def test_fallible_builtin_manager_callers_propagate_or_report_initialization(self) -> None:
        runner = (
            REPO_ROOT
            / "zircon_editor"
            / "src"
            / "core"
            / "commandlet"
            / "runner.rs"
        ).read_text(encoding="utf-8")
        editor_manager = (
            REPO_ROOT
            / "zircon_editor"
            / "src"
            / "ui"
            / "host"
            / "editor_manager.rs"
        ).read_text(encoding="utf-8")
        module = (
            REPO_ROOT / "zircon_editor" / "src" / "ui" / "host" / "module.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("match EditorPluginManager::builtin_shared()", runner)
        self.assertIn("plugin catalog initialization failed", runner)
        self.assertNotIn(
            "EditorPluginManager::builtin_shared().catalog_snapshot()", runner
        )
        self.assertIn(
            "pub fn new(core: &CoreHandle) -> Result<Self, CoreError>", editor_manager
        )
        self.assertIn("CoreError::Initialization", editor_manager)
        self.assertIn("EditorManager::new(&core)?", module)


if __name__ == "__main__":
    unittest.main()
