from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.platform_bundle_native_plugins_materialize import (
    materialize_platform_bundle_native_plugins,
)


class Tooling03PlatformBundleNativePluginsSingleProbePerformanceContractTests(
    unittest.TestCase
):
    def test_valid_plugins_root_uses_one_metadata_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            plugins = root / "native-plugins"
            plugins.mkdir()
            observed_plugins = mock.Mock(wraps=plugins)
            observed_plugins.__str__ = mock.Mock(return_value=str(plugins))
            fatal, destination, copied = materialize_platform_bundle_native_plugins(
                bundle_root=root / "bundle",
                native_plugins_dir=observed_plugins,
                copied_template_files=[],
                diagnostics=[],
            )

        self.assertFalse(fatal)
        self.assertIsNotNone(destination)
        self.assertEqual(copied, [])
        probes = (
            observed_plugins.exists.call_count
            + observed_plugins.is_dir.call_count
            + observed_plugins.stat.call_count
        )
        self.assertEqual(probes, 1)

    def test_plugins_root_inspection_error_is_not_reported_as_missing(self) -> None:
        plugins = Path("native-plugins")
        diagnostics: list[str] = []
        observed_plugins = mock.Mock(wraps=plugins)
        observed_plugins.__str__ = mock.Mock(return_value=str(plugins))
        observed_plugins.stat.side_effect = PermissionError(
            "simulated permission failure"
        )
        fatal, destination, copied = materialize_platform_bundle_native_plugins(
            bundle_root=Path("bundle"),
            native_plugins_dir=observed_plugins,
            copied_template_files=[],
            diagnostics=diagnostics,
        )

        self.assertTrue(fatal)
        self.assertIsNone(destination)
        self.assertEqual(copied, [])
        self.assertEqual(
            diagnostics,
            [
                f"native plugins directory {plugins} could not be inspected: "
                "simulated permission failure"
            ],
        )


if __name__ == "__main__":
    unittest.main()
