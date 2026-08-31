from __future__ import annotations

import unittest

from tools.zircon_build_plugin_selection import select_plugins


class ProbePackage:
    def __init__(
        self,
        plugin_id: str,
        *,
        native: bool = False,
        static: bool = False,
    ) -> None:
        self._plugin_id = plugin_id
        self._native = native
        self._static = static
        self.plugin_id_reads = 0
        self.native_reads = 0
        self.static_reads = 0

    @property
    def plugin_id(self) -> str:
        self.plugin_id_reads += 1
        return self._plugin_id

    @property
    def native_dynamic_crates(self) -> tuple[object, ...]:
        self.native_reads += 1
        return (self,) if self._native else ()

    @property
    def rlib_static_crates(self) -> tuple[object, ...]:
        self.static_reads += 1
        return (self,) if self._static else ()


class PluginSelectionLazyIndexPerformanceContractTests(unittest.TestCase):
    def test_numeric_selection_does_not_build_the_id_index(self) -> None:
        packages = [ProbePackage(f"plugin-{index}") for index in range(256)]

        selected = select_plugins(packages, "128")

        self.assertEqual([packages[127]], selected)
        self.assertEqual(1, sum(package.plugin_id_reads for package in packages))

    def test_repeated_native_selectors_reuse_one_projection(self) -> None:
        packages = [
            ProbePackage(f"plugin-{index}", native=index % 2 == 0)
            for index in range(256)
        ]

        selected = select_plugins(packages, "native,native_dynamic,native")

        self.assertEqual(packages[::2], selected)
        self.assertEqual(256, sum(package.native_reads for package in packages))

    def test_repeated_static_selectors_reuse_one_projection(self) -> None:
        packages = [
            ProbePackage(f"plugin-{index}", static=index % 2 == 1)
            for index in range(256)
        ]

        selected = select_plugins(packages, "static,rlib,rlib_static")

        self.assertEqual(packages[1::2], selected)
        self.assertEqual(256, sum(package.static_reads for package in packages))


if __name__ == "__main__":
    unittest.main()
