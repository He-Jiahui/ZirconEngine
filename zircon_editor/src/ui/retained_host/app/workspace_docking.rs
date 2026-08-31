use super::*;

mod drag_drop;
mod drawer_resize;

const HOST_POINTER_DOWN: i32 = 0;
const HOST_POINTER_MOVE: i32 = 1;
const HOST_POINTER_UP: i32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostPointerFactKind {
    Down,
    Move,
    Up,
}

impl RetainedEditorHost {
    pub(super) fn host_drag_pointer_event(&mut self, kind: i32, x: f32, y: f32) {
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
                let _ = self.sync_drag_target_group(x, y);
            }
            HostPointerFactKind::Up => self.dispatch_drag_drop_from_pointer(x, y),
        }
    }

    pub(super) fn host_resize_pointer_event(&mut self, kind: i32, x: f32, y: f32) {
        self.use_committed_pointer_layout();
        let kind = match map_host_pointer_kind(kind, "resize") {
            Ok(kind) => kind,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };

        match kind {
            HostPointerFactKind::Down => self.begin_drawer_resize_capture(x, y),
            HostPointerFactKind::Move => self.update_drawer_resize_capture(x, y),
            HostPointerFactKind::Up => self.finish_drawer_resize_capture(x, y),
        }
    }
}

fn map_host_pointer_kind(kind: i32, channel: &str) -> Result<HostPointerFactKind, String> {
    match kind {
        HOST_POINTER_DOWN => Ok(HostPointerFactKind::Down),
        HOST_POINTER_MOVE => Ok(HostPointerFactKind::Move),
        HOST_POINTER_UP => Ok(HostPointerFactKind::Up),
        _ => Err(format!("unknown host {channel} pointer kind {kind}")),
    }
}
