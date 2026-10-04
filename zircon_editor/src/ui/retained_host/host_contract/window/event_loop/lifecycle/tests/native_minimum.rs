use super::{
    current_native_minimum_surface_size, logical_to_physical, physical_minimum_size,
    published_or_fallback_physical_extent,
};
use crate::core::commands::EditorCommandRegistry;
use crate::ui::layouts::windows::workbench_host_window::{
    build_host_window_shell_data, HostChromeProjectionCache,
};
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellSizePx, WorkbenchChromeMetrics,
};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn astra_editor_native_minimum_rounds_logical_extent_up_at_dpi() {
    assert_eq!(
        physical_minimum_size(720.0, 480.0, 1.0),
        winit::dpi::PhysicalSize::new(720, 480)
    );
    assert_eq!(
        physical_minimum_size(720.0, 480.0, 1.25),
        winit::dpi::PhysicalSize::new(900, 600)
    );
    assert_eq!(
        physical_minimum_size(720.0, 480.0, 2.0),
        winit::dpi::PhysicalSize::new(1440, 960)
    );
    assert_eq!(logical_to_physical(720.01, 1.25), 901);
    assert_eq!(logical_to_physical(480.0, 2.0), 960);
}

#[test]
fn astra_editor_native_minimum_sanitizes_scale_and_non_finite_extent() {
    assert_eq!(physical_minimum_size(f32::NAN, 480.0, 0.0).width, 0);
    assert_eq!(
        physical_minimum_size(720.0, f32::INFINITY, f32::NAN).height,
        0
    );
    assert_eq!(physical_minimum_size(720.0, 480.0, f32::NAN).width, 720);
}

#[test]
fn astra_editor_two_x_geometry_minimum_is_published_to_native_exactly_once() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&EditorCommandRegistry::default_workbench(), &chrome);
    let solve = |scale| {
        compute_workbench_shell_geometry(
            &model,
            &chrome,
            &fixture.layout,
            &fixture.descriptors,
            ShellSizePx::new(1_280.0 * scale, 720.0 * scale),
            scale,
            &WorkbenchChromeMetrics::default(),
            None,
        )
    };
    let logical = solve(1.0);
    let physical = solve(2.0);
    assert_eq!(physical.window_min_width, logical.window_min_width * 2.0);
    assert_eq!(physical.window_min_height, logical.window_min_height * 2.0);
    let shell = build_host_window_shell_data(
        &model,
        &chrome,
        &physical,
        &[],
        None,
        &mut HostChromeProjectionCache::default(),
    );
    let host = UiHostWindow::new().expect("test host");
    let handle = host.window();
    {
        let mut state = handle.state.borrow_mut();
        state.window_scale_factor = 2.0;
        let presentation = std::sync::Arc::make_mut(&mut state.host_presentation);
        presentation.host_shell.shell_min_width_px = shell.shell_min_width_px;
        presentation.host_shell.shell_min_height_px = shell.shell_min_height_px;
    }
    let native = current_native_minimum_surface_size(&host);
    assert_eq!(native.width, physical.window_min_width.ceil() as u32);
    assert_eq!(native.height, physical.window_min_height.ceil() as u32);
    assert_eq!(
        published_or_fallback_physical_extent(0.0, 720.0, 2.0),
        1_440
    );
}

#[test]
fn astra_editor_native_minimum_is_synchronized_by_the_event_loop_redraw_owner() {
    let redraw = include_str!("../../redraw.rs");
    let production = redraw.split("#[cfg(test)]").next().expect("redraw source");
    let frame_update = production
        .find("self.host.request_frame_update()")
        .expect("retained frame update");
    let minimum_sync = production
        .find("self.sync_native_window_minimum_from_presentation()")
        .expect("native minimum sync");
    assert!(frame_update < minimum_sync);
}
