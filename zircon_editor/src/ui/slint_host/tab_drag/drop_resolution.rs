use crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::host_resolution::drop_host_for_tab;
use super::resolved_drop::ResolvedTabDrop;
use super::strip_hitbox::precise_drop_target;

#[cfg(test)]
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
    resolve_tab_drop_with_root_frames(
        layout,
        model,
        geometry,
        metrics,
        instance_id,
        target_group,
        pointer_x,
        pointer_y,
        None,
    )
}

pub(crate) fn resolve_tab_drop_with_root_frames(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> Option<ResolvedTabDrop> {
    precise_drop_target(
        model,
        geometry,
        metrics,
        instance_id,
        target_group,
        pointer_x,
        pointer_y,
        shared_root_frames,
    )
    .or_else(|| {
        drop_host_for_tab(layout, instance_id, target_group)
            .map(|host| ResolvedTabDrop { host, anchor: None })
    })
}
