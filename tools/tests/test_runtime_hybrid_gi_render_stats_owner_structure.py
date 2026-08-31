import unittest
from pathlib import Path


class RuntimeHybridGiRenderStatsOwnerStructureTests(unittest.TestCase):
    def test_hybrid_gi_metric_families_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/hybrid_gi.rs"
        )
        owner = owner_path.read_text(encoding="utf-8")
        owner_dir = owner_path.with_suffix("")
        children = {
            name: (owner_dir / f"{name}.rs").read_text(encoding="utf-8")
            for name in (
                "global_sdf",
                "payload_source",
                "probe_cache",
                "scene_surface_cache",
                "voxel_cache",
            )
        }

        self.assertLessEqual(len(owner.splitlines()), 30)
        for name in children:
            self.assertIn(f"mod {name};", owner)
        self._assert_anchors_are_ordered(
            owner,
            (
                "probe_cache::record(store, stats);",
                "scene_surface_cache::record(store, stats);",
                "voxel_cache::record(store, stats);",
                "global_sdf::record(store, stats);",
                "payload_source::record(store, stats);",
            ),
        )

        for moved_anchor in (
            '"render.hybrid_gi.active_probe_count"',
            '"render.hybrid_gi.scene.card_count"',
            '"render.hybrid_gi.voxel.resident_clipmap_count"',
            '"render.hybrid_gi.global_sdf.cpu_prepare_time_us"',
            '"render.hybrid_gi.payload.source.none"',
        ):
            self.assertNotIn(moved_anchor, owner)

        expected_anchors = {
            "probe_cache": '"render.hybrid_gi.active_probe_count"',
            "scene_surface_cache": '"render.hybrid_gi.scene.card_count"',
            "voxel_cache": '"render.hybrid_gi.voxel.resident_clipmap_count"',
            "global_sdf": '"render.hybrid_gi.global_sdf.cpu_prepare_time_us"',
            "payload_source": '"render.hybrid_gi.payload.source.none"',
        }
        budgets = {
            "probe_cache": 70,
            "scene_surface_cache": 190,
            "voxel_cache": 40,
            "global_sdf": 280,
            "payload_source": 40,
        }
        for name, source in children.items():
            self.assertIn("pub(super) fn record", source)
            self.assertIn(expected_anchors[name], source)
            self.assertLessEqual(len(source.splitlines()), budgets[name])

    def _assert_anchors_are_ordered(self, source: str, anchors: tuple[str, ...]) -> None:
        positions = [source.index(anchor) for anchor in anchors]
        self.assertEqual(positions, sorted(positions))


if __name__ == "__main__":
    unittest.main()
