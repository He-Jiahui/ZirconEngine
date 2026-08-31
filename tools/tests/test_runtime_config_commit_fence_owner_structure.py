import unittest
from pathlib import Path


class RuntimeConfigCommitFenceOwnerStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.owner = (
            self.repo_root
            / "zircon_runtime/src/foundation/runtime/config_manager/commit_fence.rs"
        )
        self.owner_dir = self.owner.with_suffix("")

    def test_config_commit_fence_uses_focused_folder_backed_owners(self) -> None:
        owner_source = self.owner.read_text(encoding="utf-8")
        production_lines = [
            line
            for line in owner_source.splitlines()
            if line.strip() and not line.lstrip().startswith("//")
        ]

        self.assertLessEqual(len(production_lines), 16)
        for declaration in (
            "mod fence;",
            "mod path_key;",
            "mod registry;",
            '#[cfg(test)]\nmod tests;',
        ):
            self.assertIn(declaration, owner_source)
        self.assertIn(
            "pub(in crate::foundation::runtime) use fence::ConfigCommitFence;",
            owner_source,
        )

        expected_children = {
            "fence.rs": (
                "pub(in crate::foundation::runtime) struct ConfigCommitFence",
                "pub(in crate::foundation::runtime::config_manager) fn register",
                "pub(in crate::foundation::runtime) fn commit",
                "pub(in crate::foundation::runtime::config_manager) fn cancel",
                "impl Drop for ConfigCommitFence",
                "struct CommitActiveGuard",
            ),
            "path_key.rs": (
                "pub(super) fn absolute_path",
                "fn normalize_platform_path",
            ),
            "registry.rs": (
                "static PATH_COMMIT_GATES",
                "pub(super) struct PathCommitEpoch",
                "pub(super) fn register_path_gate",
                "pub(super) fn reclaim_path_gate",
            ),
            "tests.rs": (
                "path_commit_gate_registry_reclaims_only_after_the_last_fence_drops",
                "path_commit_gate_registry_reclaim_release_benchmark",
                "FOUNDATION_PATH_GATE_RECLAIM_BENCH_V1",
            ),
        }
        for child_name, anchors in expected_children.items():
            child = self.owner_dir / child_name
            self.assertTrue(child.is_file(), child)
            child_source = child.read_text(encoding="utf-8")
            for anchor in anchors:
                self.assertIn(anchor, child_source)

        for forbidden in (
            "static PATH_COMMIT_GATES",
            "struct PathCommitEpoch",
            "struct ConfigCommitFence",
            "fn absolute_path",
            "struct CommitActiveGuard",
        ):
            self.assertNotIn(forbidden, owner_source)

        manager_source = (
            self.repo_root / "zircon_runtime/src/foundation/runtime/config_manager.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("use commit_fence::ConfigCommitFence;", manager_source)
        self.assertIn("ConfigCommitFence::register(&path)", manager_source)
        self.assertIn(
            "pub(super) use commit_fence::ConfigCommitFence as ConfigCommitFenceForTest;",
            manager_source,
        )


if __name__ == "__main__":
    unittest.main()
