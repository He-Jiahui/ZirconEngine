from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class Editor17SettingsFileGenerationContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_persistence_ticket_owns_file_identity_and_separate_generations(self) -> None:
        persistence = self.read("zircon_editor/src/core/settings/persistence.rs")

        self.assertIn("pub struct SettingsFileGeneration", persistence)
        self.assertIn("static NEXT_SETTINGS_FILE_GENERATION: AtomicU64", persistence)
        service = persistence[persistence.index("pub struct SettingsPersistenceService"):]
        self.assertNotIn("next_file_generation:", service)
        self.assertIn("file_generation: SettingsFileGeneration", persistence)
        self.assertIn("authority_generation: u64", persistence)
        self.assertIn("pub const fn file_generation(&self)", persistence)
        self.assertIn("pub const fn authority_generation(&self)", persistence)
        self.assertIn("pub fn target(&self) -> &str", persistence)
        self.assertNotIn("\n            generation: change.revision", persistence)

    def test_file_generation_is_reserved_before_persistent_authority_mutation(self) -> None:
        mutation = self.read("zircon_editor/src/core/settings/mutation.rs")

        begin = mutation.index("    fn begin_mutation(")
        target = mutation[begin:]
        self.assertIn("allocate_file_generation", target)
        self.assertIn("file_generation", target)
        self.assertIn("persistence_generation", mutation)

    def test_admission_failure_and_retry_keep_the_exact_file_generation(self) -> None:
        mutation = self.read("zircon_editor/src/core/settings/mutation.rs")

        self.assertIn("file_generation: SettingsFileGeneration", mutation)
        self.assertIn("ticket.file_generation()", mutation)
        self.assertIn("pub const fn file_generation(&self)", mutation)
        self.assertNotIn("pub const fn authority_generation(&self) -> Option<u64>", mutation)

    def test_health_uses_file_generation_not_authority_revision(self) -> None:
        health = self.read("zircon_editor/src/core/settings/mutation/health.rs")

        self.assertIn("file_generation: Option<SettingsFileGeneration>", health)
        self.assertIn("pub const fn file_generation(self)", health)
        self.assertNotIn("authority_generation", health)


if __name__ == "__main__":
    unittest.main()
