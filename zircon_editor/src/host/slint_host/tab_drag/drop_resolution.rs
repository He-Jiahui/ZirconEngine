use crate::{WorkbenchChromeMetrics, WorkbenchLayout, WorkbenchShellGeometry, WorkbenchViewModel};

use super::host_resolution::drop_host_for_tab;
use super::resolved_drop::ResolvedTabDrop;
use super::strip_hitbox::precise_drop_target;

pub(crate) fn resolve_tab_drop(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
) -> Option<ResolvedTabDrop> {
    precise_drop_target(
        model,
        geometry,
        metrics,
        instance_id,
        target_group,
        pointer_x,
        pointer_y,
    )
    .or_else(|| {
        drop_host_for_tab(layout, instance_id, target_group)
            .map(|host| ResolvedTabDrop { host, anchor: None })
    })
}
