from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export import native_dynamic_payload_directory as payload_directory
from tools.zircon_export import (
    pipeline_report_native_dynamic_stage_payload_operation_audit as operation_audit,
)


class CountingPrefixIndex(dict[str, list[str]]):
    get_calls = 0
    iterated_keys = 0

    def get(self, key: str, default: object = None):
        type(self).get_calls += 1
        return super().get(key, default)

    def __iter__(self):
        for key in super().__iter__():
            type(self).iterated_keys += 1
            yield key


class NativeDynamicPayloadIndexPerformanceContractTests(unittest.TestCase):
    def test_loadable_artifact_prefix_index_preserves_sorted_descendants(self) -> None:
        prefix_index = payload_directory.native_dynamic_loadable_artifact_prefix_index(
            {
                "plugins/animation/native/z.dll",
                "plugins/animation/native/debug/a.pdb",
                "plugins/audio/native/m.dll",
            }
        )

        self.assertEqual(
            prefix_index["plugins/animation/"],
            [
                "plugins/animation/native/debug/a.pdb",
                "plugins/animation/native/z.dll",
            ],
        )
        self.assertEqual(
            prefix_index["plugins/animation/native/debug/"],
            ["plugins/animation/native/debug/a.pdb"],
        )
        self.assertEqual(
            prefix_index["plugins/audio/"],
            ["plugins/audio/native/m.dll"],
        )

    def test_manifest_match_uses_one_prefix_index_lookup_per_package(self) -> None:
        package_count = 200
        prefix_index = CountingPrefixIndex(
            {
                f"plugins/package_{index:05d}/": [
                    f"plugins/package_{index:05d}/native/artifact.dll"
                ]
                for index in range(package_count)
            }
        )
        materialized_packages = [
            {
                "destination": f"C:/bundle/plugins/package_{index:05d}",
                "loadable_artifacts": [
                    f"plugins/package_{index:05d}/native/artifact.dll"
                ],
            }
            for index in range(package_count)
        ]
        file_manifest = [
            {"path": f"plugins/package_{index:05d}/native/artifact.dll"}
            for index in range(package_count)
        ]
        CountingPrefixIndex.get_calls = 0
        CountingPrefixIndex.iterated_keys = 0

        with mock.patch.object(
            payload_directory,
            "native_dynamic_loadable_artifact_prefix_index",
            return_value=prefix_index,
        ), mock.patch.object(
            payload_directory,
            "resolve_native_dynamic_payload_path",
            side_effect=lambda _label, path, _diagnostics: path,
        ):
            matches = payload_directory.materialized_package_loadable_artifacts_match_manifest(
                materialized_packages,
                file_manifest,
                Path("C:/bundle/plugins"),
            )

        self.assertTrue(matches)
        self.assertEqual(CountingPrefixIndex.get_calls, package_count)
        self.assertEqual(CountingPrefixIndex.iterated_keys, 0)

    def test_manifest_match_resolves_plugins_root_once_for_package_batch(self) -> None:
        package_count = 200
        plugins_dir = Path("C:/bundle/plugins")
        materialized_packages = [
            {
                "destination": plugins_dir / f"package_{index:05d}",
                "loadable_artifacts": [],
            }
            for index in range(package_count)
        ]
        plugins_root_resolve_calls = 0

        def counted_resolve(
            label: str,
            path: Path,
            _diagnostics: list[str] | None,
        ) -> Path:
            nonlocal plugins_root_resolve_calls
            if label == "NativeDynamic payload plugins_dir":
                plugins_root_resolve_calls += 1
            return path

        with mock.patch.object(
            payload_directory,
            "resolve_native_dynamic_payload_path",
            side_effect=counted_resolve,
        ):
            matches = payload_directory.materialized_package_loadable_artifacts_match_manifest(
                materialized_packages,
                [],
                plugins_dir,
            )

        self.assertTrue(matches)
        self.assertLessEqual(plugins_root_resolve_calls, 1)

    def test_operation_audit_resolves_plugins_root_once_for_package_batch(self) -> None:
        package_count = 200
        with tempfile.TemporaryDirectory() as temp_dir:
            plugins_dir = Path(temp_dir) / "plugins"
            packages = [
                {
                    "package_id": f"package_{index:05d}",
                    "destination": plugins_dir / f"package_{index:05d}",
                    "loadable_artifacts": [],
                }
                for index in range(package_count)
            ]
            original_resolve = Path.resolve
            plugins_root_resolve_calls = 0

            def counted_resolve(path: Path, *args: object, **kwargs: object) -> Path:
                nonlocal plugins_root_resolve_calls
                if path == plugins_dir:
                    plugins_root_resolve_calls += 1
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(
                Path,
                "resolve",
                autospec=True,
                side_effect=counted_resolve,
            ):
                artifacts = operation_audit.materialized_package_relative_artifacts(
                    packages,
                    plugins_dir,
                )

        self.assertEqual(len(artifacts), package_count)
        self.assertLessEqual(plugins_root_resolve_calls, 1)


if __name__ == "__main__":
    unittest.main()
