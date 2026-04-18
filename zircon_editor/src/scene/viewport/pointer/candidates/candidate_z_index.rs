use crate::scene::viewport::pointer::constants::{
    GIZMO_PRIORITY, HANDLE_PRIORITY, RENDERABLE_PRIORITY,
};

pub(in crate::scene::viewport::pointer) fn candidate_z_index(priority: u8) -> i32 {
    match priority {
        HANDLE_PRIORITY => 300,
        GIZMO_PRIORITY => 200,
        RENDERABLE_PRIORITY => 100,
        _ => 0,
    }
}
