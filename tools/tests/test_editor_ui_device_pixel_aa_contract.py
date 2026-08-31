import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


def source(relative_path: str) -> str:
    return (REPO_ROOT / relative_path).read_text(encoding="utf-8")


class EditorUiDevicePixelAaContractTests(unittest.TestCase):
    def test_product_semantic_glyphs_do_not_fall_back_to_manual_primitives(self):
        glyph_root = REPO_ROOT / (
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes"
        )
        allowed_analytic_glyphs = {
            Path("template_status_glyphs/signals/base.rs"),
        }
        direct_primitive_files = {
            path.relative_to(glyph_root)
            for glyph_family in glyph_root.glob("template_*_glyphs")
            for path in glyph_family.rglob("*.rs")
            if "HostPaintCommand::" in path.read_text(encoding="utf-8")
        }

        self.assertEqual(direct_primitive_files, allowed_analytic_glyphs)

        svg_backed_glyphs = (
            "template_alert_glyphs/marks.rs",
            "template_selection_controls/checkbox/tick.rs",
            "template_tree_row_glyphs/disclosure.rs",
            "template_tooltip_glyphs/arrows.rs",
        )
        for relative_path in svg_backed_glyphs:
            glyph = source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                f"paint_template_nodes/{relative_path}"
            )
            self.assertIn("push_icon_asset_pixels", glyph, relative_path)

    def test_native_rhi_surface_uses_the_physical_window_extent(self):
        rhi = source("zircon_runtime/crates/zr_rhi/src/ui_surface.rs")
        self.assertIn("let size = window.surface_size();", rhi)
        self.assertIn("size.width.max(1)", rhi)
        self.assertIn("size.height.max(1)", rhi)

    def test_editor_present_chain_preserves_the_physical_client_extent(self):
        window_lifecycle = source(
            "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/"
            "lifecycle.rs"
        )
        gpu_present = source(
            "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/"
            "present.rs"
        )
        draw_list = source(
            "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
            "runtime_draw_list.rs"
        )
        surface_setup = source(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/surface_setup.rs"
        )

        self.assertIn("let size = window.surface_size();", window_lifecycle)
        self.assertIn(
            "state.window_size = PhysicalSize::new(size.width, size.height);",
            window_lifecycle,
        )
        self.assertIn("presentation,\n            self.size,", gpu_present)
        self.assertIn("let surface_size = stream.surface_size();", draw_list)
        self.assertIn("width: size.0.max(1)", surface_setup)
        self.assertIn("height: size.1.max(1)", surface_setup)

    def test_resize_snapshot_cannot_become_the_ordinary_surface_baseline(self):
        gpu_present = source(
            "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/"
            "present.rs"
        )
        ordinary_present = gpu_present.split(
            "pub(in crate::ui::retained_host::host_contract) fn present(", 1
        )[1].split(
            "pub(in crate::ui::retained_host::host_contract) fn "
            "present_during_native_resize(",
            1,
        )[0]
        resize_present = gpu_present.split(
            "pub(in crate::ui::retained_host::host_contract) fn "
            "present_during_native_resize(",
            1,
        )[1].split(
            "pub(in crate::ui::retained_host::host_contract) fn present_stream(",
            1,
        )[0]

        self.assertIn("self.native_resize_draw_list = None;", ordinary_present)
        self.assertIn("self.native_resize_projection_size = self.size;", ordinary_present)
        self.assertIn(
            "damage.as_ref().filter(|_| self.surface_cache_initialized)",
            ordinary_present,
        )
        self.assertIn("retarget_surface_size_preserving_projection", resize_present)
        self.assertIn("self.surface_cache_initialized = false;", resize_present)

    def test_ordinary_draw_lists_cannot_drop_below_the_physical_target_extent(self):
        rhi = source("zircon_runtime/crates/zr_rhi/src/ui_surface.rs")
        constructor_block = rhi.split("impl UiSurfaceDrawList {", 1)[1].split(
            "pub fn style_count", 1
        )[0]

        self.assertIn(
            "let surface_size = (surface_size.0.max(1), surface_size.1.max(1));",
            constructor_block,
        )
        self.assertEqual(constructor_block.count("projection_size: surface_size"), 3)
        self.assertIn("target_only_resize: false", constructor_block)

    def test_native_resize_projection_is_at_least_the_current_physical_target(self):
        lifecycle = source(
            "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/lifecycle.rs"
        )
        present = source(
            "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/present.rs"
        )

        self.assertIn(".max(self.size.0)", lifecycle)
        self.assertIn(".max(size.0)", lifecycle)
        self.assertIn(".max(self.size.1)", lifecycle)
        self.assertIn(".max(size.1)", lifecycle)
        resize_present = present.split(
            "pub(in crate::ui::retained_host::host_contract) fn "
            "present_during_native_resize(",
            1,
        )[1]
        self.assertIn("self.native_resize_projection_size", resize_present)
        self.assertIn("retarget_surface_size_preserving_projection", resize_present)

    def test_resolution_dependent_ui_resources_never_drop_below_physical_scale(self):
        extract = source("zircon_runtime_interface/src/ui/surface/render/extract.rs")
        frame_extract = source(
            "zircon_runtime_interface/src/ui/surface/render/frame_extract.rs"
        )
        surface_rebuild = source(
            "zircon_runtime/src/ui/surface/surface/rebuild.rs"
        )
        runtime_render = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
        )
        runtime_plan_cache = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs"
        )
        icon_atlas = source("zircon_runtime/src/ui/icon_atlas/atlas.rs")
        normalized = extract.split("pub fn normalized_raster_scale", 1)[1].split(
            "const fn default_raster_scale", 1
        )[0]

        self.assertIn("self.raster_scale.max(1.0)", normalized)
        self.assertIn("rasterized below the physical target", normalized)
        frame_normalized = frame_extract.split(
            "pub fn normalized_raster_scale", 1
        )[1].split("pub fn command_range", 1)[0]
        self.assertIn("self.raster_scale.max(1.0)", frame_normalized)
        self.assertIn("metrics.physical_size.width", surface_rebuild)
        self.assertIn("metrics.logical_size.width", surface_rebuild)
        self.assertIn("metrics.physical_size.height", surface_rebuild)
        self.assertIn("metrics.logical_size.height", surface_rebuild)
        self.assertIn("reported_scale.max(physical_scale)", surface_rebuild)
        self.assertIn("extract.normalized_raster_scale(),", runtime_render)
        self.assertIn("let raster_scale = raster_scale.max(1.0);", runtime_render)
        self.assertIn(
            "segment.extract().normalized_raster_scale(),",
            runtime_plan_cache,
        )
        self.assertIn("dpi_scale.max(1.0)", icon_atlas)
        self.assertIn("below-native raster", icon_atlas)

    def test_per_monitor_dpi_change_ends_in_a_full_physical_reflow(self):
        resize_events = source(
            "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/"
            "events/resize.rs"
        )
        redraw = source(
            "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/"
            "redraw.rs"
        )

        scale_handler = resize_events.split(
            "pub(super) fn handle_window_scale_factor_changed(", 1
        )[1].split("fn queue_resize_frame(", 1)[0]
        self.assertIn("self.host.window().set_scale_factor(scale_factor);", scale_handler)
        self.assertIn("self.queue_resize_frame(None);", scale_handler)

        surface_handler = resize_events.split(
            "pub(super) fn handle_surface_resized(", 1
        )[1].split(
            "pub(super) fn handle_window_scale_factor_changed(", 1
        )[0]
        self.assertIn("metrics.physical_size.width", surface_handler)
        self.assertIn("metrics.physical_size.height", surface_handler)
        self.assertIn(
            ".set_scale_factor(metrics.scale_factor as f32);",
            surface_handler,
        )
        self.assertIn("self.host.window().set_size(physical_size.clone());", surface_handler)

        queue = resize_events.split("fn queue_resize_frame(", 1)[1].split(
            "pub(super) fn handle_window_moved(", 1
        )[0]
        self.assertIn("self.pending_presenter_resize = Some", queue)
        self.assertIn(
            "HostRedrawRequest::full_frame_for_scenario(UiPerfScenario::WindowResize, true)",
            queue,
        )
        self.assertIn(".into_interactive_frame_update()", queue)

        redraw_impl = redraw.split(
            "pub(in crate::ui::retained_host::host_contract) fn "
            "redraw_requested_impl(",
            1,
        )[1]
        resize = redraw_impl.find("self.apply_pending_presenter_resize(event_loop)")
        frame_update = redraw_impl.find("if redraw.requires_frame_update()")
        present = redraw_impl.find(
            "present_redraw(self, event_loop, redraw, present_scenario)"
        )
        self.assertGreaterEqual(resize, 0)
        self.assertGreaterEqual(frame_update, 0)
        self.assertGreaterEqual(present, 0)
        self.assertLess(resize, frame_update)
        self.assertLess(frame_update, present)
        self.assertNotIn("schedule_due_resize_reflow", redraw_impl)

    def test_workbench_projection_crosses_the_logical_physical_boundary_once(
        self,
    ):
        bridge = source(
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/bridge.rs"
        )
        mount = source(
            "zircon_editor/src/ui/retained_host/ui/workbench_window_projection/mount.rs"
        )
        self.assertIn(
            "physical_shell_size.width / self.presentation_scale_factor", bridge
        )
        self.assertIn("scale_node_metrics(&mut node, scale_factor);", mount)
        for metric in (
            "&mut node.frame",
            "&mut node.font_size",
            "&mut node.corner_radius",
            "&mut node.border_width",
        ):
            self.assertIn(metric, mount)

    def test_wgpu_rounded_edges_use_analytic_pixel_coverage(self):
        shader = source(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shaders/"
            "ui_material.wgsl"
        )
        runtime_shader = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/"
            "screen_space_ui.wgsl"
        )
        runtime_geometry = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/geometry.rs"
        )
        runtime_renderer = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
        )
        geometry = source(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/geometry.rs"
        )
        batching = source(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs"
        )
        self.assertIn("fn rounded_box_distance", shader)
        self.assertIn("fn rounded_box_coverage", shader)
        self.assertIn("let sample_offsets = array<vec2<f32>, 16>", shader)
        self.assertIn("max(fwidth(local_position.x), 0.0001)", shader)
        self.assertIn("max(fwidth(local_position.y), 0.0001)", shader)
        self.assertIn(
            "local_position + sample_offsets[sample_index] * pixel_step", shader
        )
        self.assertIn("let subpixel_filter_scale = 0.25", shader)
        self.assertIn("distance_width * subpixel_filter_scale", shader)
        self.assertIn("let coverage_guard = distance_width * 0.75", shader)
        self.assertIn("let inner_coverage_guard = inner_distance_width * 0.75", shader)
        self.assertIn("if outer_distance <= coverage_guard * -1.0", shader)
        self.assertIn("if inner_distance <= inner_coverage_guard * -1.0", shader)
        self.assertIn("let sample_outer_distance", shader)
        self.assertNotIn("fwidth(sample_outer_distance)", shader)
        self.assertIn("outer_coverage_sum += outer_coverage", shader)
        self.assertIn("inner_coverage_sum += inner_coverage", shader)
        self.assertIn(
            "return vec2<f32>(outer_coverage_sum, inner_coverage_sum) * 0.0625",
            shader,
        )
        self.assertIn("fwidth(outer_distance)", shader)
        self.assertIn(
            "1.0 - smoothstep(\n        distance_width * -0.5,\n        distance_width * 0.5,",
            shader,
        )
        self.assertIn("fwidth(inner_distance)", shader)
        self.assertIn(
            "rounded_box_alpha(\n                sample_inner_distance,\n                inner_distance_width * subpixel_filter_scale,\n            )",
            shader,
        )
        self.assertIn("max(coverages.x - coverages.y, 0.0)", shader)
        self.assertIn("@location(6) fill_color: vec4<f32>", shader)
        self.assertIn("let inner_coverage = min(coverages.y, coverages.x)", shader)
        self.assertIn("coverages.x - inner_coverage", shader)
        self.assertIn("return fill + border", shader)
        self.assertIn("local_position: local_positions[index]", geometry)
        self.assertIn("corner_radius,", geometry)
        self.assertIn("fused_border_command_index: Some", geometry)
        self.assertIn("fused_border_commands: Vec<bool>", batching)
        self.assertIn(".fused_border_commands\n                .get(command_index)", batching)
        self.assertIn("stats.record_draw_item_fusion()", batching)

        for contract in (
            "let sample_offsets = array<vec2<f32>, 16>",
            "max(fwidth(local_position.x), 0.0001)",
            "max(fwidth(local_position.y), 0.0001)",
            "local_position + sample_offsets[sample_index] * pixel_step",
            "let subpixel_filter_scale = 0.25",
            "let coverage_guard = distance_width * 0.75",
            "let inner_coverage_guard = inner_distance_width * 0.75",
            "return vec2<f32>(outer_coverage_sum, inner_coverage_sum) * 0.0625",
        ):
            self.assertIn(contract, runtime_shader)
        self.assertNotIn("fwidth(sample_outer_distance)", runtime_shader)
        self.assertIn("pub(super) fill_color: [f32; 4]", runtime_geometry)
        self.assertIn("6 => Float32x4", runtime_geometry)
        self.assertIn("pub(super) fn push_rounded_box(", runtime_geometry)
        self.assertIn("vertex.fill_color = fill_color", runtime_geometry)
        self.assertIn("push_rounded_box(", runtime_renderer)
        self.assertIn("@location(6) fill_color: vec4<f32>", runtime_shader)
        self.assertIn(
            "let fill_alpha = input.fill_color.a * inner_coverage", runtime_shader
        )
        self.assertIn(
            "let border_alpha = input.color.a * border_coverage", runtime_shader
        )
        self.assertIn("premultiplied_rgb / alpha", runtime_shader)

    def test_editor_rounded_fill_and_border_surfaces_use_one_coverage_path(self):
        combined_only_surfaces = [
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/docks/pane/fallback/empty_state.rs"
            ),
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/menus/popup.rs"
            ),
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/menus/popup/submenus.rs"
            ),
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/welcome/main_column/form/preview.rs"
            ),
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/welcome/recent_projects/list.rs"
            ),
        ]
        mixed_shape_surfaces = [
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/scene_layers/overlay/dock_overflow.rs"
            ),
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/scene_layers/overlay/page_overflow.rs"
            ),
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/welcome/main_column/hero.rs"
            ),
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/welcome/recent_projects/rows/surface.rs"
            ),
        ]

        for surface in combined_only_surfaces:
            self.assertIn("draw_rounded_box_clipped(", surface)
            self.assertNotIn("draw_rounded_rect_clipped", surface)
            self.assertNotIn("draw_rounded_border_clipped", surface)
        for surface in mixed_shape_surfaces:
            self.assertIn("draw_rounded_box_clipped(", surface)

    def test_retained_menu_uses_scaled_small_control_and_panel_radius_tiers(self):
        metrics = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_theme/"
            "metrics.rs"
        )
        root_popup = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/menus/popup.rs"
        )
        submenus = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/menus/popup/submenus.rs"
        )
        rows = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/menus/rows.rs"
        )
        overflow_surfaces = [
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/scene_layers/overlay/page_overflow.rs"
            ),
            source(
                "zircon_editor/src/ui/retained_host/host_contract/"
                "paint_workbench_renderer/scene_layers/overlay/dock_overflow.rs"
            ),
        ]
        settings_color_controls = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_template_nodes/template_settings_window/color_controls.rs"
        )
        settings_enum_controls = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_template_nodes/template_settings_window/enum_controls.rs"
        )
        settings_commands = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_template_nodes/template_settings_window/commands.rs"
        )
        template_popup_background = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_template_nodes/template_popup_rows/surface/background/style.rs"
        )

        for field in ("radius_small", "radius_control", "radius_panel"):
            self.assertIn(f"pub {field}: f32", metrics)
            self.assertIn(f"{field}: scaled(self.{field})", metrics)
        self.assertIn("radius_small: 6.0", metrics)
        self.assertIn("radius_control: 8.0", metrics)
        self.assertIn("radius_panel: 12.0", metrics)
        self.assertIn("controls.small_radius", metrics)
        self.assertIn("controls.control_radius", metrics)
        self.assertIn("controls.panel_radius", metrics)

        self.assertEqual(root_popup.count("metrics.radius_panel"), 1)
        self.assertEqual(submenus.count("metrics.radius_panel"), 1)
        self.assertIn("draw_rounded_box_clipped(", root_popup)
        self.assertIn("draw_rounded_box_clipped(", submenus)
        self.assertIn("draw_rounded_rect_clipped(", rows)
        self.assertIn("metrics.radius_small", rows)
        hover_block = rows.split("if hovered {", 1)[1].split("}", 1)[0]
        self.assertNotIn("draw_rect_clipped", hover_block)
        for overflow in overflow_surfaces:
            production = overflow.split("#[cfg(test)]", 1)[0]
            self.assertEqual(production.count("metrics.radius_panel"), 1)
            self.assertIn("metrics.radius_small", production)
            self.assertIn("draw_rounded_box_clipped(", production)
            self.assertIn("draw_rounded_rect_clipped(", production)
        self.assertIn("metrics.radius_panel", settings_color_controls)
        self.assertIn("metrics.radius_panel", settings_enum_controls)
        panel_block = settings_commands.split("fn push_panel(", 1)[1].split(
            "fn push_title(", 1
        )[0]
        self.assertIn("metrics.radius_panel", panel_block)
        self.assertNotIn("metrics.radius_control + metrics.gap_s", panel_block)
        self.assertIn("metrics.radius_panel", template_popup_background)
        self.assertNotIn(
            "metrics.radius_control + metrics.gap_s", template_popup_background
        )

    def test_wgpu_square_edges_keep_fractional_coverage_without_penalizing_aligned_fills(
        self,
    ):
        geometry = source(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/geometry.rs"
        )

        self.assertIn("rect_edges_are_physical_pixel_aligned", geometry)
        self.assertIn("!rect_edges_are_physical_pixel_aligned(frame)", geometry)
        self.assertIn("SolidGeometry::Instance(solid_instance", geometry)
        self.assertIn("push_border_item(", geometry)
        self.assertNotIn("fn border_rects(", geometry)
        self.assertNotIn("let width = width.max(1.0)", geometry)

    def test_native_wgpu_readback_distinguishes_fractional_square_border_width(self):
        native_submission = source(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests/"
            "native_submission.rs"
        )

        regression = native_submission.split(
            "fn wgpu_ui_fractional_square_border_readback_preserves_subpixel_width()",
            1,
        )[1].split("#[test]", 1)[0]
        self.assertIn("render(0.625)", regression)
        self.assertIn("render(1.0)", regression)
        self.assertIn("alpha_sum(&thin) < alpha_sum(&one_pixel)", regression)
        self.assertIn("corner_radius: 0.0", regression)

    def test_rounded_border_keeps_fractional_physical_width_for_gpu_coverage(self):
        rounded_border = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_primitives/"
            "shapes/borders/rounded.rs"
        )
        recording_path = rounded_border.split("fn draw_square_border_stack", 1)[0]

        self.assertNotIn("border_width.ceil()", recording_path)
        self.assertIn(
            "let border_width = clamped_border_width(&rect, border_width);",
            recording_path,
        )
        record_call = recording_path.split("if frame.is_recording()", 1)[1]
        self.assertIn("frame.record_border(", record_call)
        self.assertIn("border_width,", record_call)
        self.assertIn("corner_radius,", record_call)
        self.assertNotIn("ceil()", record_call)

    def test_square_border_keeps_fractional_width_for_software_coverage(self):
        command_border = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "render_commands/draw/border.rs"
        )
        pixel_border = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_primitives/"
            "pixels/border.rs"
        )

        self.assertNotIn("border_width.ceil()", command_border)
        self.assertIn("draw_rounded_border_clipped", command_border)
        self.assertIn("fill_rect_border_pixels", pixel_border)
        self.assertIn("rect_pixel_coverage", pixel_border)

    def test_direct_workbench_borders_share_analytic_coverage_authority(self):
        direct_border = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_primitives/"
            "shapes/borders/rect.rs"
        )

        self.assertIn("draw_rounded_border_clipped", direct_border)
        self.assertIn("color, 1.0, 0.0", direct_border)
        self.assertNotIn("draw_rect_clipped", direct_border)
        self.assertNotIn("border_top", direct_border)

    def test_software_command_replay_does_not_requantize_square_borders(self):
        replay_border = source(
            "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
            "replay/commands/shapes/border.rs"
        )
        retired_rect = REPO_ROOT / (
            "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
            "replay/commands/shapes/border/rect.rs"
        )

        self.assertIn("draw_rounded_border_clipped", replay_border)
        self.assertNotIn("paint_rect_border_command", replay_border)
        self.assertNotIn("corner_radius > 0.0", replay_border)
        self.assertFalse(retired_rect.exists())

    def test_square_fill_recording_preserves_fractional_device_geometry(self):
        solid_rect = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_primitives/"
            "shapes/rects/solid.rs"
        )
        recording_path = solid_rect.split("if frame.is_recording()", 1)[1]

        self.assertIn("frame.record_quad(rect.clone()", recording_path)
        self.assertNotIn("target.to_frame()", solid_rect)
        self.assertNotIn("PixelRectExt", solid_rect)

    def test_software_square_fills_resolve_only_fractional_edge_coverage(self):
        solid_rect = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_primitives/"
            "shapes/rects/solid.rs"
        )
        fill = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_primitives/"
            "pixels/fill.rs"
        )

        self.assertIn("fill_rect_pixel_coverage", solid_rect)
        self.assertIn("rect_pixel_coverage", fill)
        self.assertIn("fill_pixel_span", fill)
        self.assertNotIn("CHART_RASTER_SAMPLES_PER_AXIS", fill)

    def test_native_rounded_surfaces_keep_post_dpi_fractional_geometry(self):
        geometry_paths = (
            "material_primitives/badge/geometry/metrics.rs",
            "template_axis_value_fields/geometry.rs",
            "template_command_palette/layout/common.rs",
            "template_dialogs/layout.rs",
            "template_notification_center/layout/common.rs",
            "template_alerts/layout/common.rs",
            "template_tooltips/layout.rs",
            "template_section_titles/geometry.rs",
        )
        root = (
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_template_nodes/"
        )

        for relative_path in geometry_paths:
            geometry = source(root + relative_path)
            for quantizer in (".round()", ".ceil()", ".floor()"):
                self.assertNotIn(quantizer, geometry, relative_path)

        settings_commands = source(root + "template_settings_window/commands.rs")
        section_commands = source(root + "template_section_titles/commands.rs")
        self.assertNotIn("let rect = pixel_aligned(rect)", settings_commands)
        self.assertNotIn("fn pixel_aligned(", settings_commands)
        self.assertNotIn("pixel_aligned_rect", section_commands)

    def test_svg_search_glyph_keeps_fractional_post_dpi_origin(self):
        glyph = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_template_nodes/template_fields/search/glyph.rs"
        )

        search_icon_geometry = glyph.split("fn search_icon_rect", 1)[1]
        self.assertNotIn(".round()", search_icon_geometry)
        self.assertIn("rect.x + metrics.input_pad_left", search_icon_geometry)
        self.assertIn(
            "rect.y + (rect.height - metrics.search_icon_size).max(0.0) * 0.5",
            search_icon_geometry,
        )

    def test_software_shape_blending_uses_the_shared_linear_light_lut(self):
        color = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_color.rs"
        )
        span = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_primitives/"
            "pixels/span.rs"
        )
        coverage_path = span.split("pub(in crate::ui::retained_host::host_contract) fn write_pixel_with_coverage", 1)[1]

        self.assertIn("blend_srgb_pixel_linear", coverage_path)
        self.assertIn("blend_srgb_pixel_linear", span)
        self.assertIn("OnceLock<[f32; 256]>", color)
        self.assertIn("OnceLock<[u8; LINEAR_ENCODE_LUT_MAX + 1]>", color)
        self.assertIn("srgb_byte_to_linear", color)
        self.assertIn("linear_to_srgb_byte", color)
        self.assertNotIn("powf", span)
        self.assertNotIn("covered_color[3]", coverage_path)

    def test_software_scaled_images_sample_and_composite_in_linear_light(self):
        color = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_color.rs"
        )
        image_pixel = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_primitives/"
            "image/raster/pixel.rs"
        )

        self.assertIn("srgb_byte_to_linear", image_pixel)
        self.assertIn("blend_premultiplied_linear_srgb_pixel", image_pixel)
        self.assertIn("premultiplied_linear", image_pixel)
        self.assertIn("LINEAR_ENCODE_LUT_MAX", color)
        self.assertNotIn("OnceLock", image_pixel)
        self.assertNotIn("rgba[source_offset + channel] as f32 *", image_pixel)
        self.assertNotIn("powf", image_pixel)

    def test_software_alpha_consumers_share_wgpu_linear_blend_semantics(self):
        image_pixel = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_primitives/"
            "image/raster/pixel.rs"
        )
        text_blend = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_text/blend.rs"
        )

        self.assertIn("blend_srgb_pixel_linear", image_pixel)
        self.assertIn("blend_srgb_pixel_linear", text_blend)
        self.assertIn("blend_srgb_pixel_linear_channels", text_blend)
        self.assertNotIn("source * alpha + destination * inverse", image_pixel)
        self.assertNotIn("source * alpha + destination * inverse", text_blend)

    def test_viewport_overlay_uses_shared_linear_source_over(self):
        overlay = source(
            "zircon_editor/src/ui/retained_host/host_contract/data/"
            "viewport_image/overlay.rs"
        )
        blend_path = overlay.split("fn blend_source_over", 1)[1].split(
            "fn fnv1a", 1
        )[0]

        self.assertIn("blend_srgb_pixel_linear", blend_path)
        self.assertNotIn("source * source_alpha", blend_path)
        self.assertNotIn("destination_alpha", blend_path)

    def test_wgpu_coverage_and_color_follow_the_surface_transfer_function(self):
        shader = source(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shaders/"
            "ui_material.wgsl"
        )
        pipeline = source(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/pipeline.rs"
        )

        self.assertIn("srgb_to_linear(tinted.rgb)", shader)
        self.assertIn("premultiply_alpha", shader)
        self.assertIn("UiTargetColorMode::LinearSrgb", pipeline)
        self.assertIn('solid: "solid_fs_linear_target"', pipeline)

    def test_runtime_scene_glyph_atlas_uses_padded_bilinear_device_pixel_sampling(self):
        resources = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/"
            "atlas_renderer/resources.rs"
        )
        bitmap_run = source("zircon_runtime/src/text/atlas/bitmap_run.rs")

        self.assertIn(
            "pub(crate) const GLYPH_BITMAP_ATLAS_PADDING_PX: u32 = 2;",
            bitmap_run,
        )
        self.assertIn("mag_filter: wgpu::FilterMode::Linear", resources)
        self.assertIn("min_filter: wgpu::FilterMode::Linear", resources)
        self.assertIn("mipmap_filter: wgpu::MipmapFilterMode::Nearest", resources)
        self.assertIn("lod_min_clamp: 0.0", resources)
        self.assertIn("lod_max_clamp: 0.0", resources)

    def test_editor_wgpu_text_rasterizes_at_physical_metrics_without_post_scale(self):
        text_renderer = source(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/text.rs"
        )

        self.assertIn("width: projection_size.0.max(1)", text_renderer)
        self.assertIn("height: projection_size.1.max(1)", text_renderer)
        self.assertIn(
            "Metrics::new(font_size.max(1.0), line_height.max(1.0))",
            text_renderer,
        )
        self.assertIn("scale: 1.0", text_renderer)
        self.assertIn("UiTargetColorMode::LinearSrgb => ColorMode::Accurate", text_renderer)

    def test_softbuffer_fallback_keeps_physical_extent_and_local_edge_supersampling(
        self,
    ):
        surface_size = source(
            "zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/"
            "surface_io/size.rs"
        )
        geometry = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_primitives/"
            "pixels/geometry.rs"
        )

        self.assertIn("let size = window.surface_size();", surface_size)
        self.assertIn("clamp_size((size.width, size.height))", surface_size)
        self.assertIn("const COVERAGE_SAMPLE_AXIS: u32 = 8;", geometry)
        self.assertIn("for sample_y in 0..COVERAGE_SAMPLE_AXIS", geometry)
        self.assertIn("for sample_x in 0..COVERAGE_SAMPLE_AXIS", geometry)
        self.assertIn("rounded_rect_signed_distance", geometry)

    def test_local_raster_quality_is_bounded_and_target_aware(self):
        vector_target = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "visual_assets/target.rs"
        )
        vector_pixels = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "visual_assets/svg/pixels.rs"
        )
        render_command = source(
            "zircon_runtime_interface/src/ui/surface/render/command.rs"
        )
        image_conversion = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "render_command_conversion/image.rs"
        )
        wgpu_pipeline = source(
            "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/pipeline.rs"
        )
        text_draw = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/"
            "glyphs.rs"
        )
        text_raster = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_text/"
            "raster.rs"
        )
        text_raster_metrics = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_text/"
            "raster/metrics.rs"
        )
        self.assertIn(
            "const VECTOR_SMALL_ICON_SUPERSAMPLE_SCALE: u32 = 4;", vector_target
        )
        self.assertIn("const VECTOR_SUPERSAMPLE_SCALE: u32 = 2;", vector_target)
        self.assertIn(
            "const VECTOR_SMALL_ICON_SUPERSAMPLE_MAX_EDGE: u32 = 32;",
            vector_target,
        )
        self.assertIn("let supersample_scale = self.vector_supersample_scale();", vector_target)
        self.assertIn("checked_mul(supersample_scale)", vector_target)
        self.assertIn(
            "const MAX_VECTOR_RASTER_EDGE_VALUE: u32 = 4096;", vector_target
        )
        self.assertIn("center_rgba_in_target", vector_pixels)
        self.assertIn("downsample_rgba(", vector_pixels)
        self.assertIn("supersample_scale,", vector_pixels)
        self.assertNotIn("downsample_rgba_2x", vector_pixels)
        self.assertIn("width: target.width", vector_pixels)
        self.assertIn("height: target.height", vector_pixels)
        self.assertIn("render_bounds.width.max(0.0) * dpi_scale", render_command)
        self.assertIn("physical_pixel_size", image_conversion)
        self.assertIn("mag_filter: wgpu::FilterMode::Linear", wgpu_pipeline)
        self.assertIn("min_filter: wgpu::FilterMode::Linear", wgpu_pipeline)
        self.assertIn("physical_raster_px_size(logical_px, surface_scale_factor)", text_raster)
        self.assertIn("swash_hinting_for_physical_size", text_raster)
        self.assertIn("sample_scale: NATIVE_RASTER_SAMPLE_SCALE", text_raster)
        self.assertIn("raster.sample_scale", text_draw)
        self.assertIn(
            "pub(super) const NATIVE_RASTER_SAMPLE_SCALE: f32 = 1.0;",
            text_raster_metrics,
        )
        self.assertIn(
            "(logical_px * surface_scale_factor)", text_raster_metrics
        )
        self.assertNotIn("TEXT_RASTER_SUPERSAMPLE", text_draw)

    def test_vector_cache_bucketing_cannot_change_non_square_aspect(self):
        vector_target = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "visual_assets/target.rs"
        ).split("#[cfg(test)]", 1)[0]
        cache_bucket = vector_target.split("fn vector_cache_bucket(", 1)[1].split(
            "fn fit_preserving_aspect(", 1
        )[0]

        self.assertIn("if self.width != self.height", cache_bucket)
        self.assertIn("return self;", cache_bucket)
        self.assertIn("self.quantized_up(bucket_edge)", cache_bucket)

    def test_svg_resolve_is_alpha_aware_and_linear_light(self):
        color = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_color.rs"
        )
        vector_pixels = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "visual_assets/svg/pixels.rs"
        )

        self.assertIn("premultiplied_linear_sum[channel]", vector_pixels)
        self.assertIn("srgb_byte_to_linear", vector_pixels)
        self.assertIn(
            "straight_linear = premultiplied_linear_sum[channel] / alpha_sum",
            vector_pixels,
        )
        self.assertIn("linear_to_srgb_byte(straight_linear)", vector_pixels)
        self.assertIn("OnceLock", color)
        self.assertNotIn("powf", vector_pixels)

    def test_avatar_rounding_uses_local_coverage_and_cannot_bypass_the_mask(self):
        avatar_mask = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "material_primitives/avatar/mask.rs"
        )

        self.assertIn("AVATAR_MASK_SAMPLES_PER_AXIS", avatar_mask)
        self.assertIn("rounded_mask_pixel_coverage", avatar_mask)
        self.assertIn("source_alpha * coverage", avatar_mask)
        self.assertIn("image.atlas = None", avatar_mask)

    def test_circular_progress_raster_preserves_analytic_edge_coverage(self):
        circular_progress = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_material_feedback/circular_progress/pixels.rs"
        )

        self.assertIn("coverage: u8", circular_progress)
        self.assertIn("annulus_pixel_coverage", circular_progress)
        self.assertIn("circular_progress_fill_coverage", circular_progress)
        self.assertIn("mix_srgba_linear_by_coverage", circular_progress)
        self.assertIn("scale_alpha_by_coverage", circular_progress)
        self.assertNotIn("distance_squared < inner_squared", circular_progress)

    def test_missing_icon_fallback_resolves_local_edge_samples(self):
        missing_icon = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "visual_assets/loading/missing.rs"
        )

        self.assertIn("MISSING_ICON_SMALL_SAMPLES_PER_AXIS: u32 = 4", missing_icon)
        self.assertIn("MISSING_ICON_LARGE_SAMPLES_PER_AXIS: u32 = 2", missing_icon)
        self.assertIn("missing_icon_pixel_may_be_covered", missing_icon)
        self.assertIn("missing_icon_sample_coverage", missing_icon)
        self.assertIn("scale_alpha_by_coverage", missing_icon)
        self.assertNotIn("x.abs_diff(y) < stroke", missing_icon)

    def test_sample_and_timeline_diamonds_share_cached_local_coverage(self):
        diamond = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_diamond_glyph.rs"
        )
        sample_points = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_sample_grid/points.rs"
        )
        timeline_keys = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_timeline_strip/keys.rs"
        )

        self.assertIn("DIAMOND_SAMPLES_PER_AXIS: u32 = 4", diamond)
        self.assertIn("diamond_sample_coverage", diamond)
        self.assertIn("CACHED_DIAMOND_RASTERS", diamond)
        self.assertIn("let half_edge = edge as f32 * 0.5", diamond)
        self.assertIn("x: x - half_edge", diamond)
        self.assertIn("y: y - half_edge", diamond)
        for consumer in (sample_points, timeline_keys):
            self.assertIn("push_aa_diamond", consumer)
            self.assertNotIn("for offset in -radius..=radius", consumer)

    def test_cached_chart_bitmaps_resolve_local_samples_in_linear_light(self):
        raster_root = (
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "mui_x_primitives/charts/raster/"
        )
        model = source(raster_root + "model.rs")

        self.assertIn("CHART_RASTER_SAMPLES_PER_AXIS: u32 = 4", model)
        self.assertIn("sample_pixel", model)
        self.assertIn("premultiplied_linear", model)
        self.assertIn("srgb_byte_to_linear", model)
        self.assertIn("blend_premultiplied_linear_srgb_pixel", model)
        self.assertNotIn("powf", model)
        for primitive in ("arc.rs", "line.rs", "pie.rs", "shape.rs"):
            primitive_source = source(raster_root + primitive)
            self.assertIn("self.sample_pixel", primitive_source, primitive)
            self.assertNotIn("self.set_pixel", primitive_source, primitive)

    def test_visual_profile_publishes_text_aa_evidence_from_one_command_stream(self):
        export = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "profiling_artifacts/export.rs"
        )
        geometry = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "profiling_artifacts/geometry.rs"
        )
        geometry_schema = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "profiling_artifacts/schema/geometry.rs"
        )
        text_schema = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "profiling_artifacts/schema/text.rs"
        )
        oracle = source("tools/zircon_editor_ui_visual_oracle.py")

        self.assertEqual(export.count("build_chrome_command_stream("), 1)
        self.assertIn("UiProfileGeometry::from_presentation_with_stream", export)
        self.assertIn("paint_chrome_command_stream_to_frame", export)
        self.assertNotIn("paint_host_presentation_snapshot", export)
        self.assertIn("schema_version: 4", geometry)
        self.assertIn("text_runs: collect_text_runs(stream)", geometry)
        self.assertIn("text_length", text_schema)
        self.assertNotIn("text: String", text_schema)
        self.assertIn('profile.get("text_runs")', oracle)
        self.assertIn("text_run_antialiased_ratio", oracle)

    def test_visual_profile_publishes_resolved_rounded_shape_geometry(self):
        export = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "profiling_artifacts/export.rs"
        )
        geometry = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "profiling_artifacts/geometry.rs"
        )
        geometry_schema = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "profiling_artifacts/schema/geometry.rs"
        )
        rounded_schema = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "profiling_artifacts/schema/rounded.rs"
        )
        rounded_geometry = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "profiling_artifacts/geometry/rounded_shapes.rs"
        )
        oracle = source("tools/zircon_editor_ui_visual_oracle.py")

        self.assertIn("rounded_shapes: Vec<UiProfileRoundedShape>", geometry_schema)
        self.assertIn("collect_rounded_shapes(stream)", geometry)
        self.assertIn("corner_radius", rounded_schema)
        self.assertIn("border_width", rounded_schema)
        self.assertIn("ChromeCommandKind::Quad", rounded_geometry)
        self.assertIn("ChromeCommandKind::Border", rounded_geometry)
        self.assertIn("rounded_shapes", oracle)
        self.assertIn("corner_radius", oracle)
        self.assertNotIn("8.0 * float(scale_factor)", oracle)
        self.assertIn("UiProfileGeometry::from_presentation_with_stream", export)


if __name__ == "__main__":
    unittest.main()
