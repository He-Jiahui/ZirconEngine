import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export import plugin_build_asset_pack, plugin_build_package


class Tooling03PluginBuildArtifactSingleProbePerformanceContractTests(unittest.TestCase):
    def test_native_artifact_uses_one_regular_file_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            target = root / "target"
            artifact = target / "release" / "plugin.dll"
            artifact.mkdir(parents=True)
            exists_calls = 0
            is_file_calls = 0
            original_exists = Path.exists
            original_is_file = Path.is_file

            def exists(path: Path) -> bool:
                nonlocal exists_calls
                if path == artifact:
                    exists_calls += 1
                return original_exists(path)

            def is_file(path: Path) -> bool:
                nonlocal is_file_calls
                if path == artifact:
                    is_file_calls += 1
                return original_is_file(path)

            with (
                mock.patch.object(Path, "exists", exists),
                mock.patch.object(Path, "is_file", is_file),
                mock.patch.object(
                    plugin_build_package,
                    "platform_dynamic_library_name",
                    return_value="plugin.dll",
                ),
            ):
                result = plugin_build_package.materialize_plugin_build_package(
                    out_root=root / "out",
                    package_id="plugin",
                    plugin_manifest_path=root / "plugin.toml",
                    package_manifest_text="",
                    repo_root=root,
                    target_dir=target,
                    dist_crate="plugin",
                    mode="release",
                    target_platform="windows-x86_64",
                    abi_version=3,
                    distribution={},
                    cargo="cargo",
                    locked=True,
                    offline=True,
                    packer=None,
                    signing_enabled=False,
                    signing_command_template=[],
                    signing_profile=None,
                    signing_platforms=[],
                    diagnostics=[],
                )

            self.assertIsNone(result)
            self.assertEqual(0, exists_calls)
            self.assertEqual(1, is_file_calls)

    def test_asset_pack_uses_one_regular_file_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            package_dir = root / "package"
            package_dir.mkdir()
            pack_path = package_dir / "plugin.zrpack"
            exists_calls = 0
            is_file_calls = 0
            original_exists = Path.exists
            original_is_file = Path.is_file

            def exists(path: Path) -> bool:
                nonlocal exists_calls
                if path == pack_path:
                    exists_calls += 1
                return original_exists(path)

            def is_file(path: Path) -> bool:
                nonlocal is_file_calls
                if path == pack_path:
                    is_file_calls += 1
                return original_is_file(path)

            def run_command(*_args, **_kwargs):
                pack_path.mkdir()
                return subprocess.CompletedProcess([], 0)

            with (
                mock.patch.object(Path, "exists", exists),
                mock.patch.object(Path, "is_file", is_file),
                mock.patch.object(
                    plugin_build_asset_pack,
                    "plugin_asset_pack_entries",
                    return_value=[{"path": "asset", "relative_path": "asset"}],
                ),
                mock.patch.object(
                    plugin_build_asset_pack,
                    "run_plugin_asset_pack_command",
                    side_effect=run_command,
                ),
            ):
                result = plugin_build_asset_pack.materialize_plugin_asset_pack(
                    package_id="plugin",
                    directory="plugin",
                    plugin_root=root,
                    repo_root=root,
                    package_dir=package_dir,
                    target_dir=root / "target",
                    distribution={},
                    cargo="cargo",
                    locked=True,
                    offline=True,
                    packer=None,
                    diagnostics=[],
                )

            self.assertFalse(result)
            self.assertEqual(0, exists_calls)
            self.assertEqual(1, is_file_calls)


if __name__ == "__main__":
    unittest.main()
