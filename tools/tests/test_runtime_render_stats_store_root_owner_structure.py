import unittest
from pathlib import Path


class RuntimeRenderStatsStoreRootOwnerStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.owner_dir = (
            self.repo_root
            / "zircon_runtime/src/core/runtime/diagnostics/render_stats_store"
        )
        self.owner = self.owner_dir / "mod.rs"
        self.legacy_owner = self.owner_dir.with_suffix(".rs")

    def test_render_stats_store_root_is_structural(self) -> None:
        self.assertFalse(self.legacy_owner.exists(), self.legacy_owner)
        owner_source = self.owner.read_text(encoding="utf-8")
        production_lines = [
            line
            for line in owner_source.splitlines()
            if line.strip() and not line.lstrip().startswith("//")
        ]

        self.assertLessEqual(len(production_lines), 24)
        for declaration in (
            "mod advanced_provider;",
            "mod anti_alias;",
            "mod capability;",
            "mod dispatch;",
            "mod graph;",
            "mod history;",
            "mod hybrid_gi;",
            "mod measurement;",
            "mod particle;",
            "mod post_process;",
            "mod product;",
            "mod profile;",
            "mod shader_variant;",
            "mod solari;",
            "mod virtual_geometry;",
            "mod volumetric_fog;",
            '#[cfg(test)]\nmod tests;',
        ):
            self.assertIn(declaration, owner_source)
        self.assertIn(
            "use measurement::{record_bool, record_bytes, record_count, record_microseconds};",
            owner_source,
        )
        self.assertIn(
            "pub(crate) use dispatch::record_render_stats_diagnostics;",
            owner_source,
        )
        for forbidden in (
            "fn record_",
            "RenderStats",
            "store.record",
            "::record(store, stats)",
        ):
            self.assertNotIn(forbidden, owner_source)

        dispatch_source = (self.owner_dir / "dispatch.rs").read_text(
            encoding="utf-8"
        )
        ordered_calls = (
            "capability::record(store, stats);",
            "history::record(store, stats);",
            "graph::record(store, stats);",
            "profile::record(store, stats);",
            "product::record(store, stats);",
            "shader_variant::record(store, stats);",
            "post_process::record(store, stats);",
            "anti_alias::record(store, stats);",
            "particle::record(store, stats);",
            "virtual_geometry::record(store, stats);",
            "hybrid_gi::record(store, stats);",
            "volumetric_fog::record(store, stats);",
            "advanced_provider::record(store, stats);",
            "solari::record(store, stats);",
        )
        positions = [dispatch_source.index(call) for call in ordered_calls]
        self.assertEqual(positions, sorted(positions))

        measurement_source = (self.owner_dir / "measurement.rs").read_text(
            encoding="utf-8"
        )
        for helper in (
            "pub(super) fn record_count",
            "pub(super) fn record_bytes",
            "pub(super) fn record_microseconds",
            "pub(super) fn record_bool",
        ):
            self.assertIn(helper, measurement_source)
        self.assertEqual(measurement_source.count("store.record_static("), 4)
        self.assertNotIn("store.record(", measurement_source)

        tests_source = (self.owner_dir / "tests.rs").read_text(encoding="utf-8")
        self.assertIn('include_str!("measurement.rs")', tests_source)
        self.assertIn(
            "render_stats_helpers_use_static_metadata_recording", tests_source
        )
        self.assertIn(
            "render_stats_product_leaves_use_static_metadata_recording", tests_source
        )

        docs_source = (
            self.repo_root / "docs/zircon_runtime/core/diagnostics.md"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/mod.rs",
            docs_source,
        )
        self.assertIn(
            "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/measurement.rs",
            docs_source,
        )
        self.assertNotIn(
            "zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs",
            docs_source,
        )


if __name__ == "__main__":
    unittest.main()
