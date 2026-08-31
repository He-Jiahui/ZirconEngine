from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export import native_dynamic_materialize


class Tooling03NativeDynamicMaterializeManifestIndexPerformanceContractTests(
    unittest.TestCase
):
    def test_materialization_scans_fallback_package_manifests_once(self) -> None:
        package_count = 12
        package_ids = [f"package-{index:04d}" for index in range(package_count)]
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            plugin_root = root / "plugins"
            stage_dir = root / "stage"
            plugin_root.mkdir()
            package_exports: list[dict[str, object]] = []
            expected_sources: dict[str, Path] = {}
            for index, package_id in enumerate(package_ids):
                source = plugin_root / f"container-{index:04d}"
                source.mkdir()
                (source / "plugin.toml").write_text(
                    f'id = "{package_id}"\n',
                    encoding="utf-8",
                )
                expected_sources[package_id] = source
                package_exports.append(
                    {
                        "package_id": package_id,
                        "directory": package_id,
                    }
                )

            diagnostics: list[str] = []
            source_packages: dict[str, Path] = {}
            original_list_dir = native_dynamic_materialize.list_native_dynamic_dir
            original_manifest_read = (
                native_dynamic_materialize.read_package_manifest_id
            )
            with mock.patch.object(
                native_dynamic_materialize,
                "list_native_dynamic_dir",
                wraps=original_list_dir,
            ) as list_dir, mock.patch.object(
                native_dynamic_materialize,
                "read_package_manifest_id",
                wraps=original_manifest_read,
            ) as manifest_read, mock.patch.object(
                native_dynamic_materialize,
                "copy_native_dynamic_package",
                return_value=True,
            ), mock.patch.object(
                native_dynamic_materialize,
                "native_dynamic_package_loadable_artifacts",
                return_value=[],
            ):
                materialized = (
                    native_dynamic_materialize.materialize_native_dynamic_packages(
                        package_exports,
                        plugin_root,
                        stage_dir,
                        {".dll"},
                        {".dll"},
                        source_packages,
                        diagnostics,
                        require_source_native_artifacts=False,
                    )
                )

        self.assertEqual(diagnostics, [])
        self.assertEqual(len(materialized), package_count)
        self.assertEqual(source_packages, expected_sources)
        self.assertEqual(
            list_dir.call_count,
            package_count + 1,
            "fallback discovery must walk the plugin tree once",
        )
        self.assertEqual(
            manifest_read.call_count,
            package_count,
            "fallback discovery must parse each package manifest once",
        )

    def test_shared_scan_failure_preserves_package_specific_diagnostics(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            plugin_root = root / "plugins"
            plugin_root.mkdir()
            diagnostics: list[str] = []

            def fail_listing(
                label: str,
                directory: Path,
                listing_diagnostics: list[str],
            ) -> None:
                listing_diagnostics.append(
                    f"{label} {directory} could not be listed: access denied"
                )
                return None

            with mock.patch.object(
                native_dynamic_materialize,
                "list_native_dynamic_dir",
                side_effect=fail_listing,
            ) as list_dir:
                materialized = (
                    native_dynamic_materialize.materialize_native_dynamic_packages(
                        [
                            {"package_id": "animation", "directory": "animation"},
                            {"package_id": "audio", "directory": "audio"},
                        ],
                        plugin_root,
                        root / "stage",
                        {".dll"},
                        {".dll"},
                        {},
                        diagnostics,
                        require_source_native_artifacts=False,
                    )
                )

        self.assertEqual(materialized, [])
        self.assertEqual(list_dir.call_count, 1)
        self.assertEqual(
            diagnostics,
            [
                f"native dynamic package animation source search directory "
                f"{plugin_root} could not be listed: access denied",
                f"native dynamic package audio source search directory "
                f"{plugin_root} could not be listed: access denied",
            ],
        )


if __name__ == "__main__":
    unittest.main()
