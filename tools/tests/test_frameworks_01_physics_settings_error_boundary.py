import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class Frameworks01PhysicsSettingsErrorBoundaryTests(unittest.TestCase):
    def test_physics_settings_errors_are_contract_owned(self) -> None:
        physics_root = REPO_ROOT / "zircon_runtime/src/core/framework/physics"
        error_path = physics_root / "settings_store_error.rs"
        manager_source = (physics_root / "manager.rs").read_text(encoding="utf-8")
        physics_mod = (physics_root / "mod.rs").read_text(encoding="utf-8")
        plugin_settings = (
            REPO_ROOT / "zircon_plugins/physics/runtime/src/manager/settings.rs"
        ).read_text(encoding="utf-8")
        plugin_service = (
            REPO_ROOT / "zircon_plugins/physics/runtime/src/manager/service.rs"
        ).read_text(encoding="utf-8")

        self.assertTrue(error_path.is_file())
        error_source = error_path.read_text(encoding="utf-8")
        self.assertIn("pub enum PhysicsSettingsStoreError", error_source)
        self.assertIn("ReadOnlyBackend", error_source)
        self.assertIn("Persistence", error_source)
        self.assertIn("mod settings_store_error;", physics_mod)
        self.assertIn(
            "pub use settings_store_error::PhysicsSettingsStoreError;", physics_mod
        )

        self.assertNotIn("CoreError", manager_source)
        self.assertRegex(
            manager_source,
            r"fn store_settings\([^)]*\)\s*->\s*Result<\(\),\s*"
            r"PhysicsSettingsStoreError>",
        )
        self.assertIn(
            "PhysicsSettingsStoreError::read_only_backend", manager_source
        )
        self.assertIn("PhysicsSettingsStoreError", plugin_settings)
        self.assertIn("PhysicsSettingsStoreError", plugin_service)
        self.assertNotIn("Result<(), CoreError>", plugin_settings)
        self.assertNotIn("zircon_runtime::core::CoreError", plugin_service)
        self.assertRegex(
            plugin_settings,
            r"core\.store_config\([^;]+?\)\s*\.map_err\(\|source\|\s*"
            r"PhysicsSettingsStoreError::persistence\(source\.to_string\(\)\)\)",
        )
        self.assertNotRegex(
            error_source,
            r"impl\s+From\s*<[^>]*CoreError[^>]*>\s+for\s+PhysicsSettingsStoreError",
        )

        definitions = []
        compatibility_aliases = []
        definition_pattern = re.compile(
            r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+"
            r"PhysicsSettingsStoreError\b"
        )
        type_alias_pattern = re.compile(
            r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?type\s+"
            r"PhysicsSettingsStoreError\s*="
        )
        import_alias_pattern = re.compile(
            r"(?s)\buse\s+[^;]+\bas\s+PhysicsSettingsStoreError\b"
        )
        for root in (
            REPO_ROOT / "zircon_runtime/src",
            REPO_ROOT / "zircon_runtime/tests",
            REPO_ROOT / "zircon_app/src",
            REPO_ROOT / "zircon_app/tests",
            REPO_ROOT / "zircon_editor/src",
            REPO_ROOT / "zircon_editor/tests",
            REPO_ROOT / "zircon_plugins",
        ):
            for path in root.rglob("*.rs"):
                source = path.read_text(encoding="utf-8")
                if definition_pattern.search(source):
                    definitions.append(path.relative_to(REPO_ROOT).as_posix())
                if type_alias_pattern.search(source) or import_alias_pattern.search(
                    source
                ):
                    compatibility_aliases.append(
                        path.relative_to(REPO_ROOT).as_posix()
                    )
        self.assertEqual(
            [
                "zircon_runtime/src/core/framework/physics/"
                "settings_store_error.rs"
            ],
            definitions,
        )
        self.assertEqual([], compatibility_aliases)

        framework_core_error_consumers = []
        for path in (REPO_ROOT / "zircon_runtime/src/core/framework").rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            if "CoreError" in source:
                framework_core_error_consumers.append(
                    path.relative_to(REPO_ROOT).as_posix()
                )
        self.assertEqual([], framework_core_error_consumers)


if __name__ == "__main__":
    unittest.main()
