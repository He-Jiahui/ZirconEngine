use std::slice;

use zircon_runtime::core::framework::navigation::{NavPathPoint, NavPathResult, NavPathStatus};

use crate::ffi::ZrNavDetourPathResult;

const DT_STRAIGHTPATH_OFFMESH_CONNECTION: u8 = 0x04;

pub(crate) fn convert_path_result(result: &ZrNavDetourPathResult) -> Option<NavPathResult> {
    if result.points.is_null() || result.point_count == 0 {
        return None;
    }
    let points = unsafe { slice::from_raw_parts(result.points, result.point_count as usize) }
        .iter()
        .map(|point| NavPathPoint {
            position: point.position,
            area: point.area,
            flags: path_point_flags(point.flags),
        })
        .collect::<Vec<_>>();
    Some(NavPathResult {
        status: NavPathStatus::Complete,
        points,
        length: result.length,
        visited_nodes: (result.visited_nodes as usize).max(1),
    })
}

fn path_point_flags(flags: u8) -> Vec<String> {
    if flags & DT_STRAIGHTPATH_OFFMESH_CONNECTION == 0 {
        return Vec::new();
    }
    vec!["off_mesh_link".to_string()]
}
