use super::super::{callback_dispatch, RetainedEditorHost};
use crate::ui::retained_host::viewport_toolbar_pointer::build_viewport_toolbar_pointer_layout_with_size;
use crate::ui::workbench::{autolayout::ShellRegionId, layout::ActivityDrawerSlot, view::ViewHost};
use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint, UiSize};

const VIEWPORT_TOOLBAR_HEIGHT: f32 = 28.0;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn viewport_toolbar_pointer_clicked(
        &mut self,
        surface_key: &str,
        control_id: &str,
        control_x: f32,
        control_y: f32,
        control_width: f32,
        control_height: f32,
        point_x: f32,
        point_y: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let surface_size = self.viewport_toolbar_surface_size(surface_key);
        let _ = self.viewport_toolbar_bridge.recompute_layout(surface_size);
        self.viewport_toolbar_pointer_bridge
            .sync(build_viewport_toolbar_pointer_layout_with_size(
                [surface_key],
                surface_size,
            ));
        match callback_dispatch::dispatch_shared_viewport_toolbar_pointer_click(
            &self.runtime,
            &self.viewport_toolbar_bridge,
            &mut self.viewport_toolbar_pointer_bridge,
            surface_key,
            control_id,
            control_x,
            control_y,
            control_width,
            control_height,
            UiPoint::new(point_x, point_y),
        ) {
            Ok(dispatch) => {
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(in crate::ui::retained_host::app) fn viewport_toolbar_surface_size(
        &self,
        surface_key: &str,
    ) -> UiSize {
        let current_instance = self
            .runtime
            .current_view_instances()
            .into_iter()
            .find(|instance| instance.instance_id.0 == surface_key);
        if let Some(instance) = current_instance.as_ref() {
            if let ViewHost::FloatingWindow(window_id, _) = &instance.host {
                return UiSize::new(
                    self.resolve_floating_window_content_frame_for_window(window_id)
                        .unwrap_or_default()
                        .width
                        .max(1.0),
                    VIEWPORT_TOOLBAR_HEIGHT,
                );
            }
        }

        let workbench_layout_frames = self.workbench_window_bridge.layout_frames();
        let width = current_instance
            .map(|instance| match instance.host {
                ViewHost::FloatingWindow(_, _) => unreachable!(
                    "floating window toolbar size should return early through the projection helper"
                ),
                ViewHost::Document(_, _) => self
                    .componentized_document_viewport_toolbar_width()
                    .unwrap_or_default(),
                ViewHost::Drawer(slot) => {
                    let region = match slot {
                        ActivityDrawerSlot::LeftTop | ActivityDrawerSlot::LeftBottom => {
                            ShellRegionId::Left
                        }
                        ActivityDrawerSlot::RightTop | ActivityDrawerSlot::RightBottom => {
                            ShellRegionId::Right
                        }
                        ActivityDrawerSlot::Bottom
                        | ActivityDrawerSlot::BottomLeft
                        | ActivityDrawerSlot::BottomRight => ShellRegionId::Bottom,
                    };
                    workbench_layout_frames
                        .drawer_content_frame(region)
                        .or_else(|| workbench_layout_frames.drawer_shell_frame(region))
                        .filter(|frame| frame.width > f32::EPSILON)
                        .map(|frame| frame.width)
                        .unwrap_or(0.0)
                }
                ViewHost::ExclusivePage(_) => self.shell_size.width,
            })
            .unwrap_or_else(|| {
                self.componentized_document_viewport_toolbar_width()
                    .unwrap_or_default()
            });

        UiSize::new(width.max(1.0), VIEWPORT_TOOLBAR_HEIGHT)
    }

    fn componentized_document_viewport_toolbar_width(&self) -> Option<f32> {
        let workbench_layout_frames = self.workbench_window_bridge.layout_frames();
        workbench_layout_frames
            .viewport_toolbar_frame
            .and_then(visible_frame_width)
            .or_else(|| {
                workbench_layout_frames
                    .viewport_content_frame
                    .and_then(visible_frame_width)
            })
    }
}

fn visible_frame_width(frame: UiFrame) -> Option<f32> {
    (frame.width > f32::EPSILON).then_some(frame.width)
}
