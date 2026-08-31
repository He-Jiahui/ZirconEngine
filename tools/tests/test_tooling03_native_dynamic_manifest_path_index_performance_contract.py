from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export import pipeline_report_native_dynamic_build_execution as build_execution


class CountingPathIndex(set[str]):
    contains_calls = 0
    iterated_paths = 0

    def __contains__(self, value: object) -> bool:
        type(self).contains_calls += 1
        return super().__contains__(value)

    def __iter__(self):
        for value in super().__iter__():
            type(self).iterated_paths += 1
            yield value


class NativeDynamicManifestPathIndexPerformanceContractTests(unittest.TestCase):
    def test_manifest_path_index_covers_files_and_parent_directories(self) -> None:
        path_index = build_execution.native_dynamic_file_manifest_path_index(
            [
                {
                    "path": (
                        "plugins/animation/native/debug/"
                        "zircon_plugin_animation.pdb"
                    )
                },
                {"path": "plugins/animation/plugin.toml"},
                {"path": 7},
            ]
        )

        self.assertIn(
            "plugins/animation/native/debug/zircon_plugin_animation.pdb",
            path_index,
        )
        self.assertIn("plugins/animation/native/debug", path_index)
        self.assertIn("plugins/animation/native", path_index)
        self.assertIn("plugins/animation", path_index)
        self.assertIn("plugins", path_index)
        self.assertNotIn("plugins/audio", path_index)

    def test_path_or_directory_lookup_does_not_iterate_manifest_paths(self) -> None:
        path_index = CountingPathIndex(
            f"plugins/package_{index:05d}/native/artifact.dll"
            for index in range(5_000)
        )
        CountingPathIndex.contains_calls = 0
        CountingPathIndex.iterated_paths = 0

        found = build_execution.native_dynamic_file_manifest_contains_path_or_directory(
            "plugins/missing/native/sidecar.pdb",
            path_index,
        )

        self.assertFalse(found)
        self.assertLessEqual(CountingPathIndex.contains_calls, 1)
        self.assertEqual(CountingPathIndex.iterated_paths, 0)

    def test_plugins_root_is_resolved_once_for_the_package_batch(self) -> None:
        package_count = 200
        with tempfile.TemporaryDirectory() as temp_dir:
            plugins_dir = Path(temp_dir) / "plugins"
            packages = [
                {
                    "package_id": f"package_{index:05d}",
                    "destination": plugins_dir / f"package_{index:05d}",
                }
                for index in range(package_count)
            ]
            original_resolve = Path.resolve
            plugins_dir_resolve_calls = 0

            def counted_resolve(path: Path, *args: object, **kwargs: object) -> Path:
                nonlocal plugins_dir_resolve_calls
                if path == plugins_dir:
                    plugins_dir_resolve_calls += 1
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(
                Path,
                "resolve",
                autospec=True,
                side_effect=counted_resolve,
            ):
                package_dirs = (
                    build_execution.materialized_package_native_dir_paths(
                        packages,
                        plugins_dir,
                    )
                )

        self.assertEqual(len(package_dirs), package_count)
        self.assertLessEqual(plugins_dir_resolve_calls, 1)


if __name__ == "__main__":
    unittest.main()
