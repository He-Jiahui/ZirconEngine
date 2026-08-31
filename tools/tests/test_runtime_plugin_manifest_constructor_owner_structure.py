import unittest
from pathlib import Path


class RuntimePluginManifestConstructorOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_06_15_plugin_manifest_constructor_owner_split_"
        "static_passed_cargo_deferred"
    )

    def test_module_and_package_constructors_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = repo_root / "zircon_runtime/src/plugin/package_manifest/constructors.rs"
        owner = owner_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(owner.splitlines()), 20)
        self.assertIn("mod module;", owner)
        self.assertIn("mod package;", owner)
        for moved_anchor in (
            "impl PluginModuleManifest",
            "impl PluginPackageManifest",
            "fn default_module_description",
            "fn default_package_coordinate_name",
        ):
            self.assertNotIn(moved_anchor, owner)

        owner_dir = owner_path.with_suffix("")
        child_contracts = {
            "module.rs": (
                210,
                "impl PluginModuleManifest",
                "pub fn module_descriptor",
                "fn default_module_description",
            ),
            "package.rs": (
                360,
                "impl PluginPackageManifest",
                "pub fn with_runtime_crate",
                "pub fn with_shader_module",
                "fn default_package_coordinate_name",
            ),
        }
        for filename, anchors in child_contracts.items():
            budget, *required = anchors
            child = (owner_dir / filename).read_text(encoding="utf-8")
            self.assertLessEqual(len(child.splitlines()), budget, filename)
            for anchor in required:
                self.assertIn(anchor, child, filename)

    def test_constructor_owner_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        structure_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/plugin/package_manifest/constructors.rs",
            "zircon_runtime/src/plugin/package_manifest/constructors/module.rs",
            "zircon_runtime/src/plugin/package_manifest/constructors/package.rs",
            "tools/tests/test_runtime_plugin_manifest_constructor_owner_structure.py",
        ):
            self.assertIn(current_path, structure_plan)


if __name__ == "__main__":
    unittest.main()
