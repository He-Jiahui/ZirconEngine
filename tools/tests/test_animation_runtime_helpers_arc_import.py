import unittest
from pathlib import Path


class AnimationRuntimeHelpersArcImportTests(unittest.TestCase):
    def test_runtime_helpers_imports_arc_for_asset_manager_return_type(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        source = (
            repo_root
            / "zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("use std::sync::Arc;", source)
        self.assertIn(
            "pub(super) fn runtime_asset_manager(core: &CoreHandle) -> Arc<ProjectAssetManager>",
            source,
        )

    def test_parent_contract_has_no_stale_split_imports_or_locals(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        source = (
            repo_root
            / "zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn(
            "use zircon_runtime::core::framework::scene::EntityPath;", source
        )
        without_physics_test = source[
            source.index("fn level_tick_without_physics_plugin_does_not_run_physics()") :
        ]
        without_physics_test = without_physics_test[: without_physics_test.index("\n}\n") + 3]
        self.assertNotIn("let core = runtime.handle();", without_physics_test)


if __name__ == "__main__":
    unittest.main()
