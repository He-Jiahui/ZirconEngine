---
related_code:
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi_wgpu/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs
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
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - tools/ui-profile-capture.ps1
plan_sources:
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - user: 2026-05-15 GPU command stream should take over editor UI rendering
  - user: 2026-05-15 UI rendering needs poset depth batching for retained editor UI
  - .codex/plans/Retained Host Chrome GPU 化与 Hover 卡顿根因修复计划.md
  - .codex/plans/Retained Host Chrome 性能根因修复计划.md
  - .codex/plans/Editor 基础组件 Material 化视觉优化计划.md
  - user: 2026-05-15 Material-like retained editor UI needs real rounded controls across software and GPU presenters
tests:
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - cargo check -p zircon_runtime --lib --locked
  - cargo check -p zircon_editor --lib --locked
  - cargo test -p zircon_runtime --lib ui_surface --locked
  - cargo test -p zircon_runtime --lib draw_list_stats_skip_commands_outside_damage --locked
  - cargo test -p zircon_runtime --lib draw_list_stats_do_not_count_cached_images_as_uploads --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_text_bounds_clip_to_damage_and_command_clip --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_text_skips_disjoint_damage --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_draw_ops_preserve_mixed_command_order --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_draw_ops_sort_by_stable_z_order --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_text_ops_interleave_with_geometry_z_order --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_presenter_stats_skip_disjoint_patch_commands --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_image_cache_prune --locked
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

- Native descriptors create a real wgpu instance, adapter, device, queue, Win32 surface, swapchain configuration, retained offscreen UI texture, solid/image pipelines, glyphon text renderer, and final blit pipeline.
- Headless descriptors keep the same contract and stats without creating a native surface. Unit tests use this mode so CI does not require a window.

The native renderer preserves damage patches with an offscreen retained UI target. A full draw list clears and rebuilds that texture. A patch draw list loads the previous offscreen texture, clips command geometry to the damage/clip intersection, renders the changed commands, then blits the complete retained texture to the acquired swapchain image. This keeps GPU present complete while avoiding normal-path full-frame CPU UI painting.

`rhi_wgpu::ui_surface::geometry` owns the draw-list geometry layer: stable `(z_index, command_index, sub_index)` ordering, quad/image vertices, rounded solid tessellation, rounded border rings, image UV trimming, square border expansion, and the shared effective command rectangle. Solid and image commands are clipped on the CPU before batching, so the renderer no longer needs per-primitive scissor calls for ordinary quad/image clipping. Rounded quads and borders produce larger solid vertex lists but still remain in the normal solid batch path; they do not trigger a software fallback or a separate material. `rhi_wgpu::ui_surface::batching` turns those visible items into a partial-order draw plan: earlier items only constrain later items when their clipped rectangles intersect, independent items share a depth layer, and each layer groups solid geometry, same-resource images, and text into as few draw ops as current materials allow. `rhi_wgpu::ui_surface::pipeline` owns WGSL shader strings, solid/image pipeline creation, shared image samplers, bind-group layout creation, and final offscreen-to-surface blit resources. `rhi_wgpu::ui_surface::text` owns glyphon buffer preparation, style mapping, glyph atlas lifetime, and render-pass submission for text batches. Keeping these layers outside the presenter file keeps the wgpu presenter focused on resource lifetime, render passes, and surface presentation while making damage/clip and batching parity directly unit-testable.

Text uses glyphon inside runtime. The editor provides text, frame, color, font size, line height, and style; runtime intersects the command frame, command clip, surface bounds, and optional draw-list damage before preparing glyph buffers. Disjoint text commands are skipped, and glyphon `TextBounds` comes from the same effective rectangle used by quad/image clipping, so patch presents cannot leak text outside the damaged region. Image commands use cached runtime textures keyed by command resource key; when RGBA bytes are present and the command intersects the current damage, runtime uploads the changed texture before drawing it. The runtime intentionally treats `resource_key` as the texture cache authority, so editor command producers must derive it from the real asset identity or the RGBA content, never only from dimensions. The wgpu presenter tracks the last present that touched each cached image and caps the cache with least-recently-used pruning, which prevents content-derived viewport keys from growing GPU memory without bound during continuous viewport updates. Overlapping solid, image, and text items keep the same visible order as the softbuffer command-stream executor because the batching planner adds depth dependencies only for intersecting rectangles in stable z/index order. Non-overlapping items are incomparable in that partial order and may be reordered within a layer to batch solid geometry, same-resource images, and text. `gpu_draw_calls` now reports planned GPU batch submissions, while `gpu_visible_commands`, `gpu_visible_draw_items`, `gpu_batch_layers`, and `gpu_batch_dependencies` expose whether command-stream work is actually batching instead of just counting visible commands.

The current implementation intentionally keeps platform support narrow: `RenderNativeSurfaceTarget::Win32` maps to wgpu raw Win32 handles on Windows. Other platforms must use headless/softbuffer fallback until native targets are added to the runtime contract.

Validation should prove both the boundary and the renderer contract:

- `zircon_editor` presenter sources contain no raw `wgpu::` imports and no concrete `rhi_wgpu` provider names.
- `HostPresenterBackend::default_native()` selects GPU and falls back explicitly to softbuffer.
- `cargo test -p zircon_runtime --lib ui_surface --locked` covers draw-list stats, descriptor validation, headless presenter stats, damage/clip geometry trimming, border geometry expansion, image UV clipping, partial-order batch layering, same-resource image grouping, text batching, text damage/clip bounds, and damage-filtered GPU upload stats.
- `cargo test -p zircon_runtime --lib ui_surface --locked` also covers rounded quad/border solid vertex generation and rounded solid batching.
- `cargo test -p zircon_editor --lib gpu_presenter --locked` covers RHI failure propagation, upload/draw counters, damage diagnostics, and corner-radius transfer from chrome commands to runtime surface commands.
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
