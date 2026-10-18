import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


class RuntimeTextInfrastructureCompileContractTests(unittest.TestCase):
    def test_rich_cache_telemetry_is_owner_qualified_and_atomically_reset(self):
        cache = (ROOT / "zircon_runtime/src/text/cache/rich_cache.rs").read_text(
            encoding="utf-8"
        )
        parser = (
            ROOT / "zircon_runtime/src/text/rich/parser_registry.rs"
        ).read_text(encoding="utf-8")
        session = (ROOT / "zircon_runtime/src/text/layout_session.rs").read_text(
            encoding="utf-8"
        )
        measure = (ROOT / "zircon_runtime/src/ui/text/measure_cache.rs").read_text(
            encoding="utf-8"
        )
        profile = (
            ROOT / "zircon_runtime/src/ui/surface/render/text_prewarm/profile.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub parser_identity: u64", cache)
        self.assertIn("pub decorator_generation: u64", cache)
        self.assertIn("pub emoji_generation: u64", cache)
        self.assertIn("pub telemetry_saturated: bool", cache)
        self.assertIn("pub(crate) fn take_report", cache)
        self.assertIn("fn reset_interval_counters", cache)
        self.assertNotIn("CompiledRichTextCacheFrameSampler", cache)
        self.assertIn("fn take_compiled_cache_report", parser)
        self.assertIn("fn take_compiled_rich_text_cache_report", session)
        self.assertIn("take_compiled_rich_text_cache_report()", measure)
        self.assertNotIn("compiled_rich_text_cache_sampler", measure)
        self.assertIn('"ui_text.rich_cache.parser_identity"', profile)
        self.assertIn('"ui_text.rich_cache.decorator_generation"', profile)
        self.assertIn('"ui_text.rich_cache.emoji_generation"', profile)
        self.assertIn('"ui_text.rich_cache.counter_saturated"', profile)

    def test_rich_cache_single_flight_contention_is_measurable_before_redesign(self):
        cache = (ROOT / "zircon_runtime/src/text/cache/rich_cache.rs").read_text(
            encoding="utf-8"
        )
        profile = (
            ROOT / "zircon_runtime/src/ui/surface/render/text_prewarm/profile.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub compile_requests_in_flight: usize", cache)
        self.assertIn("pub single_flight_wait_count: u64", cache)
        self.assertIn("pub single_flight_wait_nanos: u64", cache)
        self.assertIn("pub single_flight_wait_max_nanos: u64", cache)
        self.assertIn("Cell::new(false)", cache)
        self.assertIn("initialized_here.set(true)", cache)
        self.assertIn('"ui_text.rich_cache.compile_requests_in_flight"', profile)
        self.assertIn('"ui_text.rich_cache.single_flight_waits"', profile)
        self.assertIn('"ui_text.rich_cache.single_flight_wait_nanos"', profile)
        self.assertIn('"ui_text.rich_cache.single_flight_wait_max_nanos"', profile)

    def test_compiled_rich_dependencies_are_typed_before_render_resource_collection(self):
        dependency = (
            ROOT / "zircon_runtime/src/text/rich/compiled/dependency.rs"
        ).read_text(encoding="utf-8")
        compiled = (ROOT / "zircon_runtime/src/text/rich/compiled.rs").read_text(
            encoding="utf-8"
        )
        memory = (
            ROOT / "zircon_runtime/src/text/rich/compiled/memory.rs"
        ).read_text(encoding="utf-8")
        text_facade = (ROOT / "zircon_runtime/src/text/mod.rs").read_text(
            encoding="utf-8"
        )
        texture_collection = (
            ROOT / "zircon_runtime/src/graphics/scene/resources/ui_texture.rs"
        ).read_text(encoding="utf-8")
        plan = (
            ROOT
            / "docs/plans/zircon_runtime/text/07/2026-08-30-rich-typed-dependency-closure-foundation.md"
        ).read_text(encoding="utf-8")

        self.assertIn("pub enum RichTextDependency", dependency)
        self.assertIn("ImageTexture(ResourceId)", dependency)
        self.assertIn("pub(super) fn collect", dependency)
        self.assertIn("dependencies: Arc<[RichTextDependency]>", compiled)
        self.assertIn("dependency::collect(&parsed)", compiled)
        self.assertIn("pub fn dependencies(&self) -> &[RichTextDependency]", compiled)
        self.assertNotIn("resource_ids", compiled)
        self.assertIn("compiled.dependencies.len()", memory)
        self.assertIn("RichTextDependency", text_facade)
        self.assertIn("RichTextDependency::ImageTexture", texture_collection)
        self.assertNotIn("rich.resource_ids()", texture_collection)
        self.assertIn(
            "RRT-P1-020_typed_image_dependency_foundation_static_complete", plan
        )
        self.assertIn("O(R + D log D)", plan)

    def test_rich_paint_projection_profile_is_runtime_owned_and_cache_truthful(self):
        projection = (
            ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/paint_projection.rs"
        ).read_text(encoding="utf-8")
        render = (
            ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
        ).read_text(encoding="utf-8")
        plan_cache = (
            ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("struct ScreenSpaceUiTextPaintProjectionReport", projection)
        self.assertIn("project_transient_paint_elements", projection)
        self.assertIn('"materialize_transient_text_paint"', projection)
        self.assertIn('"ui_text.paint_projection.rich_command_count"', projection)
        self.assertIn('"ui_text.paint_projection.rich_paint_run_count"', projection)
        self.assertIn('"ui_text.paint_projection.rich_run_text_bytes"', projection)
        self.assertIn("mod paint_projection;", render)
        self.assertIn("project_transient_paint_elements(", render)
        self.assertIn("publish_profile_counters", render)
        self.assertIn("paint_projection_report", plan_cache)
        self.assertIn(
            "ScreenSpaceUiTextPaintProjectionReport::default().publish_profile_counters()",
            plan_cache,
        )

    def test_table_layout_work_receipt_is_frame_scoped_and_phase_complete(self):
        work_report = (
            ROOT / "zircon_runtime/src/text/layout_session/table_work.rs"
        ).read_text(encoding="utf-8")
        session = (
            ROOT / "zircon_runtime/src/text/layout_session.rs"
        ).read_text(encoding="utf-8")
        table_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("struct TextTableLayoutWorkReport", work_report)
        self.assertIn("preferred_cell_layout_count", work_report)
        self.assertIn("final_cell_layout_count", work_report)
        self.assertIn("publish_profile_counters", work_report)
        self.assertIn('"rich_table_preferred_cell_layout_count"', work_report)
        self.assertIn('"rich_table_final_cell_layout_count"', work_report)
        self.assertIn("table_layout_work_report: TextTableLayoutWorkReport", session)
        self.assertIn("record_table_layout_attempt", session)
        self.assertIn("record_table_layout_tracks", session)
        self.assertIn("record_table_preferred_cell_layout", table_layout)
        self.assertIn("record_table_final_cell_layout", table_layout)
        self.assertIn("record_table_layout_output", table_layout)

    def test_layout_geometry_budget_is_session_owned_and_reports_rejection_context(self):
        geometry = (
            ROOT / "zircon_runtime/src/text/layout_geometry.rs"
        ).read_text(encoding="utf-8")
        session = (
            ROOT / "zircon_runtime/src/text/layout_session.rs"
        ).read_text(encoding="utf-8")
        diagnostics = (
            ROOT / "zircon_runtime/src/text/layout_session/diagnostics.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub(crate) struct TextLayoutGeometryBudget", geometry)
        self.assertIn("DEFAULT_MAX_EXACT_LAYOUT_EXTENT", geometry)
        self.assertIn("fn checked_add_accumulated", geometry)
        self.assertIn("fn checked_scale_accumulated", geometry)
        self.assertIn("geometry_budget: TextLayoutGeometryBudget", session)
        self.assertIn("new_with_font_collection_and_geometry_budget", session)
        self.assertIn("pub(crate) fn reject_geometry", session)
        self.assertIn("TextLayoutGeometryRejectionReceipt", diagnostics)
        self.assertIn("source_range: Option<(u32, u32)>", diagnostics)
        self.assertIn("work_units: usize", diagnostics)

    def test_rich_table_uses_typed_unbounded_measurement_without_fake_extent(self):
        geometry = (
            ROOT / "zircon_runtime/src/text/layout_geometry.rs"
        ).read_text(encoding="utf-8")
        measurement = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/measurement.rs"
        ).read_text(encoding="utf-8")
        table_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("enum TextLayoutAxisConstraint", geometry)
        self.assertIn("Self::Unbounded => f32::INFINITY", geometry)
        self.assertIn("intrinsic_measurement_frame_with_provider", measurement)
        self.assertIn("bounded_inline_measurement_frame_with_provider", measurement)
        self.assertIn("intrinsic_measurement_frame_with_provider", table_layout)
        self.assertIn("bounded_inline_measurement_frame_with_provider", table_layout)
        self.assertNotIn("MAX_PROVISIONAL_CELL_BLOCK_EXTENT", table_layout)
        self.assertNotIn("provisional_block_extent", table_layout)

    def test_rich_table_track_sizing_rejects_invalid_geometry_without_sanitizing(self):
        sizing = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs"
        ).read_text(encoding="utf-8")
        cell_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/cell_layout.rs"
        ).read_text(encoding="utf-8")
        table_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("TextLayoutAxisConstraint", sizing)
        self.assertIn("Result<Vec<f32>, TextLayoutGeometryViolation>", sizing)
        self.assertIn("checked_add_accumulated", sizing)
        self.assertNotIn("sanitize_non_negative", sizing)
        self.assertIn("Result<Self, TextLayoutGeometryViolation>", cell_layout)
        self.assertIn("checked_add_accumulated", cell_layout)
        self.assertIn("TextLayoutAxisConstraint::from_request_extent", table_layout)
        self.assertIn("subtract_accumulated", table_layout)

    def test_rich_table_validates_layout_and_aggregate_geometry_before_publication(self):
        geometry_admission = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/geometry_admission.rs"
        ).read_text(encoding="utf-8")
        measurement = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/measurement.rs"
        ).read_text(encoding="utf-8")
        table_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs"
        ).read_text(encoding="utf-8")
        table_geometry = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/geometry.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("validate_resolved_layout_geometry", geometry_admission)
        self.assertIn("validate_resolved_size_geometry", geometry_admission)
        self.assertIn("admit_coordinate", geometry_admission)
        self.assertIn("glyph_advances", geometry_admission)
        self.assertIn("validate_resolved_layout_geometry", measurement)
        self.assertIn("validate_resolved_size_geometry", measurement)
        self.assertIn("checked_scale_accumulated", measurement)
        self.assertIn("TextLayoutGeometryOwner::IntrinsicMeasurement", measurement)
        self.assertIn("validate_resolved_layout_geometry", table_layout)
        self.assertIn("checked_add_accumulated", table_geometry)
        self.assertIn("TextLayoutGeometryOwner::TableAggregate", table_geometry)
