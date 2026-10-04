"""Model residual Runtime UI image/text dependency-product prepare work."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path
from typing import Any


CRITICAL_SOURCE_FILES = (
    "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs",
    "zircon_runtime/src/graphics/scene/resources/ui_texture.rs",
    "zircon_runtime/src/graphics/scene/resources/ui_texture/prepare_receipt.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/record.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache/frame_journal.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache/native_dependency_index.rs",
    "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_submission_failure.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_batches.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/segment_product.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_cpu_frame.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/segment_product.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertex_buffer.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/material.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/compiled_frame.rs",
    "zircon_runtime/src/text/sdf/font_bake/atlas_build.rs",
    "zircon_runtime/src/text/sdf/font_bake/prepared_atlas.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report/profile.rs",
)

REFERENCE_SOURCE_FILES = (
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/SlateResourceHandle.h",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ShaderResourceManager.cpp",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIResourceManager.cpp",
    "dev/Fyrox/fyrox-impl/src/renderer/cache/texture.rs",
    "dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp",
    "dev/slint/internal/core/item_rendering.rs",
    "dev/slint/internal/core/textlayout/sharedparley.rs",
)

SOURCE_ANCHORS = {
    CRITICAL_SOURCE_FILES[0]: (
        "ui_texture_dependencies.prepare(ui)",
        "dependencies.as_ref()",
        "prepare_ui_textures_for_frame",
    ),
    CRITICAL_SOURCE_FILES[1]: (
        "struct UiTextureDependencyCache",
        "segment.extract().command_segments()",
        "collect_ui_texture_command_segment_dependencies",
        "UiTextureDependencyChangeJournal",
    ),
    CRITICAL_SOURCE_FILES[2]: (
        "binding_product_generation",
        "journal_applies",
        "frame_retry_ids",
        "dependency_prepare_visit_count",
        "for &requested in requested_ids",
    ),
    CRITICAL_SOURCE_FILES[3]: (
        "render_segments.iter().zip(image_segments.iter_mut())",
        "refresh_segment_dependencies",
        "prepared.change_journal()",
        "binding_product: Option<Arc<ScreenSpaceUiImageBindingProduct>>",
    ),
    CRITICAL_SOURCE_FILES[4]: (
        "struct PreparedScreenSpaceUi",
        "change_journal",
        "generation",
    ),
    CRITICAL_SOURCE_FILES[5]: (
        "return self.cached_plan.as_ref().map(Arc::clone)",
        "all_segments_reused",
        "ScreenSpaceUiFrameChangeJournal",
    ),
    CRITICAL_SOURCE_FILES[6]: (
        "prepared.change_journal()",
        "self.image_system.prepare",
        "self.text_system.prepare",
    ),
    CRITICAL_SOURCE_FILES[7]: (
        "refresh_font_dependencies(prepared)",
        "prepare_frame_product",
    ),
    CRITICAL_SOURCE_FILES[8]: (
        "self.font_dependency_generation == Some(prepared.generation())",
        "self.frame_generation == Some(generation)",
        "ScreenSpaceUiTextFrameAggregateIndex",
        "native_glyph_dependency_ref_counts",
        "journal.changed_segment_indices()",
        "text_products_changed",
        "reuse_frame_product_for_source_generation",
    ),
    CRITICAL_SOURCE_FILES[9]: (
        "ScreenSpaceUiTextFrameChangeJournal",
        "changed_segment_indices",
        "full_rebuild_reason",
    ),
    CRITICAL_SOURCE_FILES[10]: (
        "pub(super) struct NativeBitmapAtlasSegmentDependencyIndex",
        "from_glyph_runs",
    ),
    CRITICAL_SOURCE_FILES[11]: (
        "rollback_failed_frame_submissions",
        "self.last_ui_texture_prepare_receipt = None",
    ),
    CRITICAL_SOURCE_FILES[12]: (
        "preparation_inputs_match",
        "ScreenSpaceUiGlyphArtifactLine::cache_identity",
    ),
    CRITICAL_SOURCE_FILES[13]: (
        "self.retained_frame_generation == retained_generation",
        "self.prepared_texts.matches_iter",
    ),
    CRITICAL_SOURCE_FILES[14]: (
        "struct SdfAtlasSegmentProductIndex",
        "journal.changed_segment_indices()",
        "newly_active_keys",
    ),
    CRITICAL_SOURCE_FILES[15]: (
        "self.retained_frame_generation == retained_generation",
        "self.matches_iter",
    ),
    CRITICAL_SOURCE_FILES[16]: (
        "self.compiled_segments.prepare(",
        "write_sdf_vertex_buffer_ranges(",
        "self.material.prepare_ranges(",
    ),
    CRITICAL_SOURCE_FILES[17]: (
        "struct SdfCompiledTextSegmentIndex",
        "fn patch_retained_frame",
        "journal.changed_segment_indices()",
        "fn apply_replacements",
    ),
    CRITICAL_SOURCE_FILES[18]: (
        "fn write_sdf_vertex_buffer_ranges",
        "coalesced_vertex_ranges",
        "range.start.saturating_mul(vertex_size)",
    ),
    CRITICAL_SOURCE_FILES[19]: (
        "fn prepare_ranges",
        "coalesced_material_ranges",
        "material_range_upload_bytes",
    ),
    CRITICAL_SOURCE_FILES[20]: (
        "self.retained_frame_generation == retained_generation",
        "text_batches_match_iter",
    ),
    CRITICAL_SOURCE_FILES[21]: (
        "self.prepared_atlas.reuse(",
        "self.prepare_missing_glyphs",
    ),
    CRITICAL_SOURCE_FILES[22]: (
        "self.slots.as_slice() != slots",
        "Some(reused)",
    ),
    CRITICAL_SOURCE_FILES[23]: (
        "ui_text.sdf_prepare.compiled_segment_visits",
        "ui_text.sdf_prepare.vertex_buffer_write_bytes",
        "ui_text.sdf_prepare.material_buffer_write_bytes",
    ),
    REFERENCE_SOURCE_FILES[0]: ("FSlateResourceHandle", "TSharedPtr<FSlateSharedHandleData>"),
    REFERENCE_SOURCE_FILES[1]: ("ExistingHandle", "Proxy->HandleData"),
    REFERENCE_SOURCE_FILES[2]: ("DynamicResourceMap", "GetVectorResource"),
    REFERENCE_SOURCE_FILES[3]: ("modifications_counter", "sampler_modifications_counter"),
    REFERENCE_SOURCE_FILES[4]: (
        "FCachedShapedTextKey",
        "CachedShapedText",
    ),
    REFERENCE_SOURCE_FILES[5]: (
        "GetShapedGlyphFontAtlasData",
        "ShapedGlyphToAtlasData.Find(GlyphKey)",
        "GetSdfGlyphFontAtlasData",
        "SdfGlyphToAtlasData.Find(GlyphKey)",
        "SdfGlyphToAtlasData.Add(GlyphKey",
    ),
    REFERENCE_SOURCE_FILES[6]: (
        "pub struct ItemCache<T>",
        "evaluate_if_dirty(update_fn)",
        "clear_cache_if_scale_factor_changed",
    ),
    REFERENCE_SOURCE_FILES[7]: (
        "type InnerTextLayoutCache = crate::item_rendering::ItemCache<Vec<TextParagraph>>",
        "pub struct TextLayoutCache",
    ),
}


def validate_output_path(path: Path) -> Path:
    if path.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("performance artifacts must be written to D:, E:, or F:")
    return path


def run(
    frame_count: int = 4_096,
    segment_count: int = 64,
    image_dependencies_per_segment: int = 4,
    binding_cache_entry_count: int = 512,
    text_dependencies_per_segment: int = 32,
    text_run_spans_per_segment: int = 8,
    font_dependencies_per_segment: int = 2,
    ui_commands_per_frame: int = 1_024,
    compiled_vertices_per_text_dependency: int = 6,
    sdf_vertex_byte_len: int = 60,
    material_uniform_stride: int = 256,
    delta_frame_count: int = 32,
    text_affecting_delta_frame_count: int = 4,
    text_paint_only_delta_frame_count: int | None = None,
    text_resident_glyph_delta_frame_count: int | None = None,
    text_new_glyph_delta_frame_count: int | None = None,
    changed_segments_per_delta_frame: int = 1,
    resource_generation_frame_count: int = 4,
) -> dict[str, Any]:
    positive_inputs = {
        "frame_count": frame_count,
        "segment_count": segment_count,
        "image_dependencies_per_segment": image_dependencies_per_segment,
        "binding_cache_entry_count": binding_cache_entry_count,
        "text_dependencies_per_segment": text_dependencies_per_segment,
        "text_run_spans_per_segment": text_run_spans_per_segment,
        "font_dependencies_per_segment": font_dependencies_per_segment,
        "ui_commands_per_frame": ui_commands_per_frame,
        "compiled_vertices_per_text_dependency": compiled_vertices_per_text_dependency,
        "sdf_vertex_byte_len": sdf_vertex_byte_len,
        "material_uniform_stride": material_uniform_stride,
        "changed_segments_per_delta_frame": changed_segments_per_delta_frame,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if delta_frame_count <= 0:
        raise ValueError("delta_frame_count must be positive")
    if not 0 <= text_affecting_delta_frame_count <= delta_frame_count:
        raise ValueError(
            "text_affecting_delta_frame_count must fit in delta_frame_count"
        )
    text_partition = (
        text_paint_only_delta_frame_count,
        text_resident_glyph_delta_frame_count,
        text_new_glyph_delta_frame_count,
    )
    if all(value is None for value in text_partition):
        text_paint_only_delta_frame_count = min(
            1, text_affecting_delta_frame_count
        )
        text_new_glyph_delta_frame_count = min(
            1,
            text_affecting_delta_frame_count
            - text_paint_only_delta_frame_count,
        )
        text_resident_glyph_delta_frame_count = (
            text_affecting_delta_frame_count
            - text_paint_only_delta_frame_count
            - text_new_glyph_delta_frame_count
        )
    elif any(value is None for value in text_partition):
        raise ValueError("all text delta partition counts must be provided together")
    assert text_paint_only_delta_frame_count is not None
    assert text_resident_glyph_delta_frame_count is not None
    assert text_new_glyph_delta_frame_count is not None
    if min(
        text_paint_only_delta_frame_count,
        text_resident_glyph_delta_frame_count,
        text_new_glyph_delta_frame_count,
    ) < 0:
        raise ValueError("text delta partition counts must be non-negative")
    if (
        text_paint_only_delta_frame_count
        + text_resident_glyph_delta_frame_count
        + text_new_glyph_delta_frame_count
        != text_affecting_delta_frame_count
    ):
        raise ValueError(
            "text delta partition must equal text_affecting_delta_frame_count"
        )
    if resource_generation_frame_count < 0:
        raise ValueError("resource_generation_frame_count must be non-negative")
    if delta_frame_count + resource_generation_frame_count > frame_count:
        raise ValueError("delta and resource-generation frames must fit in frame_count")
    if changed_segments_per_delta_frame > segment_count:
        raise ValueError("changed segments must not exceed segment_count")

    stable_frame_count = (
        frame_count - delta_frame_count - resource_generation_frame_count
    )
    text_stable_delta_frame_count = (
        delta_frame_count - text_affecting_delta_frame_count
    )
    image_dependencies_per_frame = segment_count * image_dependencies_per_segment
    text_dependencies_per_frame = segment_count * text_dependencies_per_segment
    text_run_spans_per_frame = segment_count * text_run_spans_per_segment
    font_dependencies_per_frame = segment_count * font_dependencies_per_segment
    persistent_directory_depth = max(1, math.ceil(math.log2(segment_count)))

    rejected_image_segment_visits = frame_count * segment_count
    rejected_image_dependency_checks = frame_count * image_dependencies_per_frame
    rejected_binding_retention_visits = frame_count * binding_cache_entry_count
    rejected_upstream_command_visits = frame_count * ui_commands_per_frame
    rejected_upstream_dependency_prepare_visits = frame_count * image_dependencies_per_frame

    delta_segment_visits = delta_frame_count * changed_segments_per_delta_frame
    full_fallback_segment_visits = resource_generation_frame_count * segment_count
    target_image_segment_visits = delta_segment_visits + full_fallback_segment_visits
    delta_image_dependency_checks = (
        delta_frame_count
        * changed_segments_per_delta_frame
        * image_dependencies_per_segment
    )
    full_fallback_image_dependency_checks = (
        resource_generation_frame_count * image_dependencies_per_frame
    )
    target_image_dependency_checks = (
        delta_image_dependency_checks + full_fallback_image_dependency_checks
    )
    current_upstream_dependency_prepare_visits = target_image_dependency_checks

    rejected_text_delta_dependency_segment_visits = delta_frame_count * segment_count
    rejected_text_delta_dependency_entry_visits = (
        delta_frame_count * text_dependencies_per_frame
    )
    rejected_text_delta_run_segment_visits = delta_frame_count * segment_count
    rejected_text_delta_run_entry_visits = delta_frame_count * text_run_spans_per_frame
    current_text_stable_key_segment_visits = 0
    current_text_tail_delta_key_segment_visits = 0

    current_text_delta_input_match_segment_visits = (
        delta_frame_count * changed_segments_per_delta_frame
    )
    current_text_delta_input_match_entry_visits = (
        delta_frame_count
        * changed_segments_per_delta_frame
        * text_dependencies_per_segment
    )
    target_text_delta_dependency_segment_visits = (
        text_affecting_delta_frame_count * changed_segments_per_delta_frame
    )
    target_text_delta_dependency_entry_visits = (
        text_affecting_delta_frame_count
        * changed_segments_per_delta_frame
        * text_dependencies_per_segment
    )
    target_text_delta_run_directory_visits = text_affecting_delta_frame_count * (
        persistent_directory_depth + changed_segments_per_delta_frame
    )
    target_text_delta_run_entry_visits = (
        text_affecting_delta_frame_count
        * changed_segments_per_delta_frame
        * text_run_spans_per_segment
    )
    current_text_frame_product_arc_clone_count = (
        text_affecting_delta_frame_count * segment_count
    )
    avoided_text_stable_frame_product_arc_clone_count = (
        text_stable_delta_frame_count * segment_count
    )
    downstream_text_prepare_consumer_count = 3
    rejected_downstream_text_prepare_entry_visits = (
        delta_frame_count
        * text_dependencies_per_frame
        * downstream_text_prepare_consumer_count
    )
    current_downstream_text_prepare_entry_visits = (
        text_affecting_delta_frame_count
        * text_dependencies_per_frame
        * downstream_text_prepare_consumer_count
    )
    shaping_delta_frame_count = (
        text_resident_glyph_delta_frame_count
        + text_new_glyph_delta_frame_count
    )
    target_downstream_atlas_glyph_key_visits = (
        shaping_delta_frame_count
        * changed_segments_per_delta_frame
        * text_dependencies_per_segment
    )
    target_downstream_atlas_run_entry_visits = (
        shaping_delta_frame_count
        * changed_segments_per_delta_frame
        * text_run_spans_per_segment
    )
    target_downstream_cpu_run_entry_visits = (
        text_affecting_delta_frame_count
        * changed_segments_per_delta_frame
        * text_run_spans_per_segment
    )
    target_downstream_compiled_glyph_or_vertex_visits = (
        text_affecting_delta_frame_count
        * changed_segments_per_delta_frame
        * text_dependencies_per_segment
    )
    target_downstream_new_glyph_allocation_lookups = (
        text_new_glyph_delta_frame_count
        * changed_segments_per_delta_frame
        * text_dependencies_per_segment
    )
    target_downstream_consumer_entry_visits = (
        target_downstream_atlas_glyph_key_visits
        + target_downstream_cpu_run_entry_visits
        + target_downstream_compiled_glyph_or_vertex_visits
    )
    candidate_downstream_atlas_glyph_key_visits = (
        text_affecting_delta_frame_count
        * changed_segments_per_delta_frame
        * text_dependencies_per_segment
    )
    candidate_downstream_atlas_run_entry_visits = (
        text_affecting_delta_frame_count
        * changed_segments_per_delta_frame
        * text_run_spans_per_segment
    )
    candidate_downstream_consumer_entry_visits = (
        candidate_downstream_atlas_glyph_key_visits
        + target_downstream_cpu_run_entry_visits
        + target_downstream_compiled_glyph_or_vertex_visits
    )
    candidate_partial_vertex_write_bytes = (
        target_downstream_compiled_glyph_or_vertex_visits
        * compiled_vertices_per_text_dependency
        * sdf_vertex_byte_len
    )
    rejected_full_vertex_write_bytes = (
        text_affecting_delta_frame_count
        * text_dependencies_per_frame
        * compiled_vertices_per_text_dependency
        * sdf_vertex_byte_len
    )
    candidate_partial_material_write_bytes = (
        text_affecting_delta_frame_count
        * changed_segments_per_delta_frame
        * text_run_spans_per_segment
        * material_uniform_stride
    )
    rejected_full_material_write_bytes = (
        text_affecting_delta_frame_count
        * text_run_spans_per_frame
        * material_uniform_stride
    )
    avoided_downstream_text_prepare_entry_visits = (
        rejected_downstream_text_prepare_entry_visits
        - current_downstream_text_prepare_entry_visits
    )
    target_text_font_dependency_segment_visits = (
        delta_segment_visits + full_fallback_segment_visits
    )
    target_text_font_dependency_entry_visits = (
        target_text_font_dependency_segment_visits * font_dependencies_per_segment
    )
    ui_commands_per_segment = math.ceil(ui_commands_per_frame / segment_count)
    target_upstream_delta_command_visits = (
        delta_frame_count * changed_segments_per_delta_frame * ui_commands_per_segment
    )
    current_upstream_command_visits = target_upstream_delta_command_visits
    current_upstream_segment_identity_visits = delta_frame_count * segment_count

    full_fallback_text_dependency_segment_visits = (
        resource_generation_frame_count * segment_count
    )
    full_fallback_text_dependency_entry_visits = (
        resource_generation_frame_count * text_dependencies_per_frame
    )
    full_fallback_text_run_entry_visits = (
        resource_generation_frame_count * text_run_spans_per_frame
    )

    return {
        "schema": "zircon.runtime.ui_render_dependency_product_pressure.v9",
        "inputs": {
            **positive_inputs,
            "delta_frame_count": delta_frame_count,
            "text_affecting_delta_frame_count": text_affecting_delta_frame_count,
            "text_stable_delta_frame_count": text_stable_delta_frame_count,
            "resource_generation_frame_count": resource_generation_frame_count,
            "stable_frame_count": stable_frame_count,
            "image_dependencies_per_frame": image_dependencies_per_frame,
            "text_dependencies_per_frame": text_dependencies_per_frame,
            "text_run_spans_per_frame": text_run_spans_per_frame,
            "font_dependencies_per_frame": font_dependencies_per_frame,
            "ui_commands_per_segment": ui_commands_per_segment,
            "persistent_directory_depth": persistent_directory_depth,
            "text_delta_partition": {
                "paint_only_frame_count": text_paint_only_delta_frame_count,
                "resident_glyph_frame_count": (
                    text_resident_glyph_delta_frame_count
                ),
                "new_glyph_frame_count": text_new_glyph_delta_frame_count,
            },
        },
        "current_image_prepare": {
            "stable_frame_key_checks": frame_count,
            "segment_visits": target_image_segment_visits,
            "texture_dependency_checks": target_image_dependency_checks,
            "binding_lookups": target_image_dependency_checks,
            "binding_retention_entry_visits": 0,
        },
        "rejected_pre_product_image_prepare": {
            "segment_visits": rejected_image_segment_visits,
            "texture_dependency_checks": rejected_image_dependency_checks,
            "binding_lookups": rejected_image_dependency_checks,
            "binding_retention_entry_visits": rejected_binding_retention_visits,
        },
        "current_upstream_ui_texture_prepare": {
            "stable_submission_key_checks": frame_count,
            "segment_identity_visits": current_upstream_segment_identity_visits,
            "command_visits": current_upstream_command_visits,
            "dependency_prepare_visits": current_upstream_dependency_prepare_visits,
        },
        "rejected_pre_product_upstream_ui_texture_prepare": {
            "command_visits": rejected_upstream_command_visits,
            "dependency_prepare_visits": rejected_upstream_dependency_prepare_visits,
        },
        "target_upstream_texture_dependency_product": {
            "stable_submission_key_checks": frame_count,
            "segment_identity_visits": current_upstream_segment_identity_visits,
            "delta_command_visits": target_upstream_delta_command_visits,
            "resource_generation_command_visits": 0,
            "command_visits": current_upstream_command_visits,
            "dependency_prepare_visits": target_image_dependency_checks,
        },
        "target_image_dependency_product": {
            "stable_frame_key_checks": frame_count,
            "delta_segment_visits": delta_segment_visits,
            "full_fallback_segment_visits": full_fallback_segment_visits,
            "segment_visits": target_image_segment_visits,
            "delta_texture_dependency_checks": delta_image_dependency_checks,
            "full_fallback_texture_dependency_checks": (
                full_fallback_image_dependency_checks
            ),
            "texture_dependency_checks": target_image_dependency_checks,
            "binding_lookups": target_image_dependency_checks,
            "binding_retention_entry_visits": 0,
        },
        "current_text_delta_composition": {
            "stable_frame_dependency_visits": 0,
            "delta_dependency_segment_visits": (
                target_text_delta_dependency_segment_visits
            ),
            "delta_dependency_entry_visits": target_text_delta_dependency_entry_visits,
            "delta_run_directory_node_visits": target_text_delta_run_directory_visits,
            "delta_run_entry_visits": target_text_delta_run_entry_visits,
            "frame_product_segment_arc_clone_count": (
                current_text_frame_product_arc_clone_count
            ),
        },
        "current_text_delta_admission": {
            "changed_segment_visits": current_text_delta_input_match_segment_visits,
            "exact_text_input_entry_visits": current_text_delta_input_match_entry_visits,
            "frame_product_reuse_count": text_stable_delta_frame_count,
            "frame_product_republish_count": text_affecting_delta_frame_count,
        },
        "current_downstream_text_prepare": {
            "validation_status": "static_candidate",
            "consumer_count": downstream_text_prepare_consumer_count,
            "delta_entry_visits": candidate_downstream_consumer_entry_visits,
            "text_stable_delta_entry_visits": 0,
            "atlas_glyph_key_visits": candidate_downstream_atlas_glyph_key_visits,
            "atlas_run_entry_visits": candidate_downstream_atlas_run_entry_visits,
            "cpu_run_entry_visits": target_downstream_cpu_run_entry_visits,
            "compiled_glyph_or_vertex_visits": (
                target_downstream_compiled_glyph_or_vertex_visits
            ),
            "lower_bake_slot_equality_visits": 0,
            "same_cardinality_partial_vertex_write_bytes": (
                candidate_partial_vertex_write_bytes
            ),
            "same_cardinality_partial_material_write_bytes": (
                candidate_partial_material_write_bytes
            ),
            "structural_or_recovery_full_rebuild_is_typed": True,
        },
        "rejected_pre_journal_downstream_text_prepare": {
            "delta_entry_visits": current_downstream_text_prepare_entry_visits,
            "full_vertex_write_bytes": rejected_full_vertex_write_bytes,
            "full_material_write_bytes": rejected_full_material_write_bytes,
        },
        "target_downstream_text_prepare": {
            "atlas_glyph_key_visits": target_downstream_atlas_glyph_key_visits,
            "atlas_run_entry_visits": target_downstream_atlas_run_entry_visits,
            "cpu_run_entry_visits": target_downstream_cpu_run_entry_visits,
            "compiled_glyph_or_vertex_visits": (
                target_downstream_compiled_glyph_or_vertex_visits
            ),
            "bounded_new_glyph_allocation_lookups": (
                target_downstream_new_glyph_allocation_lookups
            ),
            "lower_bake_slot_equality_visits": 0,
            "consumer_entry_visits": target_downstream_consumer_entry_visits,
        },
        "rejected_pre_text_stable_frame_reuse": {
            "downstream_delta_entry_visits": (
                rejected_downstream_text_prepare_entry_visits
            ),
            "frame_product_segment_arc_clone_count": (
                delta_frame_count * segment_count
            ),
        },
        "rejected_pre_product_text_delta_composition": {
            "delta_dependency_segment_visits": (
                rejected_text_delta_dependency_segment_visits
            ),
            "delta_dependency_entry_visits": rejected_text_delta_dependency_entry_visits,
            "delta_run_segment_visits": rejected_text_delta_run_segment_visits,
            "delta_run_entry_visits": rejected_text_delta_run_entry_visits,
        },
        "current_text_frame_identity": {
            "frame_key_checks": frame_count,
            "stable_segment_identity_checks": current_text_stable_key_segment_visits,
            "tail_delta_segment_identity_checks": (
                current_text_tail_delta_key_segment_visits
            ),
            "resource_generation_segment_identity_checks": 0,
            "scenario": (
                "prepared frame generation is the O(1) authority; no segment identity "
                "scan is required for stable or local-delta admission"
            ),
        },
        "current_text_font_dependency_composition": {
            "segment_visits": target_text_font_dependency_segment_visits,
            "dependency_entry_visits": target_text_font_dependency_entry_visits,
        },
        "target_text_persistent_delta": {
            "frame_key_checks": frame_count,
            "stable_frame_dependency_visits": 0,
            "delta_dependency_segment_visits": (
                target_text_delta_dependency_segment_visits
            ),
            "delta_dependency_entry_visits": target_text_delta_dependency_entry_visits,
            "delta_run_directory_node_visits": target_text_delta_run_directory_visits,
            "delta_run_entry_visits": target_text_delta_run_entry_visits,
            "frame_product_segment_arc_clone_count": 0,
            "font_dependency_segment_visits": (
                target_text_font_dependency_segment_visits
            ),
            "font_dependency_entry_visits": target_text_font_dependency_entry_visits,
        },
        "typed_full_fallback": {
            "resource_generation_frame_count": resource_generation_frame_count,
            "image_segment_visits": full_fallback_segment_visits,
            "image_dependency_checks": full_fallback_image_dependency_checks,
            "text_dependency_segment_visits": (
                full_fallback_text_dependency_segment_visits
            ),
            "text_dependency_entry_visits": full_fallback_text_dependency_entry_visits,
            "text_run_entry_visits": full_fallback_text_run_entry_visits,
        },
        "delta": {
            "implemented_image_segment_visit_reduction_ratio": round(
                rejected_image_segment_visits / target_image_segment_visits, 6
            ),
            "implemented_image_dependency_check_reduction_ratio": round(
                rejected_image_dependency_checks / target_image_dependency_checks, 6
            ),
            "implemented_avoided_binding_retention_entry_visits": (
                rejected_binding_retention_visits
            ),
            "implemented_upstream_command_visit_reduction_ratio": round(
                rejected_upstream_command_visits / current_upstream_command_visits, 6
            ),
            "implemented_upstream_dependency_prepare_reduction_ratio": round(
                rejected_upstream_dependency_prepare_visits
                / target_image_dependency_checks,
                6,
            ),
            "implemented_text_delta_dependency_entry_reduction_ratio": (
                round(
                    rejected_text_delta_dependency_entry_visits
                    / target_text_delta_dependency_entry_visits,
                    6,
                )
                if target_text_delta_dependency_entry_visits
                else None
            ),
            "implemented_text_delta_run_entry_reduction_ratio": (
                round(
                    rejected_text_delta_run_entry_visits
                    / target_text_delta_run_entry_visits,
                    6,
                )
                if target_text_delta_run_entry_visits
                else None
            ),
            "implemented_avoided_text_stable_frame_product_arc_clones": (
                avoided_text_stable_frame_product_arc_clone_count
            ),
            "implemented_avoided_downstream_text_prepare_entry_visits": (
                avoided_downstream_text_prepare_entry_visits
            ),
            "static_candidate_downstream_entry_reduction_ratio": (
                round(
                    current_downstream_text_prepare_entry_visits
                    / candidate_downstream_consumer_entry_visits,
                    6,
                )
                if candidate_downstream_consumer_entry_visits
                else None
            ),
            "static_candidate_partial_vertex_write_reduction_ratio": (
                round(
                    rejected_full_vertex_write_bytes
                    / candidate_partial_vertex_write_bytes,
                    6,
                )
                if candidate_partial_vertex_write_bytes
                else None
            ),
            "static_candidate_partial_material_write_reduction_ratio": (
                round(
                    rejected_full_material_write_bytes
                    / candidate_partial_material_write_bytes,
                    6,
                )
                if candidate_partial_material_write_bytes
                else None
            ),
            "target_downstream_text_prepare_entry_reduction_ratio": (
                round(
                    current_downstream_text_prepare_entry_visits
                    / target_downstream_consumer_entry_visits,
                    6,
                )
                if target_downstream_consumer_entry_visits
                else None
            ),
        },
        "interpretation": {
            "timing_claim": False,
            "included": (
                "stable/delta/resource-generation state partition, upstream UI command "
                "discovery and texture dependency prepare, image segment and texture "
                "dependency visits, binding lookup and retention-map entry visits, text "
                "frame identity comparisons, font dependency composition, frame glyph "
                "dependency composition, run-span composition, persistent-directory depth, "
                "downstream atlas/CPU/compiled-frame text preparation admission, "
                "paint/resident/new-glyph invalidation partitions, and lower-bake slot "
                "equality visits, plus same-cardinality glyph-vertex and aligned-material "
                "write bytes"
            ),
            "excluded": (
                "actual CPU/GPU time, allocator latency, cache-line effects, hash-map "
                "constants, decoration vertex bytes, draw calls, texture upload bytes, RSS, asynchronous "
                "resource completion, transient UI texture failure retry frequency, font "
                "asset readiness polling, downstream glyph generation/upload work, and "
                "product input-to-present latency"
            ),
            "scope": (
                "deterministic current-source residual-operation model; resource "
                "generation changes are typed full fallbacks and are not hidden inside "
                "the changed-segment reduction"
            ),
        },
    }


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest().upper()


def _bound_source(source_root: Path, relative_path: str) -> dict[str, str]:
    path = source_root / relative_path
    source = path.read_text(encoding="utf-8")
    missing = [anchor for anchor in SOURCE_ANCHORS[relative_path] if anchor not in source]
    if missing:
        raise ValueError(
            f"source contract changed for {relative_path}; missing anchors: {missing}"
        )
    return {"relative_path": relative_path, "sha256": _sha256(path)}


def _source_set_sha256(entries: list[dict[str, str]]) -> str:
    digest = hashlib.sha256()
    for entry in sorted(entries, key=lambda item: item["relative_path"]):
        digest.update(entry["relative_path"].encode("utf-8"))
        digest.update(b"\0")
        digest.update(entry["sha256"].encode("ascii"))
        digest.update(b"\n")
    return digest.hexdigest().upper()


def build_source_binding(source_root: Path) -> dict[str, Any]:
    revision = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=source_root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    critical_sources = [
        _bound_source(source_root, relative_path)
        for relative_path in CRITICAL_SOURCE_FILES
    ]
    reference_sources = [
        _bound_source(source_root, relative_path)
        for relative_path in REFERENCE_SOURCE_FILES
    ]
    dirty_lines = subprocess.run(
        ["git", "status", "--short", "--", *CRITICAL_SOURCE_FILES],
        cwd=source_root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    return {
        "git_revision": revision,
        "dirty_paths": [line[3:] for line in dirty_lines if len(line) > 3],
        "critical_source_files": critical_sources,
        "reference_source_files": reference_sources,
        "source_set_sha256": _source_set_sha256(
            [*critical_sources, *reference_sources]
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--frame-count", type=int, default=4_096)
    parser.add_argument("--segment-count", type=int, default=64)
    parser.add_argument("--image-dependencies-per-segment", type=int, default=4)
    parser.add_argument("--binding-cache-entry-count", type=int, default=512)
    parser.add_argument("--text-dependencies-per-segment", type=int, default=32)
    parser.add_argument("--text-run-spans-per-segment", type=int, default=8)
    parser.add_argument("--font-dependencies-per-segment", type=int, default=2)
    parser.add_argument("--ui-commands-per-frame", type=int, default=1_024)
    parser.add_argument("--compiled-vertices-per-text-dependency", type=int, default=6)
    parser.add_argument("--sdf-vertex-byte-len", type=int, default=60)
    parser.add_argument("--material-uniform-stride", type=int, default=256)
    parser.add_argument("--delta-frame-count", type=int, default=32)
    parser.add_argument("--text-affecting-delta-frame-count", type=int, default=4)
    parser.add_argument("--text-paint-only-delta-frame-count", type=int)
    parser.add_argument("--text-resident-glyph-delta-frame-count", type=int)
    parser.add_argument("--text-new-glyph-delta-frame-count", type=int)
    parser.add_argument("--changed-segments-per-delta-frame", type=int, default=1)
    parser.add_argument("--resource-generation-frame-count", type=int, default=4)
    parser.add_argument("--source-root", type=Path, default=Path(__file__).resolve().parents[4])
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = run(
        frame_count=args.frame_count,
        segment_count=args.segment_count,
        image_dependencies_per_segment=args.image_dependencies_per_segment,
        binding_cache_entry_count=args.binding_cache_entry_count,
        text_dependencies_per_segment=args.text_dependencies_per_segment,
        text_run_spans_per_segment=args.text_run_spans_per_segment,
        font_dependencies_per_segment=args.font_dependencies_per_segment,
        ui_commands_per_frame=args.ui_commands_per_frame,
        compiled_vertices_per_text_dependency=(
            args.compiled_vertices_per_text_dependency
        ),
        sdf_vertex_byte_len=args.sdf_vertex_byte_len,
        material_uniform_stride=args.material_uniform_stride,
        delta_frame_count=args.delta_frame_count,
        text_affecting_delta_frame_count=args.text_affecting_delta_frame_count,
        text_paint_only_delta_frame_count=args.text_paint_only_delta_frame_count,
        text_resident_glyph_delta_frame_count=(
            args.text_resident_glyph_delta_frame_count
        ),
        text_new_glyph_delta_frame_count=args.text_new_glyph_delta_frame_count,
        changed_segments_per_delta_frame=args.changed_segments_per_delta_frame,
        resource_generation_frame_count=args.resource_generation_frame_count,
    )
    source_root = args.source_root.resolve()
    result["source_binding"] = build_source_binding(source_root)
    result["source_binding"]["model_source_sha256"] = _sha256(Path(__file__).resolve())
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        output = validate_output_path(args.output.resolve())
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
