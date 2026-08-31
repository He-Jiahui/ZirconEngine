from __future__ import annotations

import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.platform_bundle_native_plugins_materialize import (
    copy_platform_bundle_native_plugins_dir,
)


class _StreamingChildren:
    def __init__(self, children: tuple[Path, ...]) -> None:
        self._children = children
        self._index = 0

    def __iter__(self) -> _StreamingChildren:
        return self

    def __next__(self) -> Path:
        if self._index >= len(self._children):
            raise StopIteration
        child = self._children[self._index]
        self._index += 1
        return child

    def __length_hint__(self) -> int:
        raise AssertionError("native plugins directory was materialized into a list")


class Tooling03PlatformBundleNativePluginsStreamingDirectoryPerformanceContractTests(
    unittest.TestCase
):
    def test_directory_children_are_consumed_as_a_stream(self) -> None:
        source = mock.Mock()
        source.__str__ = mock.Mock(return_value="native-plugins")
        source.iterdir.return_value = _StreamingChildren(
            (Path("plugin-a.dll"), Path("plugin-b.dll"))
        )
        destination = Path("bundle") / "plugins"
        diagnostics: list[str] = []

        with mock.patch.object(Path, "mkdir"), mock.patch.object(
            Path,
            "is_dir",
            return_value=False,
        ), mock.patch(
            "tools.zircon_export.platform_bundle_native_plugins_materialize.copy_platform_bundle_native_plugins_file",
            return_value=True,
        ) as copy_file:
            copied = copy_platform_bundle_native_plugins_dir(
                source,
                destination,
                diagnostics,
            )

        self.assertTrue(copied)
        self.assertEqual(diagnostics, [])
        self.assertEqual(copy_file.call_count, 2)

    def test_directory_iteration_error_preserves_diagnostic(self) -> None:
        source = mock.Mock()
        source.__str__ = mock.Mock(return_value="native-plugins")
        source.iterdir.side_effect = OSError("simulated listing failure")
        destination = Path("bundle") / "plugins"
        diagnostics: list[str] = []

        with mock.patch.object(Path, "mkdir"):
            copied = copy_platform_bundle_native_plugins_dir(
                source,
                destination,
                diagnostics,
            )

        self.assertFalse(copied)
        self.assertEqual(
            diagnostics,
            [
                "native plugins directory native-plugins could not be listed: "
                "simulated listing failure"
            ],
        )


if __name__ == "__main__":
    unittest.main()
