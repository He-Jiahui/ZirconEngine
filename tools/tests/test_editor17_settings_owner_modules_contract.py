from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SETTINGS = ROOT / "zircon_editor" / "src" / "core" / "settings"


class SettingsOwnerModulesContractTests(unittest.TestCase):
    def test_registry_snapshot_and_authority_have_separate_owner_files(self) -> None:
        registry = (SETTINGS / "registry.rs").read_text(encoding="utf-8")
        snapshot = (SETTINGS / "snapshot.rs").read_text(encoding="utf-8")
        authority = (SETTINGS / "authority.rs").read_text(encoding="utf-8")

        self.assertIn("pub struct SettingsRegistry", registry)
        self.assertIn("pub enum SettingsError", registry)
        self.assertNotIn("pub struct SettingsAuthority", registry)
        self.assertNotIn("pub struct SettingsSnapshot", registry)

        self.assertIn("struct BuiltInSettingsSlots", snapshot)
        self.assertIn("pub struct SettingsSnapshot", snapshot)
        self.assertIn("pub struct ViewportSnapSettings", snapshot)
        self.assertNotIn("pub struct SettingsAuthority", snapshot)

        self.assertIn("pub struct SettingsAuthority", authority)
        self.assertIn("pub enum SettingsProjectLayerLoad", authority)
        self.assertIn("trait SettingsChangeSubscriber", authority)
        self.assertNotIn("pub struct SettingsRegistry", authority)

    def test_settings_facade_reexports_the_existing_public_surface(self) -> None:
        facade = (SETTINGS / "mod.rs").read_text(encoding="utf-8")

        for module in ("authority", "registry", "snapshot", "startup"):
            self.assertIn(f"mod {module};", facade)
        for symbol in (
            "SettingsAuthority",
            "SettingsProjectLayerLoad",
            "SettingsRegistry",
            "SettingsSnapshot",
            "SettingsUserLayerLoad",
            "ViewportSnapSettings",
        ):
            self.assertIn(symbol, facade)

    def test_existing_settings_entry_points_remain_with_their_owner(self) -> None:
        registry = (SETTINGS / "registry.rs").read_text(encoding="utf-8")
        snapshot = (SETTINGS / "snapshot.rs").read_text(encoding="utf-8")
        authority = (SETTINGS / "authority.rs").read_text(encoding="utf-8")
        startup = (SETTINGS / "startup.rs").read_text(encoding="utf-8")

        for entry_point in (
            "fn register(&mut self",
            "fn resolve(&self",
            "fn set(",
            "fn clear(",
            "fn changes_since(&mut self",
            "fn replace_persistent_layer(",
        ):
            self.assertIn(entry_point, registry)
        for entry_point in (
            "fn from_registry(registry: &SettingsRegistry)",
            "fn after_change(",
            "fn design_tokens_handle(&self)",
            "fn keymap_overrides_handle(&self)",
        ):
            self.assertIn(entry_point, snapshot)
        for entry_point in (
            "fn snapshot(&self) -> Arc<SettingsSnapshot>",
            "fn configure_change_subscriber(",
            "fn load_project_layer_from_environment(",
            "fn prepare_persistent_layer_for_write(",
            "fn clear_project_layer(&self)",
        ):
            self.assertIn(entry_point, authority)
        for entry_point in (
            "fn load_from_environment(registry: SettingsRegistry)",
            "fn load_from_store(",
            "fn into_authority(self) -> SettingsAuthority",
        ):
            self.assertIn(entry_point, startup)
        self.assertNotIn("fn at_startup() -> Self", authority)
        self.assertNotIn("SettingsStore::from_user_environment", authority)

    def test_project_layer_transition_releases_callback_visible_cache_state(self) -> None:
        authority = (SETTINGS / "authority.rs").read_text(encoding="utf-8")
        tests = (SETTINGS / "tests" / "registry.rs").read_text(encoding="utf-8")

        for symbol in (
            "struct ProjectLayerState",
            "transition_in_progress",
            "project_layer_operation: Mutex<()>",
            "fn begin_project_layer_transition(&self)",
            "fn finish_project_layer_transition(",
        ):
            self.assertIn(symbol, authority)
        self.assertIn(
            "project_layer_transition_rejects_reentrant_persistence_prepare", tests
        )

    def test_owner_files_stay_under_the_structure_review_threshold(self) -> None:
        for owner in ("registry.rs", "snapshot.rs", "authority.rs"):
            with self.subTest(owner=owner):
                self.assertLessEqual(
                    len((SETTINGS / owner).read_text(encoding="utf-8").splitlines()),
                    800,
                )


if __name__ == "__main__":
    unittest.main()
