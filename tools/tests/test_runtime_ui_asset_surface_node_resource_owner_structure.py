import unittest
from pathlib import Path


class RuntimeUiAssetSurfaceNodeResourceOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_09_15_ui_asset_surface_node_resource_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_node_resource_registration_is_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = repo_root / "zircon_runtime/src/ui/template/asset/surface_index.rs"
        child_path = (
            repo_root
            / "zircon_runtime/src/ui/template/asset/surface_index/node_resource_registration.rs"
        )

        owner = owner_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)
        self.assertIn("mod node_resource_registration;", owner)
        for moved in (
            "fn record_tree_node_resources",
            "struct NodeResourceCollector",
            "fn collect_map",
            "fn collect_value",
            "fn collect_resource_table",
            "fn push_resource_uri",
            "fn finish",
            "fn is_resource_table",
            "fn fallback_policy_from_table",
            "fn fallback_mode_from_name",
            "fn resource_kind_from_name",
            "fn has_supported_resource_scheme",
        ):
            self.assertNotIn(moved, owner)

        child = child_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 240)
        for anchor in (
            "pub fn record_tree_node_resources",
            "struct NodeResourceCollector",
            "fn collect_map",
            "fn collect_value",
            "fn collect_resource_table",
            "fn push_resource_uri",
            "fn finish",
            "fn is_resource_table",
            "fn fallback_policy_from_table",
            "fn fallback_mode_from_name",
            "fn resource_kind_from_name",
            "fn has_supported_resource_scheme",
            "strict schema diagnostics",
            "self.record_node_assets",
            "self.remove_surface_node_assets",
        ):
            self.assertIn(anchor, child)

        for index_owner_anchor in (
            "pub fn record_surface_assets",
            "pub fn record_compiled_surface",
            "pub fn record_binding_program",
            "pub fn record_node_assets",
            "pub fn remove_surface",
            "pub fn target_surfaces_for_plan",
            "pub fn target_nodes_for_plan",
            "pub fn mark_target_surfaces_dirty",
        ):
            self.assertIn(index_owner_anchor, owner)

    def test_node_resource_owner_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root
            / "docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md",
            repo_root / "docs/zircon_runtime/ui/architecture.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        runtime_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/ui/template/asset/surface_index.rs",
            "zircon_runtime/src/ui/template/asset/surface_index/node_resource_registration.rs",
            "tools/tests/test_runtime_ui_asset_surface_node_resource_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)


if __name__ == "__main__":
    unittest.main()
