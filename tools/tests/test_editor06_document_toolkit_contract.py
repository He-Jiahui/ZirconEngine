from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TOOLKIT = ROOT / "zircon_editor/src/core/extension/toolkit/document_toolkit.rs"
HOST = ROOT / "zircon_editor/src/ui/host/editor_ui_host.rs"
ASSET_SAVE = ROOT / "zircon_editor/src/ui/host/asset_editor_sessions/save.rs"
ANIMATION_SAVE = ROOT / "zircon_editor/src/ui/host/animation_editor_sessions/save.rs"
ASSET_OPEN = ROOT / "zircon_editor/src/ui/host/asset_editor_sessions/open.rs"
ASSET_LIFECYCLE = ROOT / "zircon_editor/src/ui/host/asset_editor_sessions/lifecycle.rs"
ASSET_SYNC = ROOT / "zircon_editor/src/ui/host/asset_editor_sessions/sync.rs"
BUILTIN_ASSET_TYPES = ROOT / "zircon_editor/src/core/asset/type_registry/builtin.rs"
DEFAULT_COMMANDS = ROOT / "zircon_editor/src/core/commands/defaults.rs"
LEGACY_CLOSE_SAVE = ROOT / (
    "zircon_editor/src/ui/retained_host/app/native_window_close/"
    "prompt_actions/saving.rs"
)


class Editor06DocumentToolkitContractTests(unittest.TestCase):
    def test_toolkit_hook_does_not_own_dirty_or_saved_top_state(self) -> None:
        source = TOOLKIT.read_text(encoding="utf-8")

        self.assertIn("pub trait DocumentToolkit<Host>", source)
        self.assertIn("fn save(&self, host: &Host, context: &mut SaveCtx)", source)
        self.assertNotIn("fn is_dirty", source)
        self.assertNotIn("saved_top", source)
        self.assertNotIn("generation:", source)

    def test_single_document_save_uses_token_then_hook_then_compare_and_mark(self) -> None:
        source = HOST.read_text(encoding="utf-8")
        token = source.index("capture_save_token(document)")
        hook = source.index("document_toolkits.save(document, self, reason)", token)
        mark = source.index("mark_saved_if_unchanged(document, save_token)", hook)
        external = source.index("clear_saved_external_effects(&dirty_snapshot)", mark)

        self.assertLess(token, hook)
        self.assertLess(hook, mark)
        self.assertLess(mark, external)
        self.assertIn("DocumentChangedDuringSave", source)

    def test_asset_and_animation_public_save_paths_dispatch_the_toolkit(self) -> None:
        asset = ASSET_SAVE.read_text(encoding="utf-8")
        animation = ANIMATION_SAVE.read_text(encoding="utf-8")

        self.assertIn("self.save_document_toolkit(instance_id, SaveReason::Explicit)?", asset)
        self.assertIn("self.save_document_toolkit(instance_id, SaveReason::Explicit)?", animation)
        self.assertIn("fn save_ui_asset_editor_canonical", asset)
        self.assertIn("fn save_animation_editor_canonical", animation)
        self.assertNotIn("pub fn save_ui_asset_editor_canonical", asset)
        self.assertNotIn("pub fn save_animation_editor_canonical", animation)

    def test_ui_asset_workspace_route_is_canonical_and_non_reverting(self) -> None:
        opened = ASSET_OPEN.read_text(encoding="utf-8")
        lifecycle = ASSET_LIFECYCLE.read_text(encoding="utf-8")
        sync = ASSET_SYNC.read_text(encoding="utf-8")
        builtins = BUILTIN_ASSET_TYPES.read_text(encoding="utf-8")
        commands = DEFAULT_COMMANDS.read_text(encoding="utf-8")

        self.assertIn("AssetToolkitOpenRoute", opened)
        self.assertIn("UI_ASSET_EDITOR_OPEN_OPERATION", opened)
        self.assertIn("project_asset_id_for_source_path", opened)
        self.assertIn("outside the active project asset roots", opened)
        self.assertIn("AssetToolkitOpenRoute", lifecycle)
        self.assertIn("UI_ASSET_EDITOR_OPEN_OPERATION", lifecycle)
        self.assertNotIn("UiAssetEditorRoute", lifecycle)
        self.assertNotIn("has_toolkit_route_shape", lifecycle)
        self.assertNotIn("serde_json::to_value(entry.session.route())", sync)
        self.assertNotIn("instance.serializable_payload =", sync)
        self.assertIn("ResourceKind::UiLayout | ResourceKind::UiWidget | ResourceKind::UiStyle", builtins)
        self.assertIn('path("view.editor.ui_asset.open")', commands)

    def test_legacy_close_prompt_save_branch_is_physically_removed(self) -> None:
        self.assertFalse(LEGACY_CLOSE_SAVE.exists())


if __name__ == "__main__":
    unittest.main()
