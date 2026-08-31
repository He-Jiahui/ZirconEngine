#!/usr/bin/env python3
"""Model Editor SVG materialization and GPU residency work.

This is a deterministic algorithm-pressure model, not measured product timing.
It compares repeated per-command reconstruction with the current retained
source/tree/raster/atlas/GPU identity contract and models the remaining stable
external-provider lookup work in the product presenter path.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


MIB = 1024 * 1024

CRITICAL_SOURCE_CONTRACTS = (
    (
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        "visual_assets/loading/cache.rs",
        (
            "static VISUAL_ASSET_CACHE_EPOCH",
            "advance_visual_asset_cache_epoch();",
            "source_generations: BTreeMap",
            "pending_base_loads: BTreeMap",
            "refresh_source_fingerprint_baseline",
            "begin_visual_asset_source_load",
            "store_visual_asset_pixels_if_snapshot",
            "finish_visual_asset_source_load",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        "visual_assets/loading/async_loader.rs",
        (
            "begin_visual_asset_source_load",
            "UiPerfCounter::VisualAssetAsyncStaleDiscardCount",
            "store_visual_asset_pixels_if_snapshot",
            "finish_visual_asset_source_load",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/app/assets/refresh.rs",
        (
            "VisualAssetCacheRefresh::Paths(paths)",
            "invalidate_visual_asset_pixel_paths",
            "VisualAssetCacheRefresh::Reconcile",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
        "visual_assets/svg/pixels.rs",
        ("visual_assets_render_svg_raster", "render_svg_tree_pixels"),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
        "icon_atlas.rs",
        (
            "const MAX_ICON_ATLAS_BYTES: usize = 64 * 1024 * 1024;",
            "rgba: Arc<[u8]>",
            "payload.rgba = Some(Arc::clone(&page.rgba));",
        ),
    ),
    (
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs",
        (
            "const MAX_UI_IMAGE_CACHE_BYTES: u64 = 64 * 1024 * 1024;",
            "shared_images.prepare_pixels(",
            "WgpuUiImageResource::from_external(",
            "let external_images_present = external_images.is_some();",
            "!external_images_present && image_resources.is_empty()",
            "provider.resolve(cache_key, image_generation)",
        ),
    ),
    (
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache/resource.rs",
        (
            "bind_group: wgpu::BindGroup",
            "shared_allocation_pin: Option<WgpuUiImageSurfacePin>",
        ),
    ),
    (
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry.rs",
        (
            "const MAX_SHARED_UI_IMAGE_BYTES: u64 = 64 * 1024 * 1024;",
            "Device-scoped static UI texture storage",
            "allocation_ledger: Arc<WgpuUiImageAllocationLedger>",
            "remove_shared_image(&mut state",
        ),
    ),
    (
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/"
        "shared_image_registry/allocation_ledger.rs",
        (
            "texture: Option<wgpu::Texture>",
            "unique_allocation_bytes",
            "registry_evicted_pinned_bytes",
            "surface_pin_count",
            "in_flight_present_pin_count",
            "eviction_completion_count",
            "fn begin_in_flight",
        ),
    ),
    (
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs",
        (
            "pub trait WgpuUiSurfaceExternalImageProvider: Send + Sync",
            "fn resolve(&self, resource_key: &str, generation: u64)",
            "shared_image_registry: Arc<WgpuUiSharedImageRegistry>",
            "image_cache: WgpuUiImageCache::default(),",
            "packet.retain_ui_image_pins(pins);",
            "self.render_device.submit_native_recording_packet(packet)",
        ),
    ),
    (
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs",
        (
            "pin_prepared_allocations_for_submission",
            "image_allocation_pins",
            "submit_present_command_buffer(encoder.finish(), image_allocation_pins)",
        ),
    ),
    (
        "zircon_runtime/crates/zr_rhi_wgpu/src/production/device/native_recording.rs",
        (
            "ui_image_pins: Option<WgpuUiImageInFlightPins>",
            "pub(crate) fn retain_ui_image_pins",
            "packet.into_submission_parts()",
            "commit_packet_with_ui_image_pins(",
        ),
    ),
    (
        "zircon_runtime/crates/zr_rhi_wgpu/src/production/submission.rs",
        (
            "commit_packet_with_ui_image_pins(",
            "self.ui_image_retirements",
            "retain_batch(std::mem::take(ui_image_retirements))",
            "self.queue.on_submitted_work_done",
            "completion_retirements.complete(&completed_tickets)",
        ),
    ),
    (
        "zircon_runtime/crates/zr_rhi_wgpu/src/production/submission/"
        "ui_image_retirement.rs",
        (
            "HashMap<SubmissionTicket, WgpuUiImageInFlightPins>",
            "pub(super) fn retain_batch",
            "pub(super) fn complete",
            "pending.remove(ticket)",
        ),
    ),
    (
        "zircon_runtime/crates/zr_rhi/src/ui_surface.rs",
        (
            "image_device_allocation_count",
            "image_device_allocation_bytes",
            "image_registry_evicted_pinned_bytes",
            "image_surface_pin_count",
            "image_in_flight_present_pin_count",
            "image_eviction_completion_count",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs",
        (
            "stats.image_device_allocation_count as f64",
            "stats.image_device_allocation_bytes as f64",
            "stats.image_registry_evicted_pinned_bytes as f64",
            "stats.image_surface_pin_count as f64",
            "stats.image_in_flight_present_pin_count as f64",
            "stats.image_eviction_completion_count as f64",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/ui_perf.rs",
        (
            'concat!($prefix, ".gpu_image_device_allocation_count")',
            'concat!($prefix, ".gpu_image_device_allocation_bytes")',
            'concat!($prefix, ".gpu_image_registry_evicted_pinned_bytes")',
            'concat!($prefix, ".gpu_image_surface_pin_count")',
            'concat!($prefix, ".gpu_image_in_flight_present_pin_count")',
            'concat!($prefix, ".gpu_image_eviction_completion_count")',
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/ui_perf/counter_catalog.rs",
        (
            "GpuImageDeviceAllocationCount",
            "GpuImageDeviceAllocationBytes",
            "GpuImageRegistryEvictedPinnedBytes",
            "GpuImageSurfacePinCount",
            "GpuImageInFlightPresentPinCount",
            "GpuImageEvictionCompletionCount",
        ),
    ),
    (
        "zircon_runtime/src/graphics/runtime/render_framework/"
        "render_framework_trait_binding/wgpu_framework.rs",
        (
            "WgpuViewportProductProvider::new(Arc::clone(",
            "new_with_context_and_external_images(",
            "Some(provider),",
        ),
    ),
    (
        "zircon_runtime/src/graphics/runtime/render_framework/"
        "render_framework_state/viewport_product_registry.rs",
        (
            "impl WgpuUiSurfaceExternalImageProvider for WgpuViewportProductProvider",
            "self.products.resolve(resource_key, generation)",
            "let products = self\n            .products\n            .lock()",
            "products.by_resource_key.get(resource_key)?",
        ),
    ),
)


class SourceContractError(RuntimeError):
    """Raised when the pressure model no longer describes current source."""


def _validate_source_generation_contract(relative_path: str, source: str) -> None:
    if relative_path.endswith("visual_assets/loading/cache.rs"):
        targeted = source.split("fn invalidate_visual_asset_pixel_paths", 1)[1].split(
            "fn reconcile_visual_asset_pixel_sources", 1
        )[0]
        reconcile = source.split("fn reconcile_visual_asset_pixel_sources", 1)[1].split(
            "fn clear_visual_asset_pixels_cache", 1
        )[0]
        if "advance_visual_asset_cache_epoch" in targeted:
            raise SourceContractError(
                "targeted visual-asset invalidation advances the global clear epoch"
            )
        if "advance_visual_asset_cache_epoch" in reconcile:
            raise SourceContractError(
                "visual-asset reconciliation advances the global clear epoch"
            )
    if relative_path.endswith("visual_assets/loading/async_loader.rs") and (
        "cache_epoch: visual_asset_cache_epoch()" in source
        or "store_visual_asset_pixels_if_epoch" in source
    ):
        raise SourceContractError(
            "async visual-asset publication is still bound to the global epoch"
        )
    if relative_path.endswith("ui_surface/image_cache.rs") and (
        "WgpuUiImageResource::new(" in source or "queue.write_texture" in source
    ):
        raise SourceContractError(
            "a surface image cache can still create or upload a local texture"
        )
    if relative_path.endswith("ui_surface/image_cache/resource.rs") and (
        "pub(super) texture: wgpu::Texture" in source
        or "pub(super) cpu_rgba" in source
    ):
        raise SourceContractError(
            "a surface bind product still owns an unledgered texture or CPU payload"
        )
    if relative_path.endswith("zr_rhi_wgpu/src/ui_surface.rs"):
        if "on_submitted_work_done" in source:
            raise SourceContractError(
                "a UI Surface still owns image-pin completion instead of the device timeline"
            )
        pin_index = source.index("packet.retain_ui_image_pins(pins);")
        submit_index = source.index("self.render_device.submit_native_recording_packet(packet)")
        if not pin_index < submit_index:
            raise SourceContractError(
                "UI image pins are not attached before the native packet is submitted"
            )
    if relative_path.endswith("ui_surface/presentation.rs"):
        pin_index = source.index("pin_prepared_allocations_for_submission")
        submit_index = source.index(
            "submit_present_command_buffer(encoder.finish(), image_allocation_pins)"
        )
        if not pin_index < submit_index:
            raise SourceContractError(
                "image allocations are not pinned before UI command submission"
            )
    if relative_path.endswith("production/device/native_recording.rs"):
        unpack_index = source.index("packet.into_submission_parts()")
        commit_index = source.index("commit_packet_with_ui_image_pins(", unpack_index)
        if not unpack_index < commit_index:
            raise SourceContractError(
                "native recording does not transfer UI image pins into submission ownership"
            )
    if relative_path.endswith("production/submission.rs"):
        submit_owner = source.split("fn submit_native_batch(", 1)[1]
        queue_submit_index = submit_owner.index("self.queue.submit(")
        retain_index = submit_owner.index(
            "retain_batch(std::mem::take(ui_image_retirements))"
        )
        callback_index = submit_owner.index("self.queue.on_submitted_work_done")
        complete_index = submit_owner.index(
            "completion_retirements.complete(&completed_tickets)"
        )
        if not queue_submit_index < retain_index < callback_index < complete_index:
            raise SourceContractError(
                "submission timeline does not retain and complete UI image pins in GPU order"
            )


def _sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest().upper()


def _git_output(repo_root: Path, *args: str) -> str | None:
    try:
        completed = subprocess.run(
            ["git", *args],
            cwd=repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    return completed.stdout.strip()


def source_binding_report(repo_root: Path) -> dict[str, object]:
    repo_root = repo_root.resolve()
    sources = []
    source_set = hashlib.sha256()
    relative_paths = []
    for relative_path, required_tokens in CRITICAL_SOURCE_CONTRACTS:
        path = repo_root / relative_path
        try:
            payload = path.read_bytes()
        except OSError as error:
            raise SourceContractError(f"missing critical source: {relative_path}") from error
        source = payload.decode("utf-8")
        missing = [token for token in required_tokens if token not in source]
        if missing:
            raise SourceContractError(
                f"critical source contract changed: {relative_path}: {missing}"
            )
        _validate_source_generation_contract(relative_path, source)
        digest = _sha256(payload)
        sources.append(
            {
                "relative_path": relative_path,
                "sha256": digest,
                "byte_length": len(payload),
            }
        )
        relative_paths.append(relative_path)
        source_set.update(relative_path.encode("utf-8"))
        source_set.update(b"\0")
        source_set.update(digest.encode("ascii"))
        source_set.update(b"\n")

    dirty_output = _git_output(repo_root, "status", "--porcelain=v1", "--", *relative_paths)
    dirty_entries = [] if not dirty_output else dirty_output.splitlines()
    return {
        "ready": True,
        "git_revision": _git_output(repo_root, "rev-parse", "HEAD"),
        "critical_sources_dirty": bool(dirty_entries),
        "critical_source_dirty_entry_count": len(dirty_entries),
        "source_set_sha256": source_set.hexdigest().upper(),
        "critical_sources": sources,
    }


def pressure_report(
    stable_present_count: int = 10_000,
    image_commands_per_present: int = 2_048,
    unique_svg_source_count: int = 256,
    raster_variants_per_source: int = 4,
    average_svg_source_bytes: int = 4_096,
    raster_edge: int = 32,
    atlas_page_count: int = 16,
    atlas_page_edge: int = 256,
    changed_svg_source_count: int = 1,
    changed_atlas_page_count: int = 1,
    inflight_svg_product_count: int = 256,
    unrelated_path_event_count: int = 100,
    unchanged_path_event_count: int = 100,
    affected_path_event_count: int = 1,
    affected_svg_product_count: int = 4,
    ui_surface_count: int = 16,
) -> dict[str, object]:
    positive = {
        "stable_present_count": stable_present_count,
        "image_commands_per_present": image_commands_per_present,
        "unique_svg_source_count": unique_svg_source_count,
        "raster_variants_per_source": raster_variants_per_source,
        "average_svg_source_bytes": average_svg_source_bytes,
        "raster_edge": raster_edge,
        "atlas_page_count": atlas_page_count,
        "atlas_page_edge": atlas_page_edge,
        "inflight_svg_product_count": inflight_svg_product_count,
        "ui_surface_count": ui_surface_count,
    }
    for name, value in positive.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if not 0 <= changed_svg_source_count <= unique_svg_source_count:
        raise ValueError(
            "changed_svg_source_count must be within unique_svg_source_count"
        )
    if not 0 <= changed_atlas_page_count <= atlas_page_count:
        raise ValueError("changed_atlas_page_count must be within atlas_page_count")
    for name, value in {
        "unrelated_path_event_count": unrelated_path_event_count,
        "unchanged_path_event_count": unchanged_path_event_count,
        "affected_path_event_count": affected_path_event_count,
    }.items():
        if value < 0:
            raise ValueError(f"{name} must not be negative")
    if not 0 <= affected_svg_product_count <= inflight_svg_product_count:
        raise ValueError(
            "affected_svg_product_count must be within inflight_svg_product_count"
        )

    rgba_bytes_per_raster = raster_edge * raster_edge * 4
    rgba_bytes_per_atlas_page = atlas_page_edge * atlas_page_edge * 4
    raster_product_count = unique_svg_source_count * raster_variants_per_source
    stable_image_command_visits = stable_present_count * image_commands_per_present

    repeated_file_reads = stable_image_command_visits
    repeated_source_read_bytes = repeated_file_reads * average_svg_source_bytes
    repeated_tree_parses = stable_image_command_visits
    repeated_rasterizations = stable_image_command_visits
    repeated_raster_bytes = repeated_rasterizations * rgba_bytes_per_raster
    repeated_gpu_upload_writes = stable_image_command_visits
    repeated_gpu_upload_bytes = repeated_raster_bytes

    retained_file_reads = unique_svg_source_count + changed_svg_source_count
    retained_source_read_bytes = retained_file_reads * average_svg_source_bytes
    retained_tree_parses = retained_file_reads
    retained_rasterizations = raster_product_count + (
        changed_svg_source_count * raster_variants_per_source
    )
    retained_raster_bytes = retained_rasterizations * rgba_bytes_per_raster
    retained_gpu_upload_writes = atlas_page_count + changed_atlas_page_count
    retained_gpu_upload_bytes = retained_gpu_upload_writes * rgba_bytes_per_atlas_page
    retained_residency_probes = stable_present_count * atlas_page_count
    current_provider_resolve_calls = stable_present_count * atlas_page_count
    target_provider_revision_checks = stable_present_count

    layer_budget_bytes = 64 * MIB
    configured_budget_ceiling_bytes = layer_budget_bytes * 4
    pre_ledger_reachable_unique_gpu_allocation_bytes = (
        ui_surface_count * layer_budget_bytes
    )
    device_ledger_unique_allocation_bytes = layer_budget_bytes
    device_budget_rejected_working_set_bytes = (
        pre_ledger_reachable_unique_gpu_allocation_bytes
        - device_ledger_unique_allocation_bytes
    )
    physical_budget_overshoot_bytes = max(
        0, device_ledger_unique_allocation_bytes - layer_budget_bytes
    )
    invalidation_event_count = (
        unrelated_path_event_count
        + unchanged_path_event_count
        + affected_path_event_count
    )
    global_epoch_stale_discards = inflight_svg_product_count * invalidation_event_count
    global_epoch_raster_attempts = inflight_svg_product_count + global_epoch_stale_discards
    source_generation_stale_discards = (
        affected_svg_product_count * affected_path_event_count
    )
    source_generation_raster_attempts = (
        inflight_svg_product_count + source_generation_stale_discards
    )

    return {
        "schema": "zircon.editor.svg_gpu_residency_pressure.v5",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": {
            **positive,
            "changed_svg_source_count": changed_svg_source_count,
            "changed_atlas_page_count": changed_atlas_page_count,
            "unrelated_path_event_count": unrelated_path_event_count,
            "unchanged_path_event_count": unchanged_path_event_count,
            "affected_path_event_count": affected_path_event_count,
            "affected_svg_product_count": affected_svg_product_count,
        },
        "repeated_per_command_reconstruction_baseline": {
            "svg_file_reads": repeated_file_reads,
            "svg_source_read_bytes": repeated_source_read_bytes,
            "svg_tree_parses": repeated_tree_parses,
            "svg_rasterizations": repeated_rasterizations,
            "svg_raster_bytes_materialized": repeated_raster_bytes,
            "gpu_upload_writes": repeated_gpu_upload_writes,
            "gpu_upload_bytes": repeated_gpu_upload_bytes,
            "stable_complexity": "O(P * I * (file + parse + raster + upload))",
        },
        "retained_content_addressed_residency": {
            "cold_and_one_reload_svg_file_reads": retained_file_reads,
            "cold_and_one_reload_svg_source_read_bytes": retained_source_read_bytes,
            "cold_and_one_reload_svg_tree_parses": retained_tree_parses,
            "cold_and_one_reload_svg_rasterizations": retained_rasterizations,
            "cold_and_one_reload_svg_raster_bytes_materialized": retained_raster_bytes,
            "cold_and_one_reload_gpu_page_upload_writes": retained_gpu_upload_writes,
            "cold_and_one_reload_gpu_page_upload_bytes": retained_gpu_upload_bytes,
            "stable_svg_file_reads": 0,
            "stable_svg_tree_parses": 0,
            "stable_svg_rasterizations": 0,
            "stable_gpu_upload_writes": 0,
            "stable_gpu_upload_bytes": 0,
            "stable_image_command_visits": stable_image_command_visits,
            "stable_resource_generation_residency_probes": retained_residency_probes,
            "stable_complexity": "O(P * I) command/reference work; materialization is O(changed products)",
            "stable_image_command_visits_scope": "conceptual command/reference upper bound for the historical reconstruction comparison; the current WGPU prepare loop is modeled separately by unique image sources",
        },
        "external_provider_fast_path_pressure": {
            "status": "current_source_residual",
            "scenario": "unchanged draw-list generation, unchanged external provider products, and one retained source per atlas page",
            "provider_installed_for_every_ui_surface": True,
            "generation_fast_path_enabled": False,
            "stable_image_source_count": atlas_page_count,
            "current_provider_resolve_calls": current_provider_resolve_calls,
            "current_registry_lock_acquisitions": current_provider_resolve_calls,
            "target_provider_revision_checks": target_provider_revision_checks,
            "target_provider_resolve_calls": 0,
            "avoided_provider_resolve_calls": current_provider_resolve_calls,
            "current_stable_complexity": "O(P * R)",
            "target_stable_complexity": "O(P)",
            "target_authority": "cache prepared draw-list generation together with a monotonic external-provider revision; an unchanged pair returns before the source loop",
            "stable_svg_file_reads": 0,
            "stable_svg_tree_parses": 0,
            "stable_svg_rasterizations": 0,
            "stable_gpu_upload_writes": 0,
            "is_product_timing": False,
        },
        "delta": {
            "avoided_svg_file_reads": repeated_file_reads - retained_file_reads,
            "svg_file_read_reduction_ratio": round(
                repeated_file_reads / retained_file_reads, 6
            ),
            "avoided_svg_tree_parses": repeated_tree_parses - retained_tree_parses,
            "svg_tree_parse_reduction_ratio": round(
                repeated_tree_parses / retained_tree_parses, 6
            ),
            "avoided_svg_rasterizations": (
                repeated_rasterizations - retained_rasterizations
            ),
            "svg_rasterization_reduction_ratio": round(
                repeated_rasterizations / retained_rasterizations, 6
            ),
            "avoided_gpu_upload_writes": (
                repeated_gpu_upload_writes - retained_gpu_upload_writes
            ),
            "gpu_upload_write_reduction_ratio": round(
                repeated_gpu_upload_writes / retained_gpu_upload_writes, 6
            ),
            "avoided_gpu_upload_bytes": (
                repeated_gpu_upload_bytes - retained_gpu_upload_bytes
            ),
            "gpu_upload_byte_reduction_ratio": round(
                repeated_gpu_upload_bytes / retained_gpu_upload_bytes, 6
            ),
        },
        "memory_budget_warning": {
            "visual_raster_cache_budget_bytes": layer_budget_bytes,
            "editor_icon_atlas_budget_bytes": layer_budget_bytes,
            "per_surface_gpu_image_cache_budget_bytes": layer_budget_bytes,
            "shared_gpu_image_registry_budget_bytes": layer_budget_bytes,
            "configured_four_layer_ceiling_bytes": configured_budget_ceiling_bytes,
            "configured_four_layer_ceiling_is_mixed_unit_sum": True,
            "note": "The legacy four-layer sum mixes CPU payload bytes, unique GPU allocations, and per-surface references; it is not a physical process-memory ceiling.",
        },
        "multi_surface_residency_pressure": {
            "scenario": "surfaces sequentially request disjoint full-budget image working sets while earlier surfaces retain allocation pins",
            "ui_surface_count": ui_surface_count,
            "per_surface_reference_budget_bytes": layer_budget_bytes,
            "pre_ledger_reachable_unique_gpu_allocation_bytes": pre_ledger_reachable_unique_gpu_allocation_bytes,
            "device_ledger_unique_allocation_bytes": device_ledger_unique_allocation_bytes,
            "device_budget_rejected_working_set_bytes": device_budget_rejected_working_set_bytes,
            "physical_budget_overshoot_bytes": physical_budget_overshoot_bytes,
            "pre_ledger_budget_overshoot_ratio": round(
                pre_ledger_reachable_unique_gpu_allocation_bytes
                / layer_budget_bytes,
                6,
            ),
            "device_ledger_budget_ratio": round(
                device_ledger_unique_allocation_bytes / layer_budget_bytes, 6
            ),
            "device_allocation_budget_bytes": layer_budget_bytes,
            "current_authority": "one device-scoped allocation record owns each texture and remains budgeted until registry, surface, prepared-set, and in-flight pins release",
            "admission_contract": "when existing surface pins consume the physical budget, later disjoint working sets are rejected instead of creating presenter-local textures",
            "counter_contract": "unique allocation bytes are physical authority; registry and surface resident bytes remain lookup/reference diagnostics and must not be summed as physical memory",
        },
        "global_epoch_invalidation": {
            "status": "historical_removed_current_source_baseline",
            "model_assumption": "the former implementation advanced the cache epoch for each path event before one full in-flight product wave published, then rescheduled the discarded wave",
            "invalidation_event_count": invalidation_event_count,
            "stale_discard_count": global_epoch_stale_discards,
            "raster_attempt_count": global_epoch_raster_attempts,
            "unrelated_event_stale_discard_count": (
                inflight_svg_product_count * unrelated_path_event_count
            ),
            "unchanged_event_stale_discard_count": (
                inflight_svg_product_count * unchanged_path_event_count
            ),
            "affected_event_stale_discard_count": (
                inflight_svg_product_count * affected_path_event_count
            ),
            "complexity": "O(I * (U + C + A + 1)) raster attempts",
        },
        "source_generation_invalidation": {
            "status": "current_source_contract",
            "model_assumption": "unrelated and content-unchanged events preserve pending products; a changed source invalidates only products depending on that source",
            "invalidation_event_count": invalidation_event_count,
            "stale_discard_count": source_generation_stale_discards,
            "raster_attempt_count": source_generation_raster_attempts,
            "unrelated_event_stale_discard_count": 0,
            "unchanged_event_stale_discard_count": 0,
            "affected_event_stale_discard_count": source_generation_stale_discards,
            "avoided_stale_discard_count": (
                global_epoch_stale_discards - source_generation_stale_discards
            ),
            "avoided_raster_attempt_count": (
                global_epoch_raster_attempts - source_generation_raster_attempts
            ),
            "complexity": "O(I + D * A) raster attempts",
        },
        "interpretation": {
            "included": "file reads, SVG parses, raster products, immutable atlas page uploads, conceptual stable command visits, source-counted stable external-provider resolves and registry lock acquisitions, one targeted source reload, global-epoch stale completion pressure, source-targeted completion pressure, configured cache ceilings, and a source-bound disjoint multi-surface residency scenario",
            "excluded": "actual CPU/GPU time, filesystem cache effects, compression, allocator latency, measured lock contention, exact atlas packing density, damage-region command counts, measured eviction churn, RSS, driver residency, and device recreation",
            "required_product_evidence": "current-source cold/warm SVG parse and raster counters, async enqueue/completion/stale-discard counters, image upload counters, resource churn top-N, CPU/RSS/GPU residency, and hover/resize p50/p95/p99",
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stable-present-count", type=int, default=10_000)
    parser.add_argument("--image-commands-per-present", type=int, default=2_048)
    parser.add_argument("--unique-svg-source-count", type=int, default=256)
    parser.add_argument("--raster-variants-per-source", type=int, default=4)
    parser.add_argument("--average-svg-source-bytes", type=int, default=4_096)
    parser.add_argument("--raster-edge", type=int, default=32)
    parser.add_argument("--atlas-page-count", type=int, default=16)
    parser.add_argument("--atlas-page-edge", type=int, default=256)
    parser.add_argument("--changed-svg-source-count", type=int, default=1)
    parser.add_argument("--changed-atlas-page-count", type=int, default=1)
    parser.add_argument("--inflight-svg-product-count", type=int, default=256)
    parser.add_argument("--unrelated-path-event-count", type=int, default=100)
    parser.add_argument("--unchanged-path-event-count", type=int, default=100)
    parser.add_argument("--affected-path-event-count", type=int, default=1)
    parser.add_argument("--affected-svg-product-count", type=int, default=4)
    parser.add_argument("--ui-surface-count", type=int, default=16)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = pressure_report(
        stable_present_count=args.stable_present_count,
        image_commands_per_present=args.image_commands_per_present,
        unique_svg_source_count=args.unique_svg_source_count,
        raster_variants_per_source=args.raster_variants_per_source,
        average_svg_source_bytes=args.average_svg_source_bytes,
        raster_edge=args.raster_edge,
        atlas_page_count=args.atlas_page_count,
        atlas_page_edge=args.atlas_page_edge,
        changed_svg_source_count=args.changed_svg_source_count,
        changed_atlas_page_count=args.changed_atlas_page_count,
        inflight_svg_product_count=args.inflight_svg_product_count,
        unrelated_path_event_count=args.unrelated_path_event_count,
        unchanged_path_event_count=args.unchanged_path_event_count,
        affected_path_event_count=args.affected_path_event_count,
        affected_svg_product_count=args.affected_svg_product_count,
        ui_surface_count=args.ui_surface_count,
    )
    result["source_binding"] = source_binding_report(Path(__file__).resolve().parents[1])
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        if args.output.drive.upper() == "C:":
            raise ValueError("profile artifacts must not be written to C:")
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload, encoding="utf-8")
    print(payload, end="")


if __name__ == "__main__":
    main()
