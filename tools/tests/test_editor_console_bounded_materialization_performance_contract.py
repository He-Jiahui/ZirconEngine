from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CONSOLE_METADATA = ROOT / "zircon_editor/src/ui/retained_host/console_output.rs"
CONSOLE_PROJECTION = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/console_projection.rs"
)
CONSOLE_PAINT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/"
    "docks/pane/template_nodes/console_output.rs"
)
CONSOLE_HIT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
    "template_node/hit.rs"
)
UI_PERF = ROOT / "zircon_editor/src/ui/retained_host/ui_perf.rs"
CONSOLE_SNAPSHOT = ROOT / (
    "zircon_editor/src/ui/workbench/snapshot/data/console_output_snapshot.rs"
)
CONSOLE_SNAPSHOT_GENERATION = ROOT / (
    "zircon_editor/src/ui/workbench/snapshot/data/console_output_snapshot/generation.rs"
)
CONSOLE_HISTORY = ROOT / "zircon_editor/src/ui/workbench/state/console_history.rs"
ACTIVITY_LOG_PROJECTION = ROOT / (
    "zircon_editor/src/ui/workbench/activity_log_console_projection.rs"
)
CONSOLE_PAYLOAD = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs"
)
CONSOLE_PAYLOAD_BUILDER = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "pane_payload_builders/console.rs"
)
RETAINED_HOST_APP = ROOT / "zircon_editor/src/ui/retained_host/app.rs"
APPLY_PRESENTATION = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/apply_presentation.rs"
)
CONSOLE_POINTER_LAYOUT = ROOT / (
    "zircon_editor/src/ui/retained_host/app/pointer_layout/detail_scrolls.rs"
)


class EditorConsoleBoundedMaterializationPerformanceContractTests(unittest.TestCase):
    def test_projection_materializes_only_the_bounded_slot_budget(self) -> None:
        source = CONSOLE_PROJECTION.read_text(encoding="utf-8")
        projection = source.split("fn project_console_output_lines", 1)[1].split(
            "fn console_output_text_tone", 1
        )[0]

        self.assertIn("ConsoleOutputSnapshot", projection)
        self.assertIn("new_virtualized_snapshot", projection)
        self.assertIn("materialized_line_count()", projection)
        self.assertIn("CONSOLE_OUTPUT_OVERSCAN_LINES", projection)
        self.assertNotIn(".enumerate()\n        .flat_map", projection)

    def test_metadata_owns_logical_rows_and_maps_stable_slots_arithmetically(self) -> None:
        source = CONSOLE_METADATA.read_text(encoding="utf-8")

        self.assertIn("struct ConsoleOutputLogicalLine", source)
        self.assertIn("logical_lines:", source)
        self.assertIn("fn materialized_line_count", source)
        self.assertIn("fn logical_line_for_node_row", source)
        self.assertIn("fn visible_line_node_rows", source)

    def test_paint_and_hit_share_the_metadata_slot_mapping(self) -> None:
        paint = CONSOLE_PAINT.read_text(encoding="utf-8")
        hit = CONSOLE_HIT.read_text(encoding="utf-8")

        self.assertIn("logical_line_for_node_row", paint)
        self.assertIn("logical_line_for_node_row", hit)
        self.assertNotIn("Vec<usize>", paint)

    def test_console_scale_is_exported_to_the_existing_ui_profile_stream(self) -> None:
        paint = CONSOLE_PAINT.read_text(encoding="utf-8")
        ui_perf = UI_PERF.read_text(encoding="utf-8")

        for counter in (
            "ConsoleLogicalLineCount",
            "ConsoleMaterializedLineCount",
            "ConsoleMaterializedNodeCount",
            "ConsoleVisibleLineCount",
            "ConsoleOverscanLineCount",
        ):
            self.assertIn(counter, paint)
            self.assertIn(counter, ui_perf)

        projection = CONSOLE_PROJECTION.read_text(encoding="utf-8")
        for counter in (
            "ConsoleProjectionClonedNodeCount",
            "ConsoleProjectionFormattedIdCount",
            "ConsoleEnteredLineCount",
            "ConsoleExpiredLineCount",
            "ConsoleSlotReboundCount",
            "ConsoleProjectionGenerationReuseCount",
        ):
            self.assertIn(counter, projection)
            self.assertIn(counter, ui_perf)

    def test_snapshot_append_is_chunked_and_does_not_eagerly_flatten_text(self) -> None:
        snapshot = CONSOLE_SNAPSHOT.read_text(encoding="utf-8")
        generation = CONSOLE_SNAPSHOT_GENERATION.read_text(encoding="utf-8")
        history = CONSOLE_HISTORY.read_text(encoding="utf-8")

        self.assertIn("OnceLock<Arc<str>>", snapshot)
        self.assertIn("ConsoleOutputLineGeneration", snapshot)
        self.assertIn("ConsoleOutputLineDelta", snapshot)
        self.assertIn("append_bounded", generation)
        self.assertIn("Arc<[Arc<[ConsoleOutputLineSnapshot]>]>", generation)
        self.assertNotIn("fn rebuild_output", history)

    def test_activity_log_projection_reuses_the_shell_owned_sequence_generation(self) -> None:
        projection = ACTIVITY_LOG_PROJECTION.read_text(encoding="utf-8")

        self.assertIn("ActivityLogConsoleProjection", projection)
        self.assertIn("append_bounded", projection)
        self.assertIn("trim_before_source_id", projection)
        self.assertIn("record.sequence()", projection)
        self.assertNotIn("join(\"\\n\")", projection)

    def test_pane_payload_carries_the_snapshot_generation_without_flattening(self) -> None:
        payload = CONSOLE_PAYLOAD.read_text(encoding="utf-8")
        builder = CONSOLE_PAYLOAD_BUILDER.read_text(encoding="utf-8")

        self.assertIn("pub output: ConsoleOutputSnapshot", payload)
        self.assertIn("output: context.chrome.console_output.clone()", builder)
        self.assertNotIn("text_arc()", builder)

    def test_retained_host_owns_and_threads_the_console_projection_cache(self) -> None:
        host = RETAINED_HOST_APP.read_text(encoding="utf-8")
        apply = APPLY_PRESENTATION.read_text(encoding="utf-8")
        projection = CONSOLE_PROJECTION.read_text(encoding="utf-8")

        self.assertIn("console_pane_projection_cache", host)
        self.assertIn("console_projection_cache", apply)
        self.assertIn("reuse_console_projection", projection)
        self.assertLess(
            projection.index("reuse_console_projection"),
            projection.index("project_pane_body"),
        )

    def test_same_generation_projection_skips_metadata_and_slot_rebinding(self) -> None:
        projection = CONSOLE_PROJECTION.read_text(encoding="utf-8")
        reuse = projection.split("fn reuse_console_projection", 1)[1].split(
            "fn console_control_row_patches", 1
        )[0]

        self.assertIn("shares_logical_generation_with(output)", reuse)
        self.assertLess(
            reuse.index("shares_logical_generation_with(output)"),
            reuse.index("replacing_snapshot(output.clone())"),
        )
        for counter in (
            "ConsoleEnteredLineCount, 0.0",
            "ConsoleExpiredLineCount, 0.0",
            "ConsoleSlotReboundCount, 0.0",
        ):
            self.assertIn(counter, reuse)

    def test_pointer_layout_reads_logical_extent_without_flattening_output_text(self) -> None:
        source = CONSOLE_POINTER_LAYOUT.read_text(encoding="utf-8")

        self.assertIn("console_snapshot_content_extent(output)", source)
        self.assertNotIn("console_output.as_ref()", source)
        self.assertNotIn("chrome.console_output.as_ref()", source)


if __name__ == "__main__":
    unittest.main()
