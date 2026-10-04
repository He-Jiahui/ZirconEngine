//! 启动窗口描述符到 Winit 创建属性的单次转换。
//! 显示器查询仅服务本次创建；默认位置、全屏降级和尺寸约束由各 leaf 保持一致。

use winit::dpi::{LogicalSize, PhysicalSize, Size};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;
use zircon_runtime::core::framework::window::{WindowDescriptor, WindowResizeConstraints};

use super::fullscreen::runtime_window_fullscreen;
use super::monitor::WindowMonitorContext;
use super::position::runtime_window_position;

/// 从启动描述符和当前事件循环取显示器上下文，形成一次创建窗口所需的属性。
pub(in crate::entry::runtime_entry_app) fn runtime_window_attributes(
    descriptor: &WindowDescriptor,
    event_loop: &dyn ActiveEventLoop,
) -> WindowAttributes {
    let monitor_context =
        WindowMonitorContext::for_event_loop(event_loop, descriptor.position, descriptor.mode);
    runtime_window_attributes_with_monitor_context(descriptor, &monitor_context)
}

#[cfg(test)]
fn runtime_window_attributes_with_primary_monitor(
    descriptor: &WindowDescriptor,
    primary_monitor: Option<winit::monitor::MonitorHandle>,
) -> WindowAttributes {
    let monitor_context = WindowMonitorContext::primary_only(primary_monitor);
    runtime_window_attributes_with_monitor_context(descriptor, &monitor_context)
}

// 物理像素初始尺寸与逻辑尺寸约束分别交给 Winit，后端负责 DPI 换算。
fn runtime_window_attributes_with_monitor_context(
    descriptor: &WindowDescriptor,
    monitor_context: &WindowMonitorContext,
) -> WindowAttributes {
    let physical_size = descriptor.resolution.physical_size();
    let constraints = descriptor.resize_constraints.validated();
    let mut attributes = WindowAttributes::default()
        .with_title(descriptor.title.clone())
        .with_surface_size(Size::Physical(PhysicalSize::new(
            physical_size.x,
            physical_size.y,
        )))
        .with_min_surface_size(Size::Logical(LogicalSize::new(
            constraints.min_width as f64,
            constraints.min_height as f64,
        )))
        .with_resizable(descriptor.resizable)
        .with_decorations(descriptor.decorated)
        .with_visible(descriptor.visible)
        .with_active(descriptor.focused);

    if let Some(max_size) = finite_max_surface_size(constraints) {
        attributes = attributes.with_max_surface_size(max_size);
    }

    if let Some(position) =
        runtime_window_position(descriptor.position, &descriptor.resolution, monitor_context)
    {
        attributes = attributes.with_position(winit::dpi::Position::Physical(position));
    }

    if let Some(fullscreen) = runtime_window_fullscreen(descriptor.mode, monitor_context) {
        attributes = attributes.with_fullscreen(Some(fullscreen));
    }

    attributes
}

// TODO: [CR-APP-ENTRY-0016] 确认单轴有限上限的启动策略：另一轴无界时本函数忽略整个最大尺寸，已有有限约束也不会提交；需核对后端支持并补对应场景证据。
fn finite_max_surface_size(constraints: WindowResizeConstraints) -> Option<Size> {
    if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
        Some(Size::Logical(LogicalSize::new(
            constraints.max_width as f64,
            constraints.max_height as f64,
        )))
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/builder.rs"]
mod tests;
