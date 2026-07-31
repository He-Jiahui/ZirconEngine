from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class Editor11KeymapVersionShellContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_keymap_owns_a_folder_backed_persistence_module(self) -> None:
        root = self.read("zircon_editor/src/core/commands/keymap.rs")
        persistence = self.read(
            "zircon_editor/src/core/commands/keymap/persistence.rs"
        )
        self.assertIn("mod persistence;", root)
        self.assertIn("VersionedSchema", persistence)
        self.assertIn("zircon.editor.keymap-user-layer", persistence)

    def test_user_layer_is_a_delta_with_explicit_unbinds(self) -> None:
        persistence = self.read(
            "zircon_editor/src/core/commands/keymap/persistence.rs"
        )
        self.assertIn("BTreeMap<String, Option<String>>", persistence)
        self.assertIn("serialize_user_layer", persistence)
        self.assertIn("apply_user_layer", persistence)
        self.assertIn("user_layer: BTreeMap<EditorOperationPath", self.read(
            "zircon_editor/src/core/commands/keymap.rs"
        ))

    def test_behavior_tests_cover_migration_atomicity_and_roundtrip(self) -> None:
        tests = self.read("zircon_editor/src/core/commands/keymap/tests.rs")
        for test_name in (
            "user_layer_merges_rebind_unbind_and_add_without_copying_defaults",
            "current_user_layer_roundtrip_is_canonical_and_not_reported_as_migrated",
            "invalid_user_chord_rejects_the_whole_layer_atomically",
            "absent_command_unbind_tombstone_survives_resave_and_future_default_return",
            "invalid_operation_path_rejects_the_whole_layer_atomically",
            "user_layer_path_save_and_load_preserves_effective_bindings",
            "user_layer_path_errors_preserve_io_kind_and_operation",
            "malformed_user_layer_preserves_typed_load_error",
        ):
            self.assertIn(test_name, tests)

    def test_migration_uses_a_real_v0_fixture(self) -> None:
        fixture = self.read(
            "tests/fixtures/serialization/editor-keymap-user-layer/v0/"
            "keymap-user-layer.json"
        )
        tests = self.read("zircon_editor/src/core/commands/keymap/tests.rs")
        self.assertIn('"plugin.absent.command": null', fixture)
        self.assertIn("include_str!", tests)

    def test_child_record_uses_canonical_output_heading(self) -> None:
        record = self.read(
            "docs/plans/zircon_editor/editor/11/"
            "2026-07-22-keymap-user-layer-version-shell.md"
        )
        self.assertIn("## 产出记录与时间", record)
        self.assertIn("source_complete_static_green_validation_pending", record)


if __name__ == "__main__":
    unittest.main()
