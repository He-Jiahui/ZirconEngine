use super::*;

#[derive(Clone, Debug)]
pub(crate) struct ActiveDrawerResize {
    pub(super) source_window: Option<MainPageId>,
    pub(super) region: ShellRegionId,
    pub(super) start_x: f32,
    pub(super) start_y: f32,
    pub(super) base_preferred: f32,
}

mod drag_drop;
mod drawer_resize;

const HOST_POINTER_DOWN: i32 = 0;
const HOST_POINTER_MOVE: i32 = 1;
const HOST_POINTER_UP: i32 = 2;
const HOST_POINTER_CANCEL: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostPointerFactKind {
    Down,
    Move,
    Up,
    Cancel,
}

impl RetainedEditorHost {
    pub(super) fn host_drag_pointer_event(
        &mut self,
        source_window_id: Option<&MainPageId>,
        kind: i32,
        x: f32,
        y: f32,
    ) {
        self.use_committed_pointer_layout();
        let kind = match map_host_pointer_kind(kind, "drag") {
            Ok(kind) => kind,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };

        match kind {
            HostPointerFactKind::Down | HostPointerFactKind::Move => {
                let _ = self.sync_drag_target_group(source_window_id, x, y);
            }
            HostPointerFactKind::Up => self.dispatch_drag_drop_from_pointer(source_window_id, x, y),
            HostPointerFactKind::Cancel => {}
        }
    }

    pub(super) fn host_resize_pointer_event(
        &mut self,
        source_window_id: Option<&MainPageId>,
        kind: i32,
        x: f32,
        y: f32,
    ) {
        let kind = match map_host_pointer_kind(kind, "resize") {
            Ok(kind) => kind,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        let source_matches_active = self
            .active_drawer_resize
            .as_ref()
            .is_some_and(|active| active.source_window.as_ref() == source_window_id);
        match kind {
            HostPointerFactKind::Down if self.active_drawer_resize.is_some() => return,
            HostPointerFactKind::Down => {}
            _ if !source_matches_active => return,
            _ => {}
        }

        self.use_committed_pointer_layout();
        match kind {
            HostPointerFactKind::Down => self.begin_drawer_resize_capture(source_window_id, x, y),
            HostPointerFactKind::Move => self.update_drawer_resize_capture(source_window_id, x, y),
            HostPointerFactKind::Up => self.finish_drawer_resize_capture(source_window_id, x, y),
            HostPointerFactKind::Cancel => self.cancel_drawer_resize_capture(source_window_id),
        }
    }

    pub(in crate::ui::retained_host::app) fn cancel_removed_drawer_resize_owner(&mut self) {
        let removed_owner = {
            let Some(active) = self.active_drawer_resize.as_ref() else {
                return;
            };
            let Some(owner) = active.source_window.as_ref() else {
                return;
            };
            (!self.runtime.floating_window_exists(owner)).then(|| owner.clone())
        };
        let Some(owner) = removed_owner else {
            return;
        };

        if let Some(window) = self.native_window_presenters.window(&owner) {
            window.clear_native_resize_capture();
        }
        self.cancel_drawer_resize_capture(Some(&owner));
    }
}

fn map_host_pointer_kind(kind: i32, channel: &str) -> Result<HostPointerFactKind, String> {
    match kind {
        HOST_POINTER_DOWN => Ok(HostPointerFactKind::Down),
        HOST_POINTER_MOVE => Ok(HostPointerFactKind::Move),
        HOST_POINTER_UP => Ok(HostPointerFactKind::Up),
        HOST_POINTER_CANCEL => Ok(HostPointerFactKind::Cancel),
        _ => Err(format!("unknown host {channel} pointer kind {kind}")),
    }
}
