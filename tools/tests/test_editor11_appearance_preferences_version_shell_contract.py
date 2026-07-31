from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class Editor11AppearancePreferencesVersionShellTests(unittest.TestCase):
    def test_preferences_document_uses_the_shared_version_shell(self) -> None:
        persistence = source("zircon_editor/src/ui/preferences/persistence.rs")

        for required in [
            "impl VersionedSchema for EditorAppearancePreferencesDocument",
            'SchemaId::new("zircon.editor.appearance-preferences")',
            "write_versioned_text",
            "load_versioned::<EditorAppearancePreferencesDocument>",
        ]:
            self.assertIn(required, persistence)
        self.assertRegex(
            persistence,
            re.compile(
                r"MigrationStep::new\(\s*0,\s*migrate_legacy_preferences_v0_to_v1\s*\)"
            ),
        )
        self.assertNotIn("APPEARANCE_PREFERENCES_VERSION", persistence)
        self.assertNotIn("pub(crate) version: u32", persistence)

    def test_legacy_toml_is_projected_through_v0_migration(self) -> None:
        persistence = source("zircon_editor/src/ui/preferences/persistence.rs")
        tests = source("zircon_editor/src/ui/preferences/tests/persistence.rs")

        self.assertIn("toml::from_str::<toml::Value>", persistence)
        self.assertIn("legacy_version", persistence)
        self.assertIn("migrate_legacy_workbench_typography", persistence)
        self.assertIn("appearance-v1.toml", tests)
        self.assertIn("legacy_v1_migrates_and_resaves_canonically", tests)

    def test_load_metadata_reaches_startup_policy(self) -> None:
        persistence = source("zircon_editor/src/ui/preferences/persistence.rs")
        startup = source("zircon_editor/src/ui/preferences/startup.rs")

        self.assertIn("EditorAppearancePreferencesLoad", persistence)
        self.assertIn("migrated_from: loaded.migrated_from", persistence)
        self.assertIn("loaded.migrated_from", startup)
        self.assertIn("loaded.value", startup)

    def test_old_conversion_helpers_are_removed_from_appearance_owner(self) -> None:
        appearance = source("zircon_editor/src/ui/preferences/appearance.rs")

        self.assertNotIn("to_persistence_document", appearance)
        self.assertNotIn("from_persistence_document", appearance)
        self.assertNotIn("APPEARANCE_PREFERENCES_VERSION", appearance)


if __name__ == "__main__":
    unittest.main()
