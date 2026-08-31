from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
DAMAGE_REGION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/redraw/damage_region.rs"
)
REDRAW_MERGE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/redraw/request/merge.rs"
)
REDRAW_QUERY = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/redraw/request/query.rs"
)
EVENT_LOOP_REDRAW = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs"
)
PRESENT_REDRAW = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs"
)
REDRAW_CONSTRUCTORS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/redraw/request/constructors.rs"
)
UI_PERF = REPO_ROOT / "zircon_editor/src/ui/retained_host/ui_perf.rs"
UI_PERF_COUNTER_CATALOG = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/ui_perf/counter_catalog.rs"
)
REDRAW_TESTS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/redraw_tests.rs"
)


class EditorBoundedDamageRegionPerformanceContract(unittest.TestCase):
    def test_damage_region_has_fixed_three_rect_capacity_without_heap_storage(self) -> None:
        source = DAMAGE_REGION.read_text(encoding="utf-8")

        self.assertIn("const DAMAGE_RECT_CAPACITY: usize = 3;", source)
        self.assertIn("frames: [FrameRect; DAMAGE_RECT_CAPACITY]", source)
        self.assertNotIn("Vec<", source)
        self.assertNotIn("HashMap", source)

    def test_redraw_merge_preserves_bounded_region_instead_of_immediate_union(self) -> None:
        source = REDRAW_MERGE.read_text(encoding="utf-8")
        region_branch = source.split("damage: current", 1)[1].split(
            "(Self::None", 1
        )[0]

        self.assertIn("damage: current.merge(next)", region_branch)
        self.assertNotIn("union_frame", source)

    def test_presenter_behavior_still_uses_the_legacy_bounding_frame(self) -> None:
        query = REDRAW_QUERY.read_text(encoding="utf-8")
        event_loop = EVENT_LOOP_REDRAW.read_text(encoding="utf-8")
        present = PRESENT_REDRAW.read_text(encoding="utf-8")

        self.assertIn("Some(damage.bounding_frame())", query)
        self.assertIn("redraw: HostRedrawRequest", present)
        self.assertIn("let damage_region = redraw.damage_region().cloned();", present)
        self.assertIn(
            "presenter.present(\n        presentation,\n        presentation_cursor,\n        damage_region.clone(),\n        invalidation,\n    )",
            present,
        )
        self.assertIn(
            "present_redraw(self, event_loop, redraw, present_scenario)", event_loop
        )

    def test_retry_preserves_bounded_damage_and_only_clears_frame_update(self) -> None:
        present = PRESENT_REDRAW.read_text(encoding="utf-8")
        constructors = REDRAW_CONSTRUCTORS.read_text(encoding="utf-8")
        retry = constructors.split("fn into_present_retry", 1)[1]

        self.assertIn("retry_present_request(scenario, redraw)", present)
        self.assertNotIn("damage_region: Option<FrameRect>", present)
        self.assertIn("Self::Region { damage, .. }", retry)
        self.assertIn("damage,\n                    frame_update: false,", retry)
        self.assertIn("Self::full_frame_for_scenario(scenario, false)", retry)

    def test_final_present_batch_records_all_damage_region_metrics(self) -> None:
        source = EVENT_LOOP_REDRAW.read_text(encoding="utf-8")
        ui_perf = UI_PERF.read_text(encoding="utf-8")
        counter_catalog = UI_PERF_COUNTER_CATALOG.read_text(encoding="utf-8")
        body = source.split("fn record_damage_region_metrics", 1)[1].split(
            "fn surface_present_retry_delay", 1
        )[0]
        counters = (
            ("RedrawDamageRectCount", "redraw_damage_rect_count"),
            ("RedrawDamageSourceRectCount", "redraw_damage_source_rect_count"),
            (
                "RedrawDamageSimplificationCount",
                "redraw_damage_simplification_count",
            ),
            ("RedrawDamageRepresentedArea", "redraw_damage_represented_area"),
            ("RedrawDamageBoundingArea", "redraw_damage_bounding_area"),
            (
                "RedrawDamageBoundingOverdrawArea",
                "redraw_damage_bounding_overdraw_area",
            ),
        )

        for counter, suffix in counters:
            self.assertEqual(body.count(counter), 1)
            self.assertIn(f"{counter},", counter_catalog)
            self.assertRegex(
                ui_perf,
                rf'UiPerfCounter::{counter} => \{{\s*'
                rf'concat!\(\$prefix, "\.{suffix}"\)\s*\}}',
            )

        redraw_impl = source.split("fn redraw_requested_impl", 1)[1].split(
            "fn take_pending_redraw", 1
        )[0]
        final_batch = redraw_impl.index("let redraw = self.take_redraw_for_present();")
        metrics_call = redraw_impl.index(
            "record_damage_region_metrics(&redraw, present_scenario);"
        )
        present_call = redraw_impl.index(
            "present_redraw(self, event_loop, redraw, present_scenario);"
        )
        self.assertEqual(
            redraw_impl.count(
                "record_damage_region_metrics(&redraw, present_scenario);"
            ),
            1,
        )
        self.assertLess(final_batch, metrics_call)
        self.assertLess(metrics_call, present_call)

    def test_rust_regressions_cover_sparse_compaction_and_overlap_area(self) -> None:
        source = REDRAW_TESTS.read_text(encoding="utf-8")
        damage = DAMAGE_REGION.read_text(encoding="utf-8")
        present = PRESENT_REDRAW.read_text(encoding="utf-8")

        self.assertIn("redraw_region_retains_distant_damage", source)
        self.assertIn("redraw_region_eliminates_contained_damage", source)
        self.assertIn("redraw_region_simplifies_the_fourth_rect", source)
        self.assertIn("redraw_region_reports_exact_overlap_area", source)
        self.assertIn("redraw_region_preserves_the_legacy_f32_bounding_merge_order", source)
        self.assertIn("retryable_surface_present_preserves_bounded_damage_pressure", present)
        self.assertIn(
            "let bounding_frame = union_frame(&self.bounding_frame, &next.bounding_frame);",
            damage,
        )
        self.assertNotIn("fn recompute_bounding_frame", damage)


if __name__ == "__main__":
    unittest.main()
