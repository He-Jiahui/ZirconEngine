use std::sync::Mutex;

use tauri::{
    utils::config::WindowConfig, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalRect,
    PhysicalSize,
};

use crate::settings::{default_hub_config_path, HubConfig};

use super::{
    commands::HubCommandState,
    runtime_state::{NormalWindowGeometry, WindowGeometrySample},
    view_model::HubViewModel,
};

#[derive(Default)]
pub(super) struct WindowGeometryCapture(Mutex<WindowGeometrySample>);

impl WindowGeometryCapture {
    fn capture(&self, window: &tauri::Window) -> Result<(), Box<dyn std::error::Error>> {
        if window.is_minimized()? {
            return Ok(());
        }
        let physical_size = window.inner_size()?;
        if physical_size.width == 0 || physical_size.height == 0 {
            return Ok(());
        }

        let maximized = window.is_maximized()?;
        let normal = if maximized {
            None
        } else {
            let logical = physical_size.to_logical::<f64>(window.scale_factor()?);
            let position = window.outer_position()?;
            if !logical.width.is_finite()
                || !logical.height.is_finite()
                || logical.width < 1.0
                || logical.height < 1.0
                || logical.width > f64::from(u32::MAX)
                || logical.height > f64::from(u32::MAX)
            {
                return Ok(());
            }
            Some(NormalWindowGeometry {
                position_x: position.x,
                position_y: position.y,
                width: logical.width.round() as u32,
                height: logical.height.round() as u32,
            })
        };

        let mut sample = self
            .0
            .lock()
            .map_err(|_| crate::HubError::message("Hub window geometry lock is poisoned"))?;
        if let Some(normal) = normal {
            sample.normal = Some(normal);
        }
        sample.maximized = Some(maximized);
        Ok(())
    }

    fn persist(&self, state: &HubCommandState) -> Result<Option<HubViewModel>, crate::HubError> {
        let sample = *self
            .0
            .lock()
            .map_err(|_| crate::HubError::message("Hub window geometry lock is poisoned"))?;
        let mut session = state.session()?;
        let recovered_from_error = session.persist_window_geometry(sample)?;
        Ok(recovered_from_error.then(|| session.publish_view_model()))
    }
}

fn publish_window_state(window: &tauri::Window, view_model: &HubViewModel) {
    if let Err(error) = window.emit("hub-state-changed", view_model) {
        eprintln!("zircon_hub: failed to publish window state: {error}");
    }
}

pub(super) fn handle_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    if window.label() != "main" {
        return;
    }
    let capture = window.state::<WindowGeometryCapture>();
    match event {
        tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Moved(_) => {
            if let Err(error) = capture.capture(window) {
                eprintln!("zircon_hub: failed to capture window geometry: {error}");
            }
        }
        tauri::WindowEvent::Focused(false) | tauri::WindowEvent::CloseRequested { .. } => {
            if let Err(error) = capture.capture(window) {
                eprintln!("zircon_hub: failed to capture window geometry: {error}");
            }
            let state = window.state::<HubCommandState>();
            match capture.persist(&state) {
                Ok(Some(view_model)) => publish_window_state(window, &view_model),
                Ok(None) => {}
                Err(error) => {
                    eprintln!("zircon_hub: failed to save window geometry: {error}");
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // Keep the window alive so the same geometry can be saved on a later close.
                        api.prevent_close();
                        match state.session() {
                            Ok(mut session) => {
                                let view_model =
                                    session.report_window_geometry_save_failure(&error);
                                drop(session);
                                publish_window_state(window, &view_model);
                            }
                            Err(session_error) => {
                                eprintln!(
                                    "zircon_hub: failed to report window save error: {session_error}"
                                );
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

pub(super) fn restore_saved_window_state(
    app: &mut tauri::App,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = HubConfig::load(default_hub_config_path())?;
    let native = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .ok_or_else(|| crate::HubError::message("Hub main window configuration is missing"))?;
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| crate::HubError::message("Hub main window was not created"))?;

    let saved_position = config.window.position_x.zip(config.window.position_y);
    let saved_monitor = if let Some(position) = saved_position {
        window
            .available_monitors()?
            .into_iter()
            .find(|monitor| position_in_work_area(position, monitor.work_area()))
    } else {
        None
    };
    let position_is_on_monitor = saved_monitor.is_some();
    let monitor = if let Some(monitor) = saved_monitor {
        Some(monitor)
    } else if let Some(current) = window.current_monitor()? {
        Some(current)
    } else {
        window.primary_monitor()?
    };
    let work_area = monitor.as_ref().map(|monitor| {
        monitor
            .work_area()
            .size
            .to_logical::<f64>(monitor.scale_factor())
    });
    let size = saved_logical_size(&config, native, work_area);
    if let Some(size) = size {
        window.set_size(size)?;
    }
    if position_is_on_monitor {
        if let (Some(position), Some(monitor)) = (saved_position, monitor) {
            let physical_size = if let Some(logical) = size {
                logical.to_physical::<u32>(monitor.scale_factor())
            } else {
                window.inner_size()?
            };
            window.set_position(clamp_position_to_work_area(
                position,
                physical_size,
                monitor.work_area(),
            ))?;
        }
    } else if saved_position.is_some() || size.is_some() {
        // A removed monitor cannot strand the Hub outside the visible desktop.
        window.center()?;
    }
    if config.window.maximized {
        window.maximize()?;
    }
    Ok(())
}

fn position_in_work_area(position: (i32, i32), area: &PhysicalRect<i32, u32>) -> bool {
    let right = i64::from(area.position.x) + i64::from(area.size.width);
    let bottom = i64::from(area.position.y) + i64::from(area.size.height);
    i64::from(position.0) >= i64::from(area.position.x)
        && i64::from(position.0) < right
        && i64::from(position.1) >= i64::from(area.position.y)
        && i64::from(position.1) < bottom
}

fn clamp_position_to_work_area(
    position: (i32, i32),
    size: PhysicalSize<u32>,
    area: &PhysicalRect<i32, u32>,
) -> PhysicalPosition<i32> {
    fn clamp_axis(position: i32, start: i32, available: u32, extent: u32) -> i32 {
        let end = i64::from(start) + (i64::from(available) - i64::from(extent)).max(0);
        i64::from(position).clamp(i64::from(start), end) as i32
    }
    PhysicalPosition::new(
        clamp_axis(position.0, area.position.x, area.size.width, size.width),
        clamp_axis(position.1, area.position.y, area.size.height, size.height),
    )
}

fn saved_logical_size(
    config: &HubConfig,
    native: &WindowConfig,
    work_area: Option<LogicalSize<f64>>,
) -> Option<LogicalSize<f64>> {
    let saved = &config.window;
    if saved.width.unwrap_or_default() == 0
        && saved.height.unwrap_or_default() == 0
        && saved.position_x.zip(saved.position_y).is_none()
        && !saved.maximized
        && !work_area
            .as_ref()
            .is_some_and(|area| area.width < native.width || area.height < native.height)
    {
        return None;
    }

    // Persisted and native dimensions are logical pixels; monitor work area starts in physical pixels.
    let width = fit_dimension(
        saved.width,
        native.width,
        native.min_width,
        work_area.as_ref().map(|area| area.width),
    );
    let height = fit_dimension(
        saved.height,
        native.height,
        native.min_height,
        work_area.as_ref().map(|area| area.height),
    );
    Some(LogicalSize::new(width, height))
}

fn fit_dimension(
    saved: Option<u32>,
    default: f64,
    minimum: Option<f64>,
    available: Option<f64>,
) -> f64 {
    let minimum = minimum.unwrap_or(1.0);
    let requested = saved
        .filter(|size| *size > 0)
        .map(f64::from)
        .unwrap_or(default)
        .max(minimum);

    // When a monitor is smaller than the native minimum, keep the native constraint authoritative.
    available
        .filter(|size| size.is_finite() && *size >= minimum)
        .map_or(requested, |size| requested.min(size))
}

#[cfg(test)]
#[path = "tests/window_state.rs"]
mod tests;
