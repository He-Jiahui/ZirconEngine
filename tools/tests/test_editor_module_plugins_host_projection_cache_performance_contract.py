from pathlib import Path
import unittest

from tools.editor_module_plugins_host_projection_cache_pressure import run


ROOT = Path(__file__).resolve().parents[2]
CONVERSION = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "ui"
    / "pane_data_conversion"
    / "module_plugins.rs"
)
CACHE = CONVERSION.parent / "module_plugins" / "cache.rs"
PANE_CONVERSION = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "ui"
    / "apply_presentation"
    / "pane_conversion.rs"
)
HOST_DATA = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "host_contract"
    / "data"
    / "panes"
    / "module_plugins.rs"
)


class EditorModulePluginsHostProjectionCachePerformanceContractTests(unittest.TestCase):
    def test_host_contract_does_not_retain_an_unconsumed_plugin_status_mirror(self) -> None:
        host_data = HOST_DATA.read_text(encoding="utf-8")
        conversion = CONVERSION.read_text(encoding="utf-8")

        self.assertNotIn("struct ModulePluginStatusData", host_data)
        self.assertNotIn("pub plugins:", host_data)
        self.assertNotIn("fn to_host_contract_module_plugin_status", conversion)
        cache_path = conversion.split(
            "fn to_host_contract_module_plugins_pane_from_host_pane_with_cache", 1
        )[1].split("fn module_plugins_projection_cache_key", 1)[0]
        self.assertNotIn("map_model_rc", cache_path)

    def test_pressure_model_bounds_stable_work_by_projection_generation(self) -> None:
        report = run(
            plugin_count=1_000,
            presentation_apply_count=4_096,
            projection_generation_change_count=64,
        )

        self.assertEqual(
            report["retired_full_projection"]["total_source_row_reads"], 8_192_000
        )
        self.assertEqual(
            report["generation_cached_projection"]["total_source_row_reads"],
            65_000,
        )
        self.assertEqual(report["delta"]["avoided_source_row_reads"], 8_127_000)
        self.assertEqual(
            report["generation_cached_projection"]["stable_lookup_source_row_reads"],
            0,
        )
        self.assertFalse(report["interpretation"]["runtime_cpu_measured"])

    def test_stable_generation_uses_storage_identity_before_full_projection(self) -> None:
        source = CACHE.read_text(encoding="utf-8")

        self.assertIn("struct ModulePluginsPaneProjectionCache", source)
        self.assertIn("source_plugins.shares_values_with(plugins)", source)
        self.assertIn("document_identity", source)
        self.assertIn("width_bits", source)
        self.assertIn("height_bits", source)

        cached = source.split("fn cached", 1)[1].split("fn store", 1)[0]
        self.assertNotIn(".iter()", cached)
        self.assertNotIn("row_count()", cached)

    def test_conversion_returns_cached_host_models_before_building_rows(self) -> None:
        source = CONVERSION.read_text(encoding="utf-8")
        function = source.split(
            "fn to_host_contract_module_plugins_pane_from_host_pane_with_cache", 1
        )[1].split("fn to_host_contract_module_plugin_status", 1)[0]

        cache_lookup = function.index("cache.cached(")
        template_projection = function.index("module_plugins_template_projection(")
        row_projection = function.index("module_plugin_row_nodes(")
        self.assertLess(cache_lookup, template_projection)
        self.assertLess(cache_lookup, row_projection)
        self.assertNotIn("map_model_rc(", function)
        self.assertIn("cache.store(", function)
        self.assertIn("ui.module_plugins.host_projection_cache_hit_count", function)
        self.assertIn("ui.module_plugins.host_projection_cache_miss_count", function)
        self.assertIn("ui.module_plugins.host_projection_source_row_count", function)
        cache_key = source.split("fn module_plugins_projection_cache_key", 1)[1].split(
            "fn module_plugins_template_projection", 1
        )[0]
        self.assertIn("PanePayload::ModulePluginsV1", cache_key)

    def test_pane_conversion_uses_the_long_lived_module_plugins_cache(self) -> None:
        source = PANE_CONVERSION.read_text(encoding="utf-8")
        module_branch = source.split("let module_plugins =", 1)[1].split(
            "let runtime_diagnostics =", 1
        )[0]

        self.assertIn(
            "to_host_contract_module_plugins_pane_from_host_pane_with_cache",
            module_branch,
        )
        self.assertIn("module_plugins_projection_cache", module_branch)


if __name__ == "__main__":
    unittest.main()
