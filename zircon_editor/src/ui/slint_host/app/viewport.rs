use super::*;
use crate::ui::slint_host::root_shell_projection::{
    resolve_root_bottom_region_frame, resolve_root_left_region_frame,
    resolve_root_right_region_frame, resolve_root_viewport_content_frame,
};
use crate::{ActivityDrawerSlot, ViewHost};
use zircon_ui::{UiPointerButton, UiPointerEvent, UiPointerEventKind};

const VIEWPORT_POINTER_DOWN: i32 = 0;
const VIEWPORT_POINTER_MOVE: i32 = 1;
const VIEWPORT_POINTER_UP: i32 = 2;
const VIEWPORT_POINTER_SCROLL: i32 = 3;

const VIEWPORT_POINTER_BUTTON_NONE: i32 = 0;
const VIEWPORT_POINTER_BUTTON_PRIMARY: i32 = 1;
const VIEWPORT_POINTER_BUTTON_SECONDARY: i32 = 2;
const VIEWPORT_POINTER_BUTTON_MIDDLE: i32 = 3;

impl SlintEditorHost {
    pub(super) fn viewport_pointer_event(
        &mut self,
        kind: i32,
        button: i32,
        x: f32,
        y: f32,
        delta: f32,
    ) {
        self.recompute_if_dirty();
        let event = match map_viewport_pointer_event(kind, button, x, y, delta) {
            Ok(event) => event,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        if event.kind != UiPointerEventKind::Move {
            self.focus_callback_source_window();
        }

        match callback_dispatch::dispatch_viewport_pointer_event(
            &self.runtime,
            &mut self.viewport_pointer_bridge,
            event,
        ) {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn viewport_toolbar_pointer_clicked(
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
        self.recompute_if_dirty();
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

    pub(super) fn viewport_toolbar_surface_size(&self, surface_key: &str) -> UiSize {
        const TOOLBAR_HEIGHT: f32 = 28.0;

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
                    TOOLBAR_HEIGHT,
                );
            }
        }

        let root_shell_frames = self.template_bridge.root_shell_frames();
        let default_geometry = WorkbenchShellGeometry::default();
        let geometry = self.shell_geometry.as_ref().unwrap_or(&default_geometry);

        let width = current_instance
            .map(|instance| match instance.host {
                ViewHost::FloatingWindow(_, _) => unreachable!(
                    "floating window toolbar size should return early through the projection helper"
                ),
                ViewHost::Document(_, _) => root_shell_frames
                    .pane_surface_frame
                    .or(root_shell_frames.document_host_frame)
                    .filter(|frame| frame.width > f32::EPSILON)
                    .map(|frame| frame.width)
                    .or_else(|| {
                        let frame = resolve_root_viewport_content_frame(
                            geometry,
                            Some(&root_shell_frames),
                            true,
                        );
                        (frame.width > f32::EPSILON).then_some(frame.width)
                    })
                    .unwrap_or(self.shell_size.width)
                    .max(1.0),
                ViewHost::Drawer(slot) => {
                    let region = match slot {
                        ActivityDrawerSlot::LeftTop | ActivityDrawerSlot::LeftBottom => {
                            ShellRegionId::Left
                        }
                        ActivityDrawerSlot::RightTop | ActivityDrawerSlot::RightBottom => {
                            ShellRegionId::Right
                        }
                        ActivityDrawerSlot::BottomLeft | ActivityDrawerSlot::BottomRight => {
                            ShellRegionId::Bottom
                        }
                    };
                    root_shell_frames
                        .drawer_content_frame(region)
                        .filter(|frame| frame.width > f32::EPSILON)
                        .map(|frame| frame.width)
                        .or_else(|| {
                            let frame = match region {
                                ShellRegionId::Left => resolve_root_left_region_frame(
                                    geometry,
                                    Some(&root_shell_frames),
                                ),
                                ShellRegionId::Right => resolve_root_right_region_frame(
                                    geometry,
                                    Some(&root_shell_frames),
                                ),
                                ShellRegionId::Bottom => resolve_root_bottom_region_frame(
                                    geometry,
                                    Some(&root_shell_frames),
                                ),
                                ShellRegionId::Document => Default::default(),
                            };
                            (frame.width > f32::EPSILON).then_some(frame.width)
                        })
                        .unwrap_or(0.0)
                }
                ViewHost::ExclusivePage(_) => self.shell_size.width,
            })
            .unwrap_or_else(|| {
                resolve_root_viewport_content_frame(geometry, Some(&root_shell_frames), true)
                    .width
                    .max(1.0)
            });

        UiSize::new(width.max(1.0), TOOLBAR_HEIGHT)
    }
}

fn map_viewport_pointer_event(
    kind: i32,
    button: i32,
    x: f32,
    y: f32,
    delta: f32,
) -> Result<UiPointerEvent, String> {
    let kind = match kind {
        VIEWPORT_POINTER_DOWN => UiPointerEventKind::Down,
        VIEWPORT_POINTER_MOVE => UiPointerEventKind::Move,
        VIEWPORT_POINTER_UP => UiPointerEventKind::Up,
        VIEWPORT_POINTER_SCROLL => UiPointerEventKind::Scroll,
        _ => return Err(format!("unknown viewport pointer kind {kind}")),
    };

    let mut event = UiPointerEvent::new(kind, UiPoint::new(x, y));
    if let Some(button) = map_viewport_pointer_button(button)? {
        event = event.with_button(button);
    }
    if kind == UiPointerEventKind::Scroll {
        event = event.with_scroll_delta(delta);
    }
    Ok(event)
}

fn map_viewport_pointer_button(button: i32) -> Result<Option<UiPointerButton>, String> {
    match button {
        VIEWPORT_POINTER_BUTTON_NONE => Ok(None),
        VIEWPORT_POINTER_BUTTON_PRIMARY => Ok(Some(UiPointerButton::Primary)),
        VIEWPORT_POINTER_BUTTON_SECONDARY => Ok(Some(UiPointerButton::Secondary)),
        VIEWPORT_POINTER_BUTTON_MIDDLE => Ok(Some(UiPointerButton::Middle)),
        _ => Err(format!("unknown viewport pointer button {button}")),
    }
}
