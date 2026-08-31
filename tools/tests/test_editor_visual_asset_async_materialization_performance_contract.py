from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
LOADING = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/loading.rs"
)
PIXELS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/loading/pixels.rs"
)
ASYNC_LOADER = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/loading/async_loader.rs"
)
PIXEL_CACHE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/loading/cache.rs"
)
WINDOW = ROOT / "zircon_editor/src/ui/retained_host/host_contract/window.rs"
WINDOW_LIFECYCLE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs"
)
WINDOW_REDRAW = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs"
)
EVENT_LOOP_LIFECYCLE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs"
)
EVENT_WAKE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_wake.rs"
)
STARTUP = ROOT / (
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport.rs"
)
JOB_LIMITS = ROOT / "zircon_editor/src/core/jobs/limits.rs"
RENDER_COMMAND_IMAGE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "render_command_conversion/image.rs"
)
TEMPLATE_NODE_IMAGE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_images/command.rs"
)
THUMBNAIL_PREVIEW = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_asset_placeholder_visuals/preview_image.rs"
)
TEMPLATE_ICON_ASSETS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_icon_assets.rs"
)
PLACEHOLDER_VISUALS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_asset_placeholder_visuals.rs"
)
AVATAR_ICON = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "material_primitives/avatar/image/icon.rs"
)
AVATAR_PIXELS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "material_primitives/avatar/image/pixels.rs"
)
AVATAR_COMMANDS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "material_primitives/avatar/commands.rs"
)
AVATAR_CONTENT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "material_primitives/avatar/commands/content.rs"
)


class EditorVisualAssetAsyncMaterializationPerformanceContract(unittest.TestCase):
    def test_visual_asset_misses_use_the_shared_bounded_thumbnail_jobs(self) -> None:
        loading = LOADING.read_text(encoding="utf-8")
        loader = ASYNC_LOADER.read_text(encoding="utf-8")
        limits = JOB_LIMITS.read_text(encoding="utf-8")

        self.assertIn("mod async_loader;", loading)
        self.assertIn("EditorJobSystem", loader)
        self.assertIn("EditorJobSpec::new", loader)
        self.assertIn("JobCategory::Thumbnail", loader)
        self.assertIn("JobPriority::Background", loader)
        self.assertIn("DEFAULT_THUMBNAIL_LIMIT: usize = 2", limits)
        self.assertNotIn("std::thread::spawn", loader)
        self.assertNotIn("thread::spawn", loader)

    def test_bound_paint_miss_returns_before_file_decode_or_svg_raster(self) -> None:
        source = PIXELS.read_text(encoding="utf-8")
        entry = source.split("fn load_pixels_from_candidates", 1)[1].split(
            "fn load_visual_asset_pixels_uncached", 1
        )[0]
        uncached = source.split("fn load_visual_asset_pixels_uncached", 1)[1]

        self.assertIn("schedule_visual_asset_load", entry)
        self.assertIn("VisualAssetLoadSchedule::Deferred", entry)
        self.assertIn("return None", entry)
        self.assertNotIn("first_existing_path", entry)
        self.assertNotIn("render_svg_file_pixels", entry)
        self.assertNotIn("load_image_from_path_for_target", entry)
        self.assertIn("first_existing_path", uncached)
        self.assertIn("render_svg_file_pixels", uncached)

    def test_same_cache_key_is_coalesced_before_a_second_job_is_submitted(self) -> None:
        source = ASYNC_LOADER.read_text(encoding="utf-8")

        self.assertIn("pending_keys: BTreeMap<String, VisualAssetPendingLoad>", source)
        self.assertIn("reserve_pending_key", source)
        self.assertIn("merge_pending_damage_frame", source)
        self.assertIn("union_optional_frames", source)
        self.assertIn("VisualAssetAsyncDeduplicatedCount", source)
        self.assertIn("pending_loads_deduplicate_same_key", source)
        self.assertIn("duplicate_pending_loads_union_damage_frames", source)

    def test_cache_invalidation_epoch_rejects_stale_background_results(self) -> None:
        loader = ASYNC_LOADER.read_text(encoding="utf-8")
        cache = PIXEL_CACHE.read_text(encoding="utf-8")

        self.assertIn("begin_visual_asset_source_load", loader)
        self.assertIn("store_visual_asset_pixels_if_snapshot", loader)
        self.assertIn("VISUAL_ASSET_CACHE_EPOCH", cache)
        self.assertIn("advance_visual_asset_cache_epoch", cache)
        self.assertIn("snapshot.clear_epoch", cache)
        self.assertIn("source_snapshot_is_current", cache)
        self.assertIn("changed_source_rejects_only_its_pending_background_product", cache)
        self.assertIn("stale_completion_after_cache_clear_cannot_release_a_new_generation", cache)

    def test_stale_window_binding_cannot_publish_background_pixels(self) -> None:
        loader = ASYNC_LOADER.read_text(encoding="utf-8")
        job_run = loader.split("fn run", 1)[1].split("impl Drop", 1)[0]
        self.assertIn("fn store_visual_asset_pixels_for_current_binding", loader)
        guarded_store = loader.split(
            "fn store_visual_asset_pixels_for_current_binding", 1
        )[1].split("fn release_pending_key", 1)[0]

        self.assertIn("store_visual_asset_pixels_for_current_binding", job_run)
        self.assertIn("let scheduler = lock_scheduler()", guarded_store)
        self.assertIn("!scheduler.binding_is_current(binding_epoch)", guarded_store)
        self.assertIn("store_visual_asset_pixels_if_snapshot", guarded_store)
        self.assertIn("stale_binding_cannot_publish_visual_pixels", loader)

    def test_completion_uses_a_dedicated_wake_and_local_present_only_redraw(self) -> None:
        window = WINDOW.read_text(encoding="utf-8")
        lifecycle = WINDOW_LIFECYCLE.read_text(encoding="utf-8")
        redraw = WINDOW_REDRAW.read_text(encoding="utf-8")
        event_loop = EVENT_LOOP_LIFECYCLE.read_text(encoding="utf-8")

        self.assertIn("visual_asset_wake: event_wake::HostEventLoopWake", window)
        self.assertIn("bind_visual_asset_jobs", lifecycle)
        self.assertIn("take_visual_asset_completion_wake", redraw)
        completion = event_loop.split("take_visual_asset_completion_wake", 1)[1].split(
            "take_background_event_wake", 1
        )[0]
        compact_completion = "".join(completion.split()).replace(",)", ")")
        self.assertIn("take_visual_asset_completion", completion)
        self.assertIn(
            "region_for_scenario(completion.scenario,damage_frame)",
            compact_completion,
        )
        self.assertIn("full_frame_for_scenario(completion.scenario,false)", compact_completion)
        self.assertNotIn("request_frame_update", completion)
        self.assertNotIn("request_maintenance_frame_update", completion)

    def test_background_and_visual_native_wakes_share_edge_coalescing(self) -> None:
        window = WINDOW.read_text(encoding="utf-8")
        wake = EVENT_WAKE.read_text(encoding="utf-8")

        self.assertIn("event_wake: event_wake::HostEventLoopWake", window)
        self.assertIn("visual_asset_wake: event_wake::HostEventLoopWake", window)
        self.assertIn("if !mark_wake_pending(&self.state.requested)", wake)
        self.assertIn("requested.swap(true, Ordering::AcqRel)", wake)
        self.assertIn("wake_callback_coalesces_until_the_event_loop_consumes_it", wake)
        self.assertIn("native_wake_is_signaled_only_on_the_pending_edge", wake)

    def test_visual_asset_completion_damage_is_captured_at_paint_time(self) -> None:
        loader = ASYNC_LOADER.read_text(encoding="utf-8")
        pixels = PIXELS.read_text(encoding="utf-8")

        self.assertIn("damage_frame: Option<FrameRect>", loader)
        self.assertIn("damage_frame: Option<FrameRect>", pixels)
        self.assertIn("schedule_visual_asset_load", pixels)
        self.assertNotIn("render_commands.iter()", loader)
        self.assertNotIn("presentation.nodes.iter()", loader)

    def test_product_visual_misses_publish_their_clipped_damage_frame(self) -> None:
        render_image = RENDER_COMMAND_IMAGE.read_text(encoding="utf-8")
        template_image = TEMPLATE_NODE_IMAGE.read_text(encoding="utf-8")
        preview = THUMBNAIL_PREVIEW.read_text(encoding="utf-8")
        icons = TEMPLATE_ICON_ASSETS.read_text(encoding="utf-8")
        placeholders = PLACEHOLDER_VISUALS.read_text(encoding="utf-8")
        avatar_icon = AVATAR_ICON.read_text(encoding="utf-8")
        avatar_pixels = AVATAR_PIXELS.read_text(encoding="utf-8")
        avatar_commands = AVATAR_COMMANDS.read_text(encoding="utf-8")
        avatar_content = AVATAR_CONTENT.read_text(encoding="utf-8")

        self.assertIn("intersect(&frame, clip_frame)", render_image)
        self.assertIn("Some(damage_frame)", render_image)
        self.assertIn("intersect(&materialization_rect, clip)", template_image)
        self.assertIn("Some(damage_frame)", template_image)
        self.assertIn("intersect(rect, clip)", preview)
        self.assertIn("Some(damage_frame)", preview)
        self.assertIn("intersect(rect, clip)", icons)
        self.assertIn("Some(damage_frame)", icons)
        self.assertGreaterEqual(placeholders.count("Some(damage_frame)"), 2)
        self.assertIn("intersect(&avatar_rect, clip)", avatar_commands)
        self.assertIn("damage_frame", avatar_pixels)
        self.assertIn("intersect(&icon_rect, clip)", avatar_content)
        self.assertIn("damage_frame", avatar_icon)

    def test_startup_binds_the_existing_job_system_before_first_paint(self) -> None:
        source = STARTUP.read_text(encoding="utf-8")
        bind = source.index("ui.bind_visual_asset_jobs(")
        construct = source.index("construct_startup_host(")

        self.assertLess(bind, construct)
        self.assertIn("background_visual_asset_wake_callback", source)


if __name__ == "__main__":
    unittest.main()
