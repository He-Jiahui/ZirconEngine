---
related_code:
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/retained_cache.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/shaders/ui_material.wgsl
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/rhi_wgpu/tests.rs
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi_wgpu/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/sprite_atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream/atlas_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - tools/ui-profile-capture.ps1
implementation_files:
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/retained_cache.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/shaders/ui_material.wgsl
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/rhi_wgpu/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/sprite_atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream/atlas_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - tools/ui-profile-capture.ps1
plan_sources:
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - user: 2026-05-15 GPU command stream should take over editor UI rendering
  - user: 2026-05-15 UI rendering needs poset depth batching for retained editor UI
  - user: 2026-05-16 continue UI batching and interaction validation plan
  - .codex/plans/Retained Host Chrome GPU 化与 Hover 卡顿根因修复计划.md
  - .codex/plans/Retained Host Chrome 性能根因修复计划.md
  - .codex/plans/Editor 基础组件 Material 化视觉优化计划.md
  - .codex/plans/UI 合批与交互完整校验计划.md
  - .codex/plans/UI Surface Direct-Screen Rendering Plan.md
  - user: 2026-05-15 Material-like retained editor UI needs real rounded controls across software and GPU presenters
  - docs/superpowers/plans/2026-05-18-editor-sprite-atlas-ui-batching.md
tests:
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/retained_cache.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/shaders/ui_material.wgsl
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/rhi_wgpu/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/sprite_atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - cargo check -p zircon_runtime --lib --locked
  - cargo check -p zircon_editor --lib --locked
  - cargo test -p zircon_runtime --lib ui_surface --locked
  - cargo test -p zircon_runtime --lib draw_list_stats_skip_commands_outside_damage --locked
  - cargo test -p zircon_runtime --lib draw_list_stats_do_not_count_cached_images_as_uploads --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_text_bounds_clip_to_damage_and_command_clip --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_text_skips_disjoint_damage --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_draw_items_sort_by_stable_z_order --locked
  - cargo test -p zircon_runtime --lib batch_plan_keeps_same_z_overlaps_in_original_index_order --locked
  - cargo test -p zircon_runtime --lib batch_plan_splits_text_when_overlapping_geometry_depends --locked
  - cargo test -p zircon_runtime --lib batch_plan_batches_disjoint_images_with_same_resource_key_into_one_draw --locked
  - cargo test -p zircon_runtime --lib batch_plan_splits_overlapping_images_even_with_same_resource_key --locked
  - cargo test -p zircon_runtime --lib batch_plan_preserves_overlap_chain_between_same_resource_images --locked
  - cargo test -p zircon_runtime --lib batch_plan_batches_independent_same_resource_images_around_overlap --locked
  - cargo test -p zircon_runtime --lib atlas_uv_rect_validates_normalized_finite_bounds --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_image_uvs_compose_clipped_rect_with_atlas_uv --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_skips_image_with_invalid_atlas_uv --locked
  - cargo test -p zircon_runtime --lib draw_list_stats_count_same_resource_image_upload_once --locked
  - cargo test -p zircon_runtime --lib batch_plan_batches_disjoint_atlas_images_with_same_key_and_distinct_uvs --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_headless_stats_batch_atlas_images_by_resource_key --locked
  - cargo test -p zircon_editor --lib recorded_atlas_images_keep_shared_resource_key_and_distinct_uvs --locked
  - cargo test -p zircon_editor --lib recorded_atlas_image_uses_atlas_texture_payload_not_source_payload --locked
  - cargo test -p zircon_editor --lib resolver_reads_project_library_atlas_artifacts_for_template_icon --locked
  - cargo test -p zircon_editor --lib command_stream_replay_samples_atlas_uv_from_embedded_atlas_bytes --locked
  - cargo test -p zircon_editor --lib command_stream --locked --jobs 1
  - cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_presenter_uses_damage_for_patch_stats --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_render_mode_requires_initialized_cache_for_damage_patch --locked
  - cargo test -p zircon_runtime --lib native_ui_surface_source_uses_direct_surface_without_offscreen_blit --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_image_cache_prune --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_uses_non_srgb_formats_for_byte_exact_editor_parity --locked --jobs 1 --message-format short
  - cargo test -p zircon_editor --lib gpu_presenter_damage_present_uses_patch_after_surface_cache_is_ready --locked --jobs 1
  - cargo test -p zircon_editor --lib gpu_presenter_resize_invalidates_damage_cache --locked --jobs 1
  - cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1
  - cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1
  - cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1
  - cargo test -p zircon_runtime --lib material_button_style --locked --jobs 1
  - cargo test -p zircon_runtime --locked render_product_ui --jobs 1
  - 2026-05-19 runtime UI graph closeout: cargo test --workspace --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - cargo build -p zircon_app --bin zircon_editor --profile profiling --features "target-editor-host profiling profiling-chrome" --locked
  - tools/ui-profile-capture.ps1 -Scenario startup -AutoCloseSeconds 3 -SkipBuild
  - tools/ui-profile-capture.ps1 -Scenario viewport_image -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 4 -SkipBuild
  - cargo test -p zircon_editor --lib viewport_image_resource_key_tracks_same_size_content --locked
  - cargo test -p zircon_editor --lib draw_rgba_image_clipped_records_content_scoped_resource_keys --locked
  - cargo test -p zircon_editor --lib viewport_image_patch_can_carry_upload_bytes_for_gpu --locked
  - cargo test -p zircon_editor --lib gpu_presenter --locked
  - cargo test -p zircon_editor --lib render_framework_boundary --locked
  - cargo check -p zircon_app --features "target-editor-host" --locked
  - 2026-05-15 continuation: cargo check -p zircon_runtime --lib --locked
  - 2026-05-15 continuation: cargo check -p zircon_editor --lib --locked
  - 2026-05-15 continuation: cargo test -p zircon_runtime --lib ui_surface --locked
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib command_stream --locked
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib gpu_presenter --locked
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib render_framework_boundary --locked
  - 2026-05-15 continuation: cargo test -p zircon_runtime --lib ui_hotspot --locked
  - 2026-05-15 continuation: cargo check -p zircon_app --features target-editor-host --locked
  - 2026-05-15 M5: cargo check -p zircon_app --profile profiling --features "target-editor-host profiling profiling-chrome" --locked
  - 2026-05-15 M5: cargo build -p zircon_app --bin zircon_editor --profile profiling --features "target-editor-host profiling profiling-chrome" --locked
  - 2026-05-15 M5: cargo build -p zircon_runtime --lib --profile profiling --features "target-editor-host profiling profiling-chrome" --locked
  - 2026-05-15 M5: tools/ui-profile-capture.ps1 -ScenarioList startup,idle_hover,viewport_image,click,drag,drawer_resize,asset_refresh -AutoCloseSeconds 3 -SkipBuild (20260515-055615 through 20260515-055704)
  - 2026-05-15 workspace expansion: cargo build --workspace --locked --verbose
  - 2026-05-15 workspace expansion: cargo test --workspace --locked --verbose (blocked in zircon_editor template/demo-front tests)
  - 2026-05-15 workspace expansion: cargo test -p zircon_editor --lib --locked --message-format=short (1173 passed, 121 failed, 4 ignored; failure source outside GPU presenter/RHI surface)
  - 2026-05-15 closeout: cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1
  - 2026-05-15 closeout: cargo test -p zircon_editor --lib command_stream --locked --jobs 1
  - 2026-05-15 closeout: cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1
  - 2026-05-15 closeout: cargo test -p zircon_editor --lib render_framework_boundary --locked --jobs 1
  - 2026-05-15 closeout: git diff --check -- GPU command-stream plan/docs/session/touched startup module
  - 2026-05-15 continuation: cargo test -p zircon_runtime --lib app_editor_and_core_framework_sources_do_not_import_wgpu --locked --jobs 1 --message-format=short
  - 2026-05-15 continuation: cargo test -p zircon_runtime --lib production_ui_entry_assets_live_under_crate_assets_not_src --locked --jobs 1 --message-format=short
  - 2026-05-15 continuation: cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format=short -- --test-threads=1 (1349 passed)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib editor_retained_host_presenter_boundary_keeps_wgpu_inside_runtime_rhi --locked --jobs 1 --message-format=short
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib --locked --jobs 1 --message-format=short -- --test-threads=1 (1298 passed, 4 ignored)
  - 2026-05-15 continuation: cargo test -p zircon_runtime_interface --lib --locked --jobs 1 --message-format=short -- --test-threads=1 (95 passed)
  - 2026-05-15 continuation: cargo test -p zircon_app --lib --locked --jobs 1 --message-format=short -- --test-threads=1 (42 passed)
  - 2026-05-15 continuation: cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 -- --test-threads=1 (8 passed)
  - 2026-05-15 continuation: cargo fmt --all -- --check (passed)
  - 2026-05-15 continuation: cargo test --workspace --locked --jobs 1 --message-format=short -- --test-threads=1 (attempted twice; first exposed/link-blocked runtime_ui_text_render_contract before focused rerun passed, second timed out after 30 minutes with no residual processes)
  - 2026-05-15 hover closeout: cargo test -p zircon_editor --lib native_host_hierarchy_move_prefers_native_hover_when_template_node_overlaps --locked --jobs 1 --message-format=short
  - 2026-05-15 hover closeout: cargo test -p zircon_editor --lib native_host_template_node_move_updates_hover_without_rebuilding_presentation --locked --jobs 1 --message-format=short
  - 2026-05-15 hover closeout: cargo test -p zircon_editor --lib native_host_hierarchy_move --locked --jobs 1 --message-format=short
  - 2026-05-15 hover closeout: cargo test -p zircon_editor --lib native_host_asset_template --locked --jobs 1 --message-format=short
  - 2026-05-15 hover closeout: cargo test -p zircon_editor --lib native_host_asset_tree_move_updates_visible_hover_state --locked --jobs 1 --message-format=short
  - 2026-05-15 hover closeout: tools/ui-profile-capture.ps1 -Scenario idle_hover -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 3 -SkipBuild (20260515-211644-idle_hover)
  - 2026-05-15 hover closeout: tools/ui-profile-capture.ps1 -Scenario click -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 3 -SkipBuild (20260515-205945-click)
  - 2026-05-16 viewport closeout: cargo test -p zircon_editor --lib frame_update_region_queues_external_redraw_with_frame_update --locked --jobs 1 --message-format=short
  - 2026-05-16 viewport closeout: cargo test -p zircon_editor --lib close_requested_callback_can_mutate_host_state_without_reentrant_borrow --locked --jobs 1 --message-format=short
  - 2026-05-16 viewport closeout: cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format=short (35 passed)
  - 2026-05-16 viewport closeout: cargo test -p zircon_runtime --lib ui_hotspot --locked --jobs 1 --message-format=short (9 passed)
  - 2026-05-16 viewport closeout: cargo test -p zircon_editor --lib command_stream --locked --jobs 1 --message-format=short (7 passed)
  - 2026-05-16 viewport closeout: cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format=short (2 passed)
  - 2026-05-16 viewport closeout: cargo check -p zircon_app --features target-editor-host --locked --message-format=short
  - 2026-05-16 viewport closeout: tools/ui-profile-capture.ps1 -Scenario startup -RequireScenarioEvidence -AutoCloseSeconds 3 -SkipBuild (20260516-001744-startup)
  - 2026-05-16 viewport closeout: tools/ui-profile-capture.ps1 -Scenario idle_hover -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 3 -SkipBuild (20260516-001914-idle_hover)
  - 2026-05-16 viewport closeout: tools/ui-profile-capture.ps1 -Scenario viewport_image -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 3 -SkipBuild (20260516-000208-viewport_image)
  - 2026-05-15 Material visual slice: cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format short --color never (35 passed)
  - 2026-05-15 Material visual slice: cargo test -p zircon_editor --lib command_stream --locked --jobs 1 --message-format short --color never (7 passed)
  - 2026-05-15 Material visual slice: cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format short --color never (2 passed)
  - 2026-05-15 Material visual slice: cargo test -p zircon_editor --lib gpu_surface_commands_preserve_chrome_corner_radius --locked --jobs 1 --message-format short --color never (1 passed)
  - 2026-05-16 Material visual live capture: tools/ui-profile-capture.ps1 -ScenarioList startup,idle_hover -OutputRoot .codex/material-ui-capture -SkipBuild -AutoCloseSeconds 5 -AutoInteract -RequireScenarioEvidence (startup 20260516-000244 passed; idle_hover 20260516-000253 recorded redraw/GPU work with zero alerts but missed the strict batch gate)
  - 2026-05-16 Material visual live capture: tools/ui-profile-capture.ps1 -Scenario click -OutputRoot .codex/material-ui-capture -SkipBuild -AutoCloseSeconds 5 -AutoInteract -RequireScenarioEvidence (20260516-000343 passed)
  - tools/ui-profile-capture.ps1 -Scenario startup -AutoCloseSeconds 3 -SkipBuild (20260515-013306-startup)
  - tools/ui-profile-capture.ps1 -ScenarioList startup,idle_hover,viewport_image,click,drag,drawer_resize,asset_refresh -AutoCloseSeconds 3 -SkipBuild (20260515-033851 through 20260515-033926)
doc_type: module-detail
---

# RHI UI Surface

`zircon_runtime::rhi::ui_surface` is the runtime-owned contract for presenting retained editor UI without exposing raw `wgpu` or concrete `rhi_wgpu` providers to `zircon_editor`. The editor converts `ChromeCommandStream` into `UiSurfaceDrawList`; runtime owns the surface descriptor, native target extraction, concrete presenter factory, surface resize, texture upload, draw execution, and present lifecycle.

`UiSurfaceDrawList` carries the surface size, optional damage, and ordered commands. The command vocabulary is backend-neutral: quads, borders, text runs, images, and clips. Quad and border commands retain `corner_radius` so editor Material-style controls do not lose rounded geometry when the command stream crosses from the retained host into runtime RHI. Image payloads can include RGBA bytes and upload byte counts, so the GPU presenter can update viewport/image textures without asking the editor to render a full CPU frame.

`UiSurfacePresenter` is the neutral presenter trait. Native implementations must resize, present a draw list, and expose the last present stats. The stats are intentionally small and profiling-facing: surface size, actual GPU batch draw calls, visible command count, visible draw item count, image upload bytes, image count, clip count, batch layer/dependency counts, and presented frame count. `UiSurfaceDrawList::stats()` still provides the backend-neutral visibility and upload baseline; the WGPU presenter replaces `draw_calls` and batch fields with the real batch plan it submitted.

`UiSurfaceDescriptor::from_winit_window(...)` is the editor-facing native-window conversion point when `platform-winit` is enabled. It translates a host `winit` window into the neutral `RenderNativeSurfaceTarget` and surface size without requiring the editor to name `wgpu` or `rhi_wgpu`. `create_default_ui_surface_presenter(...)` is the runtime-owned factory that selects the current concrete UI surface provider and returns `Box<dyn UiSurfacePresenter>`. This keeps backend selection inside `zircon_runtime::rhi` while preserving the existing `GpuChromePresenter<P: UiSurfacePresenter>` test seam.

`zircon_runtime::rhi_wgpu::WgpuUiSurfacePresenter` has two modes:

- Native descriptors create a real wgpu instance, adapter, device, queue, Win32 surface, non-sRGB swapchain configuration, direct solid/image pipelines, and glyphon text renderer. The native UI path renders those batches straight into the acquired swapchain surface view.
- Headless descriptors keep the same contract and stats without creating a native surface. Unit tests use this mode so CI does not require a window.

The native WGPU path intentionally keeps UI color in non-sRGB byte space. `ChromeCommandStream` colors, embedded image bytes, softbuffer screenshots, and retained-host reference snapshots are already display-space byte RGBA values; rendering them through an sRGB target would apply hardware encoding and make the GPU capture visibly brighter than the software path. The presenter therefore requires `Bgra8Unorm` or `Rgba8Unorm` as the swapchain render target, uses `Rgba8Unorm` for uploaded image textures, clears the first direct surface pass to opaque black, and prefers `CompositeAlphaMode::Opaque` when the swapchain supports it. If a native surface only exposes sRGB formats, WGPU UI presenter initialization fails and the editor can use the existing fallback path instead of silently accepting color-space drift. `wgpu_ui_surface_uses_non_srgb_formats_for_byte_exact_editor_parity` and `wgpu_ui_surface_prefers_opaque_swapchain_alpha` pin that visual-parity contract.

The native renderer supports damage without relying on the swapchain's previous contents. `WgpuRetainedSurfaceCache` owns a non-sRGB texture matching the current swapchain format and size. Full frames render the full draw list directly into the acquired swapchain view and then record the same draw ops into the retained cache, marking it initialized after submission. Damage frames require a matching initialized cache: the renderer first restores the cache into the current swapchain view with a fullscreen cache-restore draw, then records only damage-intersecting draw ops into both the swapchain view and the cache view. Cache maintenance is internal to the direct-present path; it does not restore the old offscreen render target plus final-blit architecture.

`GpuChromePresenter` protects the cache preconditions at the editor boundary. The first native GPU present, any present without damage, and the first present after resize build a full command stream so runtime can seed the retained cache with complete UI contents. Once a successful present has initialized that cache, subsequent damaged host presents build patch command streams and carry `UiSurfaceDrawList::damage` through to runtime. Headless presenter stats remain damage-aware for tests, while native presenter `draw_calls` reports the surface presentation work, including the cache restore draw on patch frames.

`rhi_wgpu::ui_surface::geometry` owns the draw-list geometry layer: stable `(z_index, command_index, sub_index)` ordering, quad/image vertices, rounded solid tessellation, rounded border rings, image UV trimming, square border expansion, and the shared effective command rectangle. Solid and image commands are clipped on the CPU before batching, so the renderer no longer needs per-primitive scissor calls for ordinary quad/image clipping. Rounded quads and borders produce larger solid vertex lists but still remain in the normal solid batch path; they do not trigger a software fallback or a separate material. `rhi_wgpu::ui_surface::batching` turns those visible items into a partial-order draw plan: earlier items only constrain later items when their clipped rectangles intersect, independent items share a depth layer, and each layer groups solid geometry, same-resource images, and text into as few draw ops as current materials allow. `rhi_wgpu::ui_surface::pipeline` owns WGSL shader strings, solid/image pipeline creation, shared image samplers, and bind-group layout creation for direct swapchain rendering. `rhi_wgpu::ui_surface::text` owns glyphon buffer preparation, style mapping, glyph atlas lifetime, and render-pass submission for text batches. Keeping these layers outside the presenter file keeps the wgpu presenter focused on resource lifetime, render passes, and surface presentation while making clip and batching parity directly unit-testable.

The Material shader keeps the alpha contract explicit. Solid and image fragment outputs pass through tint-ready helper functions and are premultiplied before the shared UI blend state, then blend directly into the non-sRGB swapchain target. Rounded fill and border shape still comes from CPU geometry today, but `ui_material.wgsl` keeps the rounded-SDF helper as the stable handoff point for future GPU-side rounded primitive work.

Text uses glyphon inside runtime. The editor provides text, frame, color, font size, line height, and style; runtime intersects the command frame, command clip, and surface bounds before preparing glyph buffers for the direct surface pass. Image commands use cached runtime textures keyed by command resource key; when RGBA bytes are present and the command is visible in the full direct draw list, runtime uploads the changed texture before drawing it. The runtime intentionally treats `resource_key` as the texture cache authority, so editor command producers must derive it from the real asset identity or the RGBA content, never only from dimensions. Atlas-backed UI images therefore use the atlas texture identity as `resource_key` and carry their subimage as `atlas_uv: Some(UiSurfaceImageUvRect)`. `atlas_uv: None` keeps the existing full-texture UV path; valid atlas UVs must be finite, normalized, and strictly ordered on both axes. WGPU geometry first computes local clipped-image UVs from the visible command rect, then composes those local values into the atlas rect. Invalid `Some` atlas UVs are skipped instead of silently sampling the wrong pixels. The wgpu presenter tracks the last present that touched each cached image and caps the cache with least-recently-used pruning, which prevents content-derived viewport keys from growing GPU memory without bound during continuous viewport updates. Overlapping solid, image, and text items keep the same visible order as the softbuffer command-stream executor because the batching planner adds depth dependencies only for intersecting rectangles in stable z/index order. Non-overlapping items are incomparable in that partial order and may be reordered within a layer to batch solid geometry, same-resource images, and text. Same-resource image batching is intentionally governed by that partial order rather than by an atlas-specific branch: disjoint images sharing `resource_key = "atlas://editor/icons"` collapse to one image draw in one layer, overlapping same-resource images split into dependency layers, an overlap chain through a middle item preserves painter order, and independent same-resource images around an overlapping solid still share the independent image layer. `gpu_draw_calls` now reports planned GPU batch submissions, while `gpu_visible_commands`, `gpu_visible_draw_items`, `gpu_batch_layers`, and `gpu_batch_dependencies` expose whether command-stream work is actually batching instead of just counting visible commands.

Milestone 5 keeps atlas batching in the normal image path. Retained-host atlas producers resolve generated editor SpriteAtlas manifests into an atlas texture `resource_key`, full atlas dimensions/RGBA, and per-entry `atlas_uv`; runtime still sees an ordinary image command keyed by the atlas texture. Draw-list stats and WGPU upload preparation de-duplicate visible image upload bytes by `resource_key`, so two visible atlas subimages from the same atlas count and upload the shared atlas texture once while touching the cached texture for the repeated subimage. The ideal profile term for image batching is therefore `image_resource_key_count` per independent layer rather than source-image count.

The current implementation intentionally keeps platform support narrow: `RenderNativeSurfaceTarget::Win32` maps to wgpu raw Win32 handles on Windows. Other platforms must use headless/softbuffer fallback until native targets are added to the runtime contract.

Validation should prove both the boundary and the renderer contract:

- `zircon_editor` presenter sources contain no raw `wgpu::` imports and no concrete `rhi_wgpu` provider names.
- `HostPresenterBackend::default_native()` selects GPU and falls back explicitly to softbuffer.
- `cargo test -p zircon_runtime --lib ui_surface --locked` covers draw-list stats, descriptor validation, headless presenter stats, clip geometry trimming, damage-aware patch stats, retained-cache damage mode selection, border geometry expansion, image UV clipping, partial-order batch layering, same-resource image grouping, same-resource image split/chain cases, text batching, text bounds, source guards for the direct-surface/no-blit path, and GPU upload stats.
- `cargo test -p zircon_runtime --lib ui_surface --locked` also covers rounded quad/border solid vertex generation and rounded solid batching.
- `cargo test -p zircon_editor --lib gpu_presenter --locked` covers RHI failure propagation, upload/draw counters, damage diagnostics, first-frame full redraw for cache bootstrap, patch submission after cache readiness, resize-driven cache invalidation, and corner-radius transfer from chrome commands to runtime surface commands.
- Startup profile `20260515-013306-startup` confirms the native editor session stayed on GPU for startup chrome with `software_fallback_present_count=0`, `gpu_draw_calls=243`, `gpu_upload_bytes=18288`, and no UI hotspot alerts.
- The 2026-05-15 auto-close sweep from `20260515-033851-startup` through `20260515-033926-asset_refresh` kept `software_fallback_present_count=0` and UI hotspot alerts at zero in every profile. Startup recorded `gpu_draw_calls=243` and `gpu_upload_bytes=18288`; `20260515-033857-idle_hover` also recorded two patch presents with `chrome_command_patch_count=2`, `gpu_draw_calls=144`, `chrome_snapshot_count=0`, `workbench_model_build_count=0`, and `presentation_rebuild_count=0`. Other non-startup scenarios in the sweep are useful as no-fallback/no-alert smoke captures unless the capture synthesizes real interaction input.
- The M5 follow-up sweep from `20260515-055615-startup` through `20260515-055704-asset_refresh` revalidated the profiling build after the demo-front `.zui` changes. Startup recorded `software_fallback_present_count=0`, `gpu_draw_calls=366`, `gpu_upload_bytes=32968`, and zero UI hotspot alerts. The auto-close non-startup captures also had zero alerts, but they did not synthesize real pointer or viewport-image interaction patches, so they are recorded as smoke evidence rather than a replacement for manual interaction capture. Their first-fix CPU candidates point at `load_component_showcase_templates`, which belongs to the active component-showcase startup plan rather than the runtime GPU surface.
- Workspace build validation with `cargo build --workspace --locked --verbose` passed after the M5 profile sweep. Workspace test validation is currently blocked before it can serve as GPU-surface acceptance: `cargo test --workspace --locked --verbose` failed in `zircon_editor`, and the narrowed `cargo test -p zircon_editor --lib --locked --message-format=short` showed stale template/demo-front assertions and missing pane body assets/bindings, including `template.ui.host_window` versus `template.v2.ui.host_window`, `runtime_diagnostics_body.ui.toml`, and `PerformanceTimelinePaneBody/RefreshSnapshot`. These failures are owned by the active demo/component-showcase UI work and did not implicate `ChromeCommandStream`, `GpuChromePresenter`, `UiSurfacePresenter`, or `WgpuUiSurfacePresenter`.
- Closeout focused GPU validation re-ran the command-stream/RHI surface tests with `--locked` and `--jobs 1`: runtime `ui_surface` passed 26/26, editor `command_stream` passed 6/6, editor `gpu_presenter` passed 2/2, and editor `render_framework_boundary` passed 3/3. The focused result confirms the workspace blocker remains outside this runtime UI surface path.
- The follow-up boundary convergence moved the native presenter factory behind `zircon_runtime::rhi::create_default_ui_surface_presenter(...)`; the exact runtime/editor boundary guards passed, then full `zircon_runtime --lib`, `zircon_editor --lib`, `zircon_runtime_interface --lib`, and `zircon_app --lib` all passed in `D:\cargo-targets\zircon-shared\demo-front-zui`. Full workspace test remains too large for the current Windows debug/PDB path in this session: one run surfaced the heavy `runtime_ui_text_render_contract` link target, which passed when rerun directly, and the final full workspace attempt timed out after 30 minutes without leaving cargo/rustc/link processes alive.
- The 2026-05-15 poset batching validation passed focused gates in `D:\cargo-targets\zircon-shared\ui-poset-batching`: runtime/editor/runtime-interface checks, runtime `ui_surface` and `ui_hotspot` tests, editor `command_stream`, `gpu_presenter`, and `render_framework_boundary` tests, `zircon_app --features target-editor-host` check, runtime-interface lib tests, and targeted `rustfmt --check`. The profiling smoke run `20260515-201453-startup` through `20260515-201507-viewport_image` recorded zero UI hotspot alerts; startup reduced 250 visible draw items to 37 GPU draw calls across 21 batch layers with 2143 depth dependencies and no software fallback.
- The hover evidence closeout made the automated `idle_hover` profile require both a real redraw and a GPU batch. Capture `20260515-211644-idle_hover` passed that gate with `redraw_region_count=1`, `gpu_draw_calls=10`, `gpu_visible_commands=12`, `gpu_visible_draw_items=12`, `gpu_batch_layers=9`, `gpu_batch_dependencies=60`, zero hotspot alerts, and no software fallback. The companion `20260515-205945-click` profile remains the stronger patch batching sample: `redraw_region_count=3`, `gpu_draw_calls=87`, `gpu_visible_commands=504`, `gpu_visible_draw_items=504`, `gpu_batch_layers=63`, `gpu_batch_dependencies=4758`, zero alerts, and no software fallback.
- The 2026-05-16 Material visual live capture re-ran the current profiling editor after rounded Material command propagation landed. Startup profile `20260516-000244-startup` passed the GPU evidence gate with `gpu_draw_calls=35`, `gpu_visible_commands=188`, `gpu_visible_draw_items=188`, `gpu_batch_layers=20`, `gpu_batch_dependencies=1490`, `gpu_upload_bytes=32968`, and zero UI hotspot alerts. `20260516-000343-click` passed the interaction gate with `redraw_region_count=3`, `gpu_draw_calls=84`, `gpu_visible_commands=318`, `gpu_visible_draw_items=318`, `gpu_batch_layers=60`, `gpu_batch_dependencies=2799`, `gpu_upload_bytes=63840`, and zero alerts. `20260516-000253-idle_hover` recorded `redraw_region_count=1`, `gpu_draw_calls=9`, and zero alerts, but missed the stricter batch gate because that tiny hover patch had one visible draw item per draw call.
- The 2026-05-16 viewport closeout fixed the lazy render-framework startup edge behind `viewport_image`: if the first retained-host tick only starts the async viewport backend resolver, the host now leaves `render_dirty` set and queues a non-reentrant frame-update redraw for the viewport region. That gives the event loop a bounded way to retry extract submission once the backend is ready, without invoking the frame callback while `RetainedEditorHost` is already borrowed. The strict profile `20260516-000208-viewport_image` then recorded `dirty_paint_only_count=1`, `redraw_region_count=1`, `gpu_upload_bytes=1306792`, `gpu_draw_calls=16`, `gpu_visible_draw_items=21`, `gpu_batch_layers=15`, `gpu_batch_dependencies=177`, zero alerts, and no software fallback.
- `tools/ui-profile-capture.ps1` now starts `idle_hover` and `viewport_image` in a temporary `renderable-empty` project so those scenarios exercise a real workbench and viewport instead of the default component-showcase smoke page. The fresh startup/hover/click captures were `20260516-001744-startup` (`188` visible draw items to `35` GPU draws), `20260516-001914-idle_hover` (`37` visible draw items to `32` GPU draws), and `20260516-002011-click` (`318` visible draw items to `84` GPU draws), all with zero hotspot alerts and no software fallback.
- The UI batching interaction gate now derives explicit profile metrics from runtime counters rather than relying on command-count inspection alone. `ui_batch_metrics.json` reports `batch_success_rate = 1 - gpu_draw_calls / gpu_visible_draw_items`, `draw_reduction_ratio = gpu_visible_draw_items / gpu_draw_calls`, `dependency_density = gpu_batch_dependencies / (n * (n - 1) / 2)`, and `layer_density = gpu_visible_draw_items / gpu_batch_layers`. It also records the partial-order model, list-row batching expectation, ideal case, worst-case degeneration, and rectangular clip/mask boundary. The ideal case remains one solid draw plus one text draw plus one image draw per distinct resource key per independent layer; the worst case is overlapping or material-incompatible items degenerating toward one draw per visible item. The strict script gate requires `gpu_draw_calls < gpu_visible_draw_items`, no normal-run softbuffer presents, and no `ui_hotspots` alerts.
- Retained-host visual parity artifacts stay editor-owned while using the runtime stats as the acceptance source. `ui_profile_geometry.json` records the live frames that generated a `UiSurfaceDrawList`, `ui_hit_consistency.json` checks rendered-frame containment against shared retained-host/surface hit routes, `ui_interaction_evidence.json` records geometry-derived pointer paths and drawer-resize layout deltas, and `screenshot_diff.json` compares the software reference with live GPU plus optional profiling-only forced-softbuffer screenshots, including direct GPU-vs-softbuffer when requested. The diff artifact records the configured parity thresholds and the strict gate fails when direct GPU-vs-softbuffer exceeds the differing-sample or average-channel-delta limit. These artifacts do not add fields to `UiSurfaceDrawList`, `UiSurfacePresenter`, or `ChromeCommandStream`; they only make the existing runtime batch, damage, hit, and visual-parity counters auditable from profile sessions.
- Mask/clip acceptance remains rectangular. `rhi_wgpu::ui_surface::geometry` CPU-clips solid/image vertices and UVs to the effective command, surface, and damage rectangles, while text uses glyphon bounds for the same rectangular scope. The current batching conclusion therefore applies to rectangular clips only. A future non-rectangular mask or stencil feature must become an explicit batch key/layer or declared fallback path, because silently adding it inside the existing command stream would invalidate the partial-order batching evidence.
- The 2026-05-16 live interaction closeout separated batch evidence from interaction and visual-parity evidence. WSL planner tests in `target/wsl-ui-validation` proved both the list-row ideal (`6` visible items to `2` draws, `2` layers, `3` dependencies) and all-overlap worst case (`4` visible items to `4` draws, `4` layers, `6` dependencies). Windows profile `20260516-213412-drag` passed the strict batch gate with `164` visible draw items to `39` GPU draw calls, `batch_success_rate=0.762`, `draw_reduction_ratio=4.205`, `dependency_density=0.098`, `layer_density=9.647`, `hit_consistency_samples=93 failed=0`, and no software fallback. `20260516-214006-viewport_image` passed the metrics gate with `21` visible draw items to `16` GPU draw calls and no fallback. `20260516-213637-drawer_resize` proved live splitter dragging by moving the left drawer 80px and keeping hit samples at `87 failed=0`, but that scenario emitted `0` visible GPU draw items and therefore cannot prove batching. `20260516-213543-asset_refresh` recorded `asset_refresh_change_count=5` and `93 failed=0` hit samples, but also emitted `0` visible GPU draw items. Screenshot parity is still open: startup direct GPU-vs-softbuffer failed because the captured GPU image was nearly white and the forced-softbuffer image was only 16x16, while viewport-image parity used normal 1280x720 captures but still failed direct GPU-vs-softbuffer with `differing_sample_ratio=0.9998` and `average_channel_delta=63.05`.
- The 2026-05-17 parity closeout moved the WGPU retained UI target and preferred swapchain formats to non-sRGB byte space, changed new-target clears to opaque black, and prefers opaque swapchain alpha. The pre-fix viewport-image parity capture `20260517-182730-viewport_image` reproduced the bright-GPU failure with direct GPU-vs-softbuffer `differing_sample_ratio=0.6522`; after the non-sRGB fix, `20260517-190736-viewport_image` recorded direct GPU-vs-softbuffer `differing_sample_ratio=0.0165` and `average_channel_delta=0.9022`, under the configured `0.25` / `10.0` thresholds, while keeping `viewport_image` at `21` visible draw items to `16` GPU draw calls and `93` hit samples with `0` failures. The same rebuilt profiling binaries re-ran the interaction gates: `20260517-190840-drawer_resize` moved the live left splitter by `80px`, refreshed geometry, kept `87` hit samples with `0` failures, and recorded `653` visible draw items to `124` GPU draw calls; `20260517-190851-asset_refresh` recorded `266` visible draw items to `42` GPU draw calls and `114` hit samples with `0` failures. These runs close the screenshot color-space and scenario-attribution blockers for the requested evidence path, but they are not a full zero-alert UI-hotspot acceptance because `click`, `drawer_resize`, `idle_hover`, and `startup` still emit follow-up alerts in the profile summaries.
- The 2026-05-18 runtime UI graph and retained-cache damage closeout connected `runtime-ui` to the product render graph through the built-in `ui.screen-space` executor and restored direct-surface damage with `WgpuRetainedSurfaceCache`. The 2026-05-19 revalidation passed `cargo test -p zircon_runtime --locked render_product_ui --jobs 1 --message-format short --color never` (2 passed), `cargo test -p zircon_runtime --locked runtime_ui --jobs 1 --message-format short --color never` (23 runtime lib tests plus 6 text-contract tests), `cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format short --color never` (50 passed), `cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format short --color never` (5 passed), `cargo test -p zircon_runtime --lib material_button_style --locked --jobs 1 --message-format short --color never` (4 passed), and `cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1 --message-format short --color never` (6 passed). The app-side linked advanced render fixture was also revalidated after registering a minimal provider: `cargo test -p zircon_app --lib linked_runtime_render_feature_descriptors_rebuild_default_pipelines --locked --jobs 1 --message-format short --color never` (1 passed) and `cargo test -p zircon_app --lib --locked --jobs 1 --message-format short --color never` (70 passed). The later `.zmaterial` source-rewrite blocker found during workspace expansion was fixed in the material asset serialization layer, after which `cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1` passed `1634` tests and `cargo test --workspace --locked --jobs 1 --message-format short --color never -- --test-threads=1` passed. All of these Windows revalidation commands used `CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage`. The native painter guard now samples the FAB shadow on a non-black test background so half-transparent shadow output is distinguishable from the default black frame.
- The 2026-05-18 SpriteAtlas M1 planner evidence added focused same-resource image partial-order tests for the future editor-generated atlas path without changing production planner code. In `D:\cargo-targets\zircon-shared\sprite-atlas-ui`, `cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed, then `cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format short --color never` passed with `47` UI-surface-filtered tests. The new cases prove three disjoint images with `resource_key = "atlas://editor/icons"` collapse to one image draw and 18 vertices, overlapping images with that same key split into two dependency layers, an image-solid-image overlap chain preserves painter order across three layers, and independent same-resource images around an overlapping solid share the first image layer with 12 vertices.
- The SpriteAtlas M4 retained UI payload slice adds optional atlas UV metadata to the RHI image DTO and forwards the matching editor chrome image metadata into runtime. Planned focused validation for this slice is `cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1`, `cargo test -p zircon_editor --lib command_stream --locked --jobs 1`, and `cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1` in `D:\cargo-targets\zircon-shared\sprite-atlas-ui`. The numeric geometry coverage pins full-image clipping at `[0.25, 0.25]` to `[0.75, 0.75]` and atlas UV `[0.5, 0.25]` to `[0.75, 0.5]` to `[0.5625, 0.3125]` through `[0.6875, 0.4375]` after the same clip.
- The SpriteAtlas M5 atlas-backed UI batching slice adds focused coverage for upload-byte de-duplication, same-key/different-UV runtime batching, retained-host same-atlas/different-atlas command streams, atlas texture payload conversion, resolver lookup through generated project-library atlas TOML/PNG artifacts, software replay atlas sampling from embedded atlas bytes, and headless WGPU stats proving same-atlas subimages reduce to one image draw/upload. Focused validation ran on 2026-05-22 in `D:\cargo-targets\zircon-shared\sprite-atlas-ui`: `cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format short --color never` passed `53` tests after one warm-up compile timeout; `cargo test -p zircon_editor --lib command_stream --locked --jobs 1 --message-format short --color never` passed `11` tests after fixing atlas recorded payload conversion and after the test-only split; `cargo test -p zircon_editor --lib sprite_atlas --locked --jobs 1 --message-format short --color never` passed `12` tests; `cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format short --color never` passed `5` tests; `cargo check -p zircon_runtime --lib --locked --message-format short --color never`, `cargo check -p zircon_editor --lib --locked --message-format short --color never`, and `cargo check -p zircon_app --features target-editor-host --locked --message-format short --color never` all finished successfully with warnings. No live atlas-heavy profile artifact was produced because the workspace currently has no `editor-sprite-atlases` project-library artifacts for a retained-host scenario.
