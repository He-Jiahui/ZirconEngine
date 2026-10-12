#[cfg(test)]
use crate::ui::retained_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::host_resolution::drop_host_for_tab;
use super::resolved_drop::ResolvedTabDrop;
#[cfg(test)]
use super::strip_hitbox::precise_drop_target;
use super::strip_hitbox::precise_drop_target_with_workbench_layout_frames;

#[cfg(test)]
pub(crate) fn resolve_tab_drop_with_root_frames(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<ResolvedTabDrop> {
    precise_drop_target(
        model,
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

pub(crate) fn resolve_tab_drop_with_workbench_layout_frames(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> Option<ResolvedTabDrop> {
    precise_drop_target_with_workbench_layout_frames(
        model,
        metrics,
        instance_id,
        target_group,
        pointer_x,
        pointer_y,
        componentized_workbench_layout_frames,
    )
    .or_else(|| {
        drop_host_for_tab(layout, instance_id, target_group)
            .map(|host| ResolvedTabDrop { host, anchor: None })
    })
}
