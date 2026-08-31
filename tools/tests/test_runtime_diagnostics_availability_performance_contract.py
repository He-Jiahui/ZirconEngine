from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class RuntimeDiagnosticsAvailabilityPerformanceContract(unittest.TestCase):
    def test_render_framework_exposes_compatible_boolean_query(self) -> None:
        framework = source("zircon_runtime/src/core/framework/render/framework.rs")

        self.assertIn("fn query_virtual_geometry_debug_snapshot_available", framework)
        self.assertIn("self.query_virtual_geometry_debug_snapshot()", framework)

    def test_wgpu_boolean_query_only_checks_retained_snapshot_presence(self) -> None:
        query = source(
            "zircon_runtime/src/graphics/runtime/render_framework/"
            "query_virtual_geometry_debug_snapshot/query_virtual_geometry_debug_snapshot.rs"
        )
        start = query.index("fn query_virtual_geometry_debug_snapshot_available")
        body = query[start:query.index("#[cfg(test)]", start)]

        self.assertIn("last_virtual_geometry_debug_snapshot.is_some()", body)
        self.assertNotIn(".clone()", body)
        self.assertNotIn(".cloned()", body)

    def test_facade_uses_boolean_query_instead_of_owned_snapshot(self) -> None:
        collect = source("zircon_runtime/src/runtime_diagnostics/collect.rs")
        start = collect.index("fn collect_render_diagnostics")
        end = collect.index("fn collect_physics_diagnostics", start)
        render = collect[start:end]

        self.assertIn("query_virtual_geometry_debug_snapshot_available()", render)
        self.assertNotIn("query_virtual_geometry_debug_snapshot()", render)


if __name__ == "__main__":
    unittest.main()
