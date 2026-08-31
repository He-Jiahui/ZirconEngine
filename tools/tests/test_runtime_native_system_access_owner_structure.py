import unittest
from pathlib import Path


class RuntimeNativeSystemAccessOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_06_15_native_system_access_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_authority_and_errors_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/plugin/native_plugin_loader/registration_manifest/system_access.rs"
        )
        owner = owner_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(owner.splitlines()), 330)
        self.assertIn("mod authority;", owner)
        self.assertIn("mod error;", owner)
        for moved_anchor in (
            "struct NativeSystemAccessAuthority",
            "enum NativeSystemAccessContractError",
            "enum NativeSystemAccessAuthorityError",
            "enum NativeSystemAccessResolveError",
            "fn required_capability",
        ):
            self.assertNotIn(moved_anchor, owner)
        for retained_anchor in (
            "struct NativeSystemAccessPlan",
            "fn from_manifest",
            "fn compile",
            "fn parse_access_declaration",
            "raw_access == [\"write:world\"]",
            "declarations.sort_by",
        ):
            self.assertIn(retained_anchor, owner)

        owner_dir = owner_path.with_suffix("")
        child_contracts = {
            "authority.rs": (
                110,
                "struct NativeSystemAccessAuthority",
                "fn authorize",
                "fn required_capability",
                "NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY",
            ),
            "error.rs": (
                170,
                "enum NativeSystemAccessContractError",
                "enum NativeSystemAccessAuthorityError",
                "enum NativeSystemAccessResolveError",
                "impl std::error::Error",
            ),
        }
        for filename, anchors in child_contracts.items():
            budget, *required = anchors
            child = (owner_dir / filename).read_text(encoding="utf-8")
            self.assertLessEqual(len(child.splitlines()), budget, filename)
            for anchor in required:
                self.assertIn(anchor, child, filename)

    def test_system_access_owner_status_is_mirrored(self) -> None:
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
            "zircon_runtime/src/plugin/native_plugin_loader/registration_manifest/system_access.rs",
            "zircon_runtime/src/plugin/native_plugin_loader/registration_manifest/system_access/authority.rs",
            "zircon_runtime/src/plugin/native_plugin_loader/registration_manifest/system_access/error.rs",
            "tools/tests/test_runtime_native_system_access_owner_structure.py",
        ):
            self.assertIn(current_path, structure_plan)


if __name__ == "__main__":
    unittest.main()
