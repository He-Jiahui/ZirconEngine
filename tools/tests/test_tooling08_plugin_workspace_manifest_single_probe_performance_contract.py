import unittest
from pathlib import Path
from unittest import mock

from tools import zircon_build_plugin_workspace_crates as workspace_crates


class Tooling08PluginWorkspaceManifestSingleProbePerformanceContractTests(
    unittest.TestCase
):
    def test_existing_member_manifest_is_opened_without_exists_probe(self) -> None:
        plugins_root = Path("zircon_plugins")
        members = [f"plugin_{index}/runtime" for index in range(64)]
        exists_calls = 0
        reads = 0

        def exists(_path: Path) -> bool:
            nonlocal exists_calls
            exists_calls += 1
            return True

        def read_toml(path: Path) -> dict:
            nonlocal reads
            reads += 1
            if path == plugins_root / "Cargo.toml":
                return {"workspace": {"members": members}}
            return {
                "package": {"name": path.parent.parent.name},
                "lib": {"crate-type": ["rlib"]},
            }

        with (
            mock.patch.object(Path, "exists", exists),
            mock.patch.object(workspace_crates, "_read_toml", side_effect=read_toml),
        ):
            packages = workspace_crates.discover_plugin_workspace_crates(plugins_root)

        self.assertEqual(64, len(packages))
        self.assertEqual(65, reads)
        self.assertEqual(0, exists_calls)

    def test_missing_members_are_skipped_but_other_io_errors_propagate(self) -> None:
        plugins_root = Path("zircon_plugins")

        def missing_member(path: Path) -> dict:
            if path == plugins_root / "Cargo.toml":
                return {"workspace": {"members": ["missing/runtime"]}}
            raise FileNotFoundError(path)

        with mock.patch.object(
            workspace_crates, "_read_toml", side_effect=missing_member
        ):
            self.assertEqual(
                (), workspace_crates.discover_plugin_workspace_crates(plugins_root)
            )

        def denied_member(path: Path) -> dict:
            if path == plugins_root / "Cargo.toml":
                return {"workspace": {"members": ["denied/runtime"]}}
            raise PermissionError(path)

        with mock.patch.object(
            workspace_crates, "_read_toml", side_effect=denied_member
        ):
            with self.assertRaises(PermissionError):
                workspace_crates.discover_plugin_workspace_crates(plugins_root)


if __name__ == "__main__":
    unittest.main()
