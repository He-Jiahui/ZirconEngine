from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
VISUAL_ROOT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets"
)
KEYS = VISUAL_ROOT / "keys.rs"
PIXELS = VISUAL_ROOT / "loading/pixels.rs"
PIXEL_CACHE = VISUAL_ROOT / "loading/cache.rs"
ASYNC_LOADER = VISUAL_ROOT / "loading/async_loader.rs"
SVG_CACHE = VISUAL_ROOT / "svg/cache.rs"
TARGET = VISUAL_ROOT / "target.rs"
ICON_ATLAS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
    "icon_atlas.rs"
)
IMAGE_RESOURCES = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
    "stream/image_resources.rs"
)
PRESENTER = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/present.rs"
)
WGPU_IMAGE_CACHE = ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs"
WGPU_IMAGE_RESOURCE = ROOT / (
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache/resource.rs"
)
WGPU_SHARED_IMAGE_REGISTRY = ROOT / (
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry.rs"
)
WGPU_IMAGE_ALLOCATION_LEDGER = ROOT / (
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/"
    "shared_image_registry/allocation_ledger.rs"
)
WGPU_UI_SURFACE = ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs"
WGPU_UI_PRESENTATION = ROOT / (
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs"
)
WGPU_NATIVE_RECORDING = ROOT / (
    "zircon_runtime/crates/zr_rhi_wgpu/src/production/device/native_recording.rs"
)
WGPU_SUBMISSION = ROOT / (
    "zircon_runtime/crates/zr_rhi_wgpu/src/production/submission.rs"
)
WGPU_UI_IMAGE_RETIREMENT = ROOT / (
    "zircon_runtime/crates/zr_rhi_wgpu/src/production/submission/"
    "ui_image_retirement.rs"
)
RHI_UI_SURFACE = ROOT / "zircon_runtime/crates/zr_rhi/src/ui_surface.rs"
EDITOR_GPU_STATS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs"
)
EDITOR_UI_PERF = ROOT / "zircon_editor/src/ui/retained_host/ui_perf.rs"
EDITOR_UI_PERF_CATALOG = ROOT / (
    "zircon_editor/src/ui/retained_host/ui_perf/counter_catalog.rs"
)


class EditorSvgGpuResidencyDesignContract(unittest.TestCase):
    def test_svg_identity_is_content_addressed_and_bucketed_before_lookup(self) -> None:
        keys = KEYS.read_text(encoding="utf-8")
        pixels = PIXELS.read_text(encoding="utf-8")
        target = TARGET.read_text(encoding="utf-8")

        self.assertIn("image_content_fingerprint(width, height, rgba)", keys)
        self.assertIn('format!("retained-image:{width}x{height}:', keys)
        self.assertIn('format!("icon-raster:{content_key}")', pixels)
        self.assertIn("vector_cache_bucket", target)
        self.assertIn("quantized_up", target)

    def test_warm_raster_and_svg_tree_hits_precede_filesystem_or_parse_work(self) -> None:
        pixels = PIXELS.read_text(encoding="utf-8")
        svg_cache = SVG_CACHE.read_text(encoding="utf-8")

        load_entry = pixels.split("fn load_pixels_from_candidates_with_status", 1)[1]
        load_entry = load_entry.split("pub(super) fn load_visual_asset_pixels_uncached", 1)[0]
        self.assertLess(
            load_entry.index("cached_visual_asset_pixels"),
            load_entry.index("let candidates ="),
        )
        self.assertNotIn("first_existing_path", load_entry)
        self.assertNotIn("render_svg_file_pixels", load_entry)

        svg_entry = svg_cache.split("fn load_svg_tree_with_parser", 1)[1]
        svg_entry = svg_entry.split("pub(in crate::ui::retained_host) fn invalidate", 1)[0]
        self.assertLess(
            svg_entry.index("get_by_query_path"),
            svg_entry.index("SvgTreeCacheKey::from_path"),
        )
        self.assertLess(
            svg_entry.index("get_by_query_path"),
            svg_entry.index("parse_svg_tree_file"),
        )

    def test_targeted_invalidation_keeps_content_identity_authoritative(self) -> None:
        pixel_cache = PIXEL_CACHE.read_text(encoding="utf-8")
        async_loader = ASYNC_LOADER.read_text(encoding="utf-8")

        self.assertIn("source_fingerprints", pixel_cache)
        self.assertIn("invalidate_paths", pixel_cache)
        self.assertIn("base_key_content_changed", pixel_cache)
        self.assertIn("source_generations", pixel_cache)
        self.assertIn("pending_base_loads", pixel_cache)
        self.assertIn("refresh_source_fingerprint_baseline", pixel_cache)
        self.assertIn("VISUAL_ASSET_CACHE_EPOCH", pixel_cache)
        self.assertIn("begin_visual_asset_source_load", pixel_cache)
        self.assertIn("store_visual_asset_pixels_if_snapshot", pixel_cache)
        self.assertIn("finish_visual_asset_source_load", pixel_cache)

        targeted = pixel_cache.split(
            "fn invalidate_visual_asset_pixel_paths", 1
        )[1].split("fn reconcile_visual_asset_pixel_sources", 1)[0]
        reconcile = pixel_cache.split(
            "fn reconcile_visual_asset_pixel_sources", 1
        )[1].split("fn clear_visual_asset_pixels_cache", 1)[0]
        self.assertNotIn("advance_visual_asset_cache_epoch", targeted)
        self.assertNotIn("advance_visual_asset_cache_epoch", reconcile)
        production_cache = pixel_cache.split("#[cfg(test)]", 1)[0]
        self.assertEqual(production_cache.count("advance_visual_asset_cache_epoch();"), 1)

        self.assertIn("begin_visual_asset_source_load", async_loader)
        self.assertIn("store_visual_asset_pixels_if_snapshot", async_loader)
        self.assertIn("finish_visual_asset_source_load", async_loader)
        self.assertNotIn("cache_epoch: visual_asset_cache_epoch()", async_loader)
        binding_store = async_loader.split(
            "fn store_visual_asset_pixels_for_current_binding", 1
        )[1].split("fn release_pending_key", 1)[0]
        self.assertIn("finish_visual_asset_source_load(source_snapshot);", binding_store)

    def test_atlas_pages_are_immutable_versioned_and_bounded(self) -> None:
        atlas = ICON_ATLAS.read_text(encoding="utf-8")

        self.assertIn("struct IconSourceVersion", atlas)
        self.assertIn("generation: payload.resource_generation", atlas)
        self.assertIn("page.generation = NEXT_ICON_ATLAS_GENERATION", atlas)
        self.assertIn("page.sealed = true", atlas)
        self.assertIn("MAX_ICON_ATLAS_PAGES", atlas)
        self.assertIn("MAX_ICON_ATLAS_BYTES", atlas)

    def test_resident_generation_omits_pixels_and_device_registry_owns_texture_write(
        self,
    ) -> None:
        resources = IMAGE_RESOURCES.read_text(encoding="utf-8")
        presenter = PRESENTER.read_text(encoding="utf-8")
        wgpu = WGPU_IMAGE_CACHE.read_text(encoding="utf-8")
        registry = WGPU_SHARED_IMAGE_REGISTRY.read_text(encoding="utf-8")

        compact = resources.split("fn compact_image_resources_with_residency", 1)[1]
        compact = compact.split("#[cfg(test)]", 1)[0]
        self.assertIn("if resident", compact)
        self.assertIn("payload.rgba = None", compact)
        self.assertIn("is_image_resource_resident", presenter)
        self.assertIn("last_uploaded_generation == Some(generation)", wgpu)
        self.assertIn("image_upload_needs_write", wgpu)
        self.assertNotIn("queue.write_texture", wgpu)
        self.assertIn("queue.write_texture", registry)
        self.assertIn("MAX_UI_IMAGE_CACHE_ENTRIES", wgpu)
        self.assertIn("MAX_UI_IMAGE_CACHE_BYTES", wgpu)

    def test_device_ledger_owns_shared_allocations_until_surface_and_gpu_pins_release(
        self,
    ) -> None:
        self.assertTrue(
            WGPU_IMAGE_ALLOCATION_LEDGER.exists(),
            "the device allocation ledger module must exist",
        )
        registry = WGPU_SHARED_IMAGE_REGISTRY.read_text(encoding="utf-8")
        ledger = WGPU_IMAGE_ALLOCATION_LEDGER.read_text(encoding="utf-8")
        cache = WGPU_IMAGE_CACHE.read_text(encoding="utf-8")
        resource = WGPU_IMAGE_RESOURCE.read_text(encoding="utf-8")
        surface = WGPU_UI_SURFACE.read_text(encoding="utf-8")
        presentation = WGPU_UI_PRESENTATION.read_text(encoding="utf-8")
        native_recording = WGPU_NATIVE_RECORDING.read_text(encoding="utf-8")
        submission = WGPU_SUBMISSION.read_text(encoding="utf-8")
        retirement = WGPU_UI_IMAGE_RETIREMENT.read_text(encoding="utf-8")
        rhi = RHI_UI_SURFACE.read_text(encoding="utf-8")

        self.assertIn("mod allocation_ledger;", registry)
        self.assertIn("allocation_ledger: Arc<WgpuUiImageAllocationLedger>", registry)
        self.assertIn("allocation_stats", registry)
        self.assertIn("is_exclusively_registry_owned", registry)
        self.assertIn("entry_count: usize", registry)
        self.assertNotIn("fn shared_entry_count", registry)

        self.assertIn("struct WgpuUiImageAllocationLedger", ledger)
        self.assertIn("texture: Option<wgpu::Texture>", ledger)
        self.assertIn("unique_allocation_bytes", ledger)
        self.assertIn("registry_evicted_pinned_bytes", ledger)
        self.assertIn("surface_pin_count", ledger)
        self.assertIn("in_flight_present_pin_count", ledger)
        self.assertIn("eviction_completion_count", ledger)
        self.assertIn("fn begin_in_flight", ledger)

        self.assertIn("shared_allocation_pin", resource)
        self.assertNotIn("pub(super) texture: wgpu::Texture", resource)
        self.assertNotIn("pub(super) cpu_rgba", resource)

        rejected = cache.split("SharedImagePrepareResult::Rejected =>", 1)[1]
        rejected = rejected.split("let cached_size", 1)[0]
        self.assertIn("continue 'source", rejected)
        self.assertNotIn("WgpuUiImageResource::new(", cache)
        self.assertIn("prepared_allocation_set", cache)
        self.assertIn("pin_prepared_allocations_for_submission", cache)
        self.assertIn("entry_count: usize", cache)
        self.assertNotIn("self.resources.values().map(BTreeMap::len).sum()", cache)

        self.assertNotIn("queue.on_submitted_work_done", surface)
        self.assertIn("packet.retain_ui_image_pins(pins)", surface)
        self.assertIn("submit_native_recording_packet(packet)", surface)
        self.assertIn("pin_prepared_allocations_for_submission", presentation)
        self.assertLess(
            presentation.index("pin_prepared_allocations_for_submission"),
            presentation.index(
                "submit_present_command_buffer(encoder.finish(), image_allocation_pins)"
            ),
        )
        self.assertIn("ui_image_pins: Option<WgpuUiImageInFlightPins>", native_recording)
        self.assertIn("commit_packet_with_ui_image_pins(", native_recording)
        submit_batch = submission.split("fn submit_native_batch(", 1)[1]
        self.assertIn("self.ui_image_retirements", submit_batch)
        self.assertIn(".retain_batch(", submit_batch)
        self.assertIn("queue.on_submitted_work_done", submit_batch)
        self.assertIn(
            "completion_retirements.complete(&completed_tickets)", submit_batch
        )
        self.assertLess(
            submit_batch.index("self.queue.submit("),
            submit_batch.index(".retain_batch("),
        )
        self.assertIn(
            "HashMap<SubmissionTicket, WgpuUiImageInFlightPins>", retirement
        )
        self.assertIn("pending.remove(ticket)", retirement)

        for field in (
            "image_device_allocation_count",
            "image_device_allocation_bytes",
            "image_registry_evicted_pinned_bytes",
            "image_surface_pin_count",
            "image_in_flight_present_pin_count",
            "image_eviction_completion_count",
        ):
            self.assertIn(field, rhi)

    def test_device_ledger_stats_reach_scenario_prefixed_editor_counters(self) -> None:
        catalog = EDITOR_UI_PERF_CATALOG.read_text(encoding="utf-8")
        ui_perf = EDITOR_UI_PERF.read_text(encoding="utf-8")
        stats = EDITOR_GPU_STATS.read_text(encoding="utf-8")
        counters = (
            ("GpuImageDeviceAllocationCount", "image_device_allocation_count"),
            ("GpuImageDeviceAllocationBytes", "image_device_allocation_bytes"),
            (
                "GpuImageRegistryEvictedPinnedBytes",
                "image_registry_evicted_pinned_bytes",
            ),
            ("GpuImageSurfacePinCount", "image_surface_pin_count"),
            (
                "GpuImageInFlightPresentPinCount",
                "image_in_flight_present_pin_count",
            ),
            ("GpuImageEvictionCompletionCount", "image_eviction_completion_count"),
        )

        for counter, field in counters:
            self.assertIn(f"{counter},", catalog)
            name_arm = ui_perf.split(f"UiPerfCounter::{counter} =>", 1)[1]
            name_arm = name_arm.split("UiPerfCounter::", 1)[0]
            self.assertIn(f'concat!($prefix, ".gpu_{field}")', name_arm)
            self.assertIn(f"UiPerfCounter::{counter}", stats)
            self.assertIn(f"stats.{field} as f64", stats)

        self.assertIn("const BASE_PRESENT_STAT_COUNTER_COUNT: usize = 52;", stats)
        self.assertIn("assert_eq!(counters.len(), 52);", stats)
        self.assertIn("assert_eq!(counters.len(), 55);", stats)


if __name__ == "__main__":
    unittest.main()
