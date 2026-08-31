from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


def source(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def between(document: str, start: str, end: str) -> str:
    return document.split(start, 1)[1].split(end, 1)[0]


class Text01CompositeActivationTests(unittest.TestCase):
    def test_registration_and_project_composite_activation_have_separate_owners(self) -> None:
        database = source("zircon_runtime/src/text/font/database.rs")
        asset_lifecycle = source(
            "zircon_runtime/src/text/font/database/asset_lifecycle.rs"
        )
        fallback_queries = source(
            "zircon_runtime/src/text/font/database/fallback_queries.rs"
        )

        self.assertIn("mod asset_lifecycle;", database)
        self.assertIn("pub(crate) fn replace_font_asset(", asset_lifecycle)
        self.assertIn("fn replace_asset_registrations(", asset_lifecycle)
        self.assertNotIn("active_composite_font", asset_lifecycle)
        self.assertNotIn("composite.default_family", asset_lifecycle)
        self.assertNotIn("composite.sub_fonts", asset_lifecycle)
        self.assertIn("pub(crate) fn set_project_composite_font(", database)
        self.assertIn("mod fallback_queries;", database)
        self.assertIn(
            "pub(crate) fn fallback_candidates_for_codepoint(", fallback_queries
        )

    def test_composite_candidate_enumeration_is_a_folder_backed_leaf(self) -> None:
        font_root = source("zircon_runtime/src/text/font/mod.rs")
        fallback = source("zircon_runtime/src/text/font/fallback.rs")
        composite_resolve = source(
            "zircon_runtime/src/text/font/composite_resolve.rs"
        )

        self.assertIn("mod composite_resolve;", font_root)
        self.assertNotIn("fn candidate_families(", fallback)
        self.assertIn("pub(super) fn candidate_faces_for_cluster(", composite_resolve)

    def test_default_font_record_is_the_only_project_activation_call_site(self) -> None:
        text = source("zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs")
        font_assets = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs"
        )
        runtime_asset = source("zircon_runtime/src/text/font/runtime_asset.rs")
        self.assertNotIn("set_project_composite_font(", text)
        self.assertNotIn("set_project_composite_font(", font_assets)
        self.assertEqual(runtime_asset.count("set_project_composite_font("), 2)
        self.assertIn("let project_composite = source", runtime_asset)
        self.assertIn("fn retire_runtime_font_assets_from_database", runtime_asset)
        self.assertIn("asset_ref == DEFAULT_UI_FONT_ASSET", runtime_asset)
        self.assertIn(
            "report.database_changed || report.asset_mapping_changed", runtime_asset
        )
        self.assertNotIn("publish_font_database", font_assets)

    def test_renderer_invalidation_consumes_semantic_font_asset_changes(self) -> None:
        text = source("zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs")
        font_assets = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs"
        )
        render_state = source("zircon_runtime/src/text/render_state.rs")

        self.assertIn("let font_faces_changed = shaping_changed;", text)
        self.assertIn("refresh_font_asset_records(", text)
        self.assertIn("replace_shared_claims_with_admissions", font_assets)
        self.assertIn(
            "font_collection_changed: text_state.refresh_font_collection()",
            font_assets,
        )
        self.assertNotIn("font_faces_changed |= ensured.faces_changed;", text)
        self.assertIn("if font_faces_changed {\n                self.invalidate_font_faces();", text)
        self.assertNotIn("resolved_texts.font_faces_changed()", text)
        self.assertNotIn("text_state.publish_font_database()", text)
        self.assertNotIn("face_count_at_entry", text)
        self.assertNotIn("native_font_id_report.font_faces_changed", text)
        self.assertNotIn("fn register_font_source(", render_state)
        self.assertNotIn("fn publish_font_database(", render_state)
        self.assertIn("#[cfg(test)]\n    pub(crate) fn face_count(", render_state)

        invalidation = between(
            text,
            "fn invalidate_font_faces(&mut self) {",
            "\n    pub(super) fn render",
        )
        for semantic_consumer in (
            "self.text_state.invalidate_font_faces();",
            ".discard_all_for_face_invalidation();",
            "self.sdf_atlas.invalidate_font_faces();",
            "self.sdf_cpu_frame.invalidate();",
            "self.segment_cache.invalidate_frame_product();",
        ):
            self.assertIn(semantic_consumer, invalidation)

    def test_native_prepare_report_does_not_duplicate_font_change_state(self) -> None:
        text = source("zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs")
        fixtures = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests/prepare_report.rs"
        )
        resolved_batches = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs"
        )

        native_report = between(
            text,
            "struct ScreenSpaceUiNativePrepareReport {",
            "}\n\nimpl ScreenSpaceUiTextSystem",
        )
        self.assertNotIn("font_faces_changed", native_report)
        self.assertNotIn("font_faces_changed:", fixtures)
        self.assertIn("font_faces_changed: bool", resolved_batches)

    def test_raster_worker_completion_is_face_owned_not_atlas_page_owned(self) -> None:
        native_atlas = source("zircon_runtime/src/text/native_bitmap_atlas.rs")
        native_frame = source(
            "zircon_runtime/src/text/native_bitmap_atlas/frame.rs"
        )
        source_cache = source(
            "zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs"
        )
        raster_pool = source("zircon_runtime/src/text/parallel/raster_pool.rs")
        combined = native_atlas + native_frame + source_cache + raster_pool

        self.assertIn("let face_epoch = source_cache.face_epoch();", native_frame)
        self.assertIn("drain_completed_for_face_epoch(", native_frame)
        self.assertIn("face_epoch,", native_frame)
        self.assertIn("apply_worker_completion_drain(completion_drain)", native_frame)
        self.assertIn("face_invalidated_count", source_cache)
        self.assertIn("disposition_for_face_epoch", raster_pool)
        for obsolete in (
            "TextRasterWorkTarget",
            "drain_completed_for_target",
            "stale_page_generation_ids",
            "stale_page_generation_count",
        ):
            self.assertNotIn(obsolete, combined)

    def test_font_publication_clone_boundaries_are_explicit(self) -> None:
        shared = source("zircon_runtime/src/text/font/shared.rs")
        runtime_asset = source("zircon_runtime/src/text/font/runtime_asset.rs")
        asset_lifecycle = source(
            "zircon_runtime/src/text/font/database/asset_lifecycle.rs"
        )

        self.assertIn("pub(crate) fn mutate_published_snapshot", shared)
        self.assertIn("shared_mutation_outer_database_clone", shared)
        self.assertIn("shared_owned_mutation_result_clone", shared)
        self.assertIn("owner_registration_staging_clone", asset_lifecycle)
        self.assertEqual(runtime_asset.count("mutate_published_snapshot"), 1)
        self.assertNotIn("fn admit_runtime_font_asset(", runtime_asset)
        self.assertNotIn("fn retire_runtime_font_asset(", runtime_asset)
        self.assertNotIn(
            "let (_, _, (font_inputs_changed, admission_outcomes)) = self.mutate(",
            runtime_asset,
        )
        self.assertNotIn(
            "font_collection\n        .mutate(|database| apply_prepared_runtime_font_asset_admission",
            runtime_asset,
        )

    def test_renderer_admission_has_one_production_batch_path(self) -> None:
        font_assets = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs"
        )
        resolved_batches = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs"
        )

        self.assertNotIn("fn load_font_asset_record", font_assets)
        self.assertIn("#[cfg(test)]\npub(super) fn ensure_font_asset_record", font_assets)
        self.assertIn("refresh_font_asset_records(", font_assets)
        self.assertNotIn("fn resolve_text_batches(", resolved_batches)
        self.assertIn(
            "refresh_font_asset_records(",
            source("zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs"),
        )

    def test_sdf_raster_font_lookup_does_not_mutate_the_renderer_database(self) -> None:
        font_bake = source("zircon_runtime/src/text/sdf/font_bake.rs")
        face_cache = source(
            "zircon_runtime/src/text/sdf/font_bake/font_asset_cache.rs"
        )
        sdf_profile = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report/profile/sdf_residency.rs"
        )

        self.assertIn("#[cfg(test)]\npub(super) fn resolve_font_face", font_bake)
        self.assertIn("resolve_registered_font_face", face_cache)
        self.assertNotIn("load_text_font_source", face_cache)
        self.assertNotIn("replace_font_asset", face_cache)
        self.assertNotIn("replace_font_source", face_cache)
        self.assertNotIn("remove_font_asset", face_cache)
        self.assertNotIn("resident_source_not_found_count", face_cache)
        self.assertNotIn("resident_registration_failure_count", face_cache)
        self.assertNotIn("resident_font_asset_source_not_found", sdf_profile)
        self.assertNotIn("resident_font_asset_registration_failures", sdf_profile)
        self.assertIn("resident_font_asset_no_registered_faces", sdf_profile)

    def test_cosmic_locale_cache_starts_from_the_callers_font_snapshot(self) -> None:
        cache = source(
            "zircon_runtime/src/text/shaping/cosmic/font_system_cache.rs"
        )
        cache_tests = source(
            "zircon_runtime/src/text/shaping/cosmic/font_system_cache/tests.rs"
        )
        self.assertIn("RefCell<Option<LocaleFontSystemCache>>", cache)
        self.assertIn("LocaleFontSystemCache::new(font_collection)", cache)
        self.assertNotIn("shared_font_collection_snapshot", cache)
        self.assertIn("LocaleFontSystemCache::new(&retired)", cache_tests)

    def test_renderer_owns_explicit_system_font_discovery_before_state_snapshot(self) -> None:
        renderer = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs"
        )
        render_state = source("zircon_runtime/src/text/render_state.rs")
        self.assertIn("font_collection", renderer)
        self.assertIn(
            "apply_system_font_policy(SystemFontPolicy::Discover)", renderer
        )
        self.assertIn(".mutate_published_snapshot(|database|", renderer)
        self.assertNotIn("discover_system_fonts", render_state)
        self.assertNotIn("apply_system_font_policy", render_state)

    def test_parallel_shape_production_requires_an_explicit_font_collection(self) -> None:
        shape_pool = source("zircon_runtime/src/text/parallel/shape_pool.rs")
        self.assertIn(
            "#[cfg(test)]\npub(crate) fn shape_paragraphs_with_cache(", shape_pool
        )
        self.assertIn(
            "pub(crate) fn shape_paragraphs_with_cache_in_font_collection(", shape_pool
        )
        self.assertNotIn("fn finish_pending_shape_job(", shape_pool)

    def test_core_render_framework_injects_the_core_owned_font_collection(self) -> None:
        create = source(
            "zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/create/create_render_framework.rs"
        )
        descriptor = source(
            "zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs"
        )
        framework = source(
            "zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs"
        )
        renderer = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/construct.rs"
        )

        self.assertIn("font_collection_service_for_core(core)", create)
        self.assertIn("font_collection,", create)
        self.assertNotIn("shared_font_collection_service()", create)
        self.assertIn(
            'dependency_on(TEXT_MODULE_NAME, ServiceKind::Manager, "FontServices")',
            descriptor,
        )
        self.assertIn(
            "new_with_plugin_render_extensions_and_solari_and_compute_task_pool(",
            create,
        )
        self.assertIn("font_collection,", framework)
        self.assertIn(
            "new_with_plugin_render_extensions_and_shading_models_and_font_collection(",
            renderer,
        )

    def test_explicit_font_collection_is_the_scene_renderer_child_boundary(self) -> None:
        renderer = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/construct.rs"
        )
        explicit_boundary = between(
            renderer,
            "pub(crate) fn new_with_plugin_render_extensions_and_shading_models_and_font_collection(",
            "\n    }\n\n    #[cfg(test)]",
        )

        self.assertIn("font_collection: Arc<FontCollectionService>", explicit_boundary)
        self.assertIn(
            "new_with_icon_source_and_plugin_render_features_and_shading_models_and_font_collection(",
            explicit_boundary,
        )
        self.assertIn("font_collection,", explicit_boundary)

    def test_dynamic_runtime_ui_builds_surfaces_from_the_core_owned_collection(self) -> None:
        construction = source("zircon_runtime/src/dynamic_api/session/construction.rs")
        error = source("zircon_runtime/src/dynamic_api/session/error.rs")
        project = source("zircon_runtime/src/dynamic_api/session/project.rs")
        runtime_ui = source("zircon_runtime/src/dynamic_api/session/runtime_ui.rs")

        self.assertIn("font_collection_service_for_core(&core)", construction)
        self.assertIn("load_runtime_ui_surfaces(&core, font_collection.clone())", construction)
        self.assertIn("font_collection: Arc<FontCollectionService>", project)
        self.assertNotIn("font_collection_service_for_core(core)", project)
        self.assertNotIn("ResolveFontServices", error)
        self.assertIn("RuntimeUiSurfaceSet::load(", project)
        self.assertIn(
            "build_surface_with_prototype_store_and_font_collection(", runtime_ui
        )
        self.assertIn("font_collection: Arc<FontCollectionService>", runtime_ui)
        self.assertNotIn("UiSurface::new(", runtime_ui.split("#[cfg(test)]", 1)[0])

    def test_dynamic_fallback_ui_cache_uses_the_core_owned_collection(self) -> None:
        construction = source("zircon_runtime/src/dynamic_api/session/construction.rs")
        cache = source("zircon_runtime/src/dynamic_api/session/ui_extract_cache.rs")

        self.assertIn(
            "ui_extract_cache: RuntimeUiExtractCache::new_with_font_collection(font_collection)",
            construction,
        )
        self.assertIn("new_with_font_collection(", cache)
        self.assertIn("self.text_measure_cache.font_database_generation()", cache)
        self.assertNotIn("current_resolved_text_font_generation", cache)
        self.assertNotIn("impl Default for RuntimeUiExtractCache", cache)

    def test_process_and_core_font_collection_entrypoints_document_their_owners(self) -> None:
        editor_paint_cache = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/cache.rs"
        )
        editor_window_host = source("zircon_editor/src/ui/host/window_host_manager.rs")
        layout_session = source("zircon_runtime/src/text/layout_session.rs")
        surface = source("zircon_runtime/src/ui/surface/surface.rs")
        measure_cache = source("zircon_runtime/src/ui/text/measure_cache.rs")
        layout_tree = source("zircon_runtime/src/ui/layout/pass/layout_tree.rs")
        one_shot_extract = source(
            "zircon_runtime/src/ui/surface/render/extract/one_shot.rs"
        )

        self.assertIn("Process-owner entrypoint", layout_session)
        self.assertIn("Editor-host or standalone surface", surface)
        self.assertIn("standalone cache backed by the process-owner", measure_cache)
        self.assertIn("standalone layout pass", layout_tree)
        self.assertIn("standalone arranged tree", one_shot_extract)
        self.assertIn("UiSurface::new(", editor_window_host)
        self.assertIn("current_resolved_text_font_generation", editor_paint_cache)
        self.assertIn("new_with_font_collection", surface)
        self.assertIn("new_with_font_collection", measure_cache)

    def test_layout_range_invariants_fail_closed_instead_of_dropping_text(self) -> None:
        line_break = source("zircon_runtime/src/text/layout/line_break/mod.rs")
        cosmic_lines = source("zircon_runtime/src/text/shaping/cosmic/hard_lines.rs")
        cosmic = source("zircon_runtime/src/text/shaping/cosmic.rs")
        wrapping = source("zircon_runtime/src/ui/text/layout_engine/wrapping.rs")

        self.assertIn("TextShapingOutcome::failed(TextLayoutError::LayoutFailed)", line_break)
        self.assertIn("TextShapingOutcome::failed(TextLayoutError::BidiInvariant)", line_break)
        self.assertIn("if !glyph.cluster_flags.cluster_start", line_break)
        self.assertIn(
            "if !glyph.cluster_flags.soft_break && !glyph.cluster_flags.mandatory_break",
            line_break,
        )
        self.assertIn("break_boundaries.windows(2).any", line_break)
        self.assertIn("break_boundaries.sort_unstable_by_key", line_break)
        self.assertIn(
            "line_break_chunks_normalize_visual_glyph_order_before_materializing_boundaries",
            source("zircon_runtime/src/text/layout/line_break/tests.rs"),
        )
        self.assertIn(
            "line_break_chunks_fail_closed_on_non_utf8_cluster_ranges",
            source("zircon_runtime/src/text/layout/line_break/tests.rs"),
        )
        self.assertIn("TextLayoutError::LayoutFailed", wrapping)
        cosmic_normalizer = between(
            cosmic_lines,
            "pub(super) fn normalize_cosmic_hard_lines(",
            "\n}\n",
        )
        self.assertIn("return Err(ItemizationError::InvalidSourceRange", cosmic_normalizer)
        self.assertIn("request.text.is_char_boundary(local_start)", cosmic_normalizer)
        self.assertIn("request.text.is_char_boundary(local_end)", cosmic_normalizer)
        self.assertIn("is_canonical_separator", cosmic_normalizer)
        self.assertIn("virtual_hard_break_glyph", cosmic_normalizer)
        self.assertNotIn("else {\n                continue;", cosmic_normalizer)
        self.assertIn("InvalidResolvedRange", cosmic)
        self.assertIn("source_start.checked_add", cosmic)
        self.assertNotIn(".unwrap_or_default()", cosmic)
        self.assertNotIn("text.get(source_range.clone()) else {\n            continue;", line_break)
        self.assertNotIn("text.get(metric.source_start..metric.source_end) else {\n            continue;", wrapping)

    def test_surface_input_source_metrics_use_the_surface_font_collection(self) -> None:
        shaping = source("zircon_runtime/src/text/shaping/mod.rs")
        source_metrics = source(
            "zircon_runtime/src/ui/text/geometry/source_metrics.rs"
        )
        geometry = source("zircon_runtime/src/ui/text/geometry.rs")
        hit_test = source("zircon_runtime/src/ui/text/hit_test.rs")
        ime = source(
            "zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs"
        )
        pointer = source(
            "zircon_runtime/src/ui/surface/input/text_pointer.rs"
        )

        self.assertIn("FontCollectionTextShapeRunProvider", shaping)
        self.assertIn("FontCollectionTextShapeRunProvider::new", source_metrics)
        self.assertNotIn("DirectTextShapeRunProvider", source_metrics)
        self.assertIn(
            "caret_frame_for_text_layout_with_font_collection", geometry
        )
        self.assertIn(
            "text_range_frames_for_text_layout_with_font_collection", geometry
        )
        self.assertIn(
            "hit_test_text_layout_with_font_collection", hit_test
        )
        self.assertIn("font_collection_snapshot()", ime)
        self.assertIn("font_collection_snapshot()", pointer)
        for surface_input in (ime, pointer):
            self.assertIn("surface.observed_text_font_generation", surface_input)
            self.assertIn("font_collection.generation()", surface_input)

    def test_direct_compatibility_provider_is_snapshot_bound(self) -> None:
        shaping = source("zircon_runtime/src/text/shaping/mod.rs")
        self.assertIn("font_collection: FontCollectionSnapshot", shaping)
        self.assertIn("font_collection: shared_font_collection_snapshot()", shaping)
        self.assertIn("shape_text_with_diagnostics_in_font_collection", shaping)
        self.assertNotIn(
            "DirectTextShapeRunProvider;",
            shaping,
            "the compatibility provider must retain an explicit snapshot field",
        )
        self.assertNotIn(
            "shape_text(request)).map(Arc::new)",
            shaping,
            "direct compatibility shaping must not reacquire the process collection per request",
        )


if __name__ == "__main__":
    unittest.main()
