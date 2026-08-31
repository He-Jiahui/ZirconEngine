"""Static contract for Editor10 runtime-backed asset delete admission."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
REFACTOR = ROOT / "zircon_editor/src/core/asset/refactor/delete.rs"
TESTS = ROOT / "zircon_editor/src/core/asset/refactor/tests.rs"
API = ROOT / "zircon_editor/src/ui/host/editor_asset_manager/api.rs"
MANAGER = (
    ROOT
    / "zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_refactor.rs"
)
EDITOR_ASSET_STATE = (
    ROOT
    / "zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/editor_asset_state.rs"
)


class Editor10AssetRefactorPreflightContractTests(unittest.TestCase):
    def test_delete_preflight_is_runtime_registry_backed_and_fail_closed(self):
        source = REFACTOR.read_text(encoding="utf-8")
        tests = TESTS.read_text(encoding="utf-8")

        self.assertIn("AssetMutationDeletePreflight", source)
        self.assertIn("AssetMutationDeleteDisposition", source)
        self.assertIn("AssetSourceAuthority::from_locator", source)
        self.assertIn("AssetDeleteDisposition::MissingAsset", source)
        self.assertIn("AssetDeleteDisposition::ReadOnlySource", source)
        self.assertIn("AssetDeleteDisposition::UnsupportedSubasset", source)
        self.assertIn("AssetDeleteDisposition::BlockedByReferencers", source)
        self.assertNotIn("get_referencers_by_uuid", source)
        self.assertIn("delete_preflight_blocks_and_projects_referencers", tests)

    def test_editor_asset_manager_exposes_the_runtime_backed_preflight_gateway(self):
        source = REFACTOR.read_text(encoding="utf-8")
        api = API.read_text(encoding="utf-8")
        manager = MANAGER.read_text(encoding="utf-8")
        editor_asset_state = EDITOR_ASSET_STATE.read_text(encoding="utf-8")

        self.assertIn("fn asset_delete_preflight(", api)
        self.assertIn("AssetSourceWritePolicy", api)
        self.assertIn("AssetDeletePreflight", api)
        self.assertIn("project.asset_registry()", manager)
        self.assertIn("AssetDeletePreflight::evaluate", manager)
        self.assertIn("self.read_state_recovering_poison()", manager)
        self.assertIn(
            "fn read_editor_asset_state_recovering_poison(", editor_asset_state
        )
        self.assertIn(
            "unwrap_or_else(|poisoned| poisoned.into_inner())", editor_asset_state
        )
        self.assertNotIn("state.read().expect(\"editor asset state lock poisoned\")", manager)
        self.assertNotIn("get_referencers_by_uuid", source)


if __name__ == "__main__":
    unittest.main()
