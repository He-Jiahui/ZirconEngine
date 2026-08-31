use crate::ui::{ActivityViewDescriptor, ActivityWindowDescriptor};
use zircon_runtime_interface::ui::event_ui::UiNodePath;

use crate::core::extension::WorkbenchSlot;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::{DockPolicy, ViewDescriptor, ViewKind};

use super::drawer_slot_preference::drawer_slot_preference;

#[cfg(test)]
#[path = "activity_descriptors/capacity_tests.rs"]
mod capacity_tests;

pub fn activity_descriptors_from_views(
    descriptors: &[ViewDescriptor],
) -> (Vec<ActivityViewDescriptor>, Vec<ActivityWindowDescriptor>) {
    let (view_count, window_count) = activity_descriptor_capacities(descriptors);
    let mut activity_views = Vec::with_capacity(view_count);
    let mut activity_windows = Vec::with_capacity(window_count);

    for descriptor in descriptors {
        match descriptor.kind {
            ViewKind::ActivityView => {
                let mut activity = ActivityViewDescriptor::new(
                    descriptor.descriptor_id.0.clone(),
                    descriptor.default_title.clone(),
                    descriptor.icon_key.clone(),
                )
                .with_multi_instance(descriptor.multi_instance)
                .with_supports_document_host(!matches!(
                    descriptor.dock_policy,
                    DockPolicy::DrawerOnly
                ))
                .with_supports_floating_window(!matches!(
                    descriptor.dock_policy,
                    DockPolicy::DrawerOnly
                ))
                .with_reflection_root(UiNodePath::new(format!(
                    "editor/views/{}",
                    descriptor.descriptor_id.0
                )));
                if let Some(slot) = drawer_slot(descriptor.workbench_slot) {
                    activity = activity.with_default_drawer(drawer_slot_preference(slot));
                }
                activity_views.push(activity);
            }
            ViewKind::ActivityWindow => {
                let activity = ActivityWindowDescriptor::new(
                    descriptor.descriptor_id.0.clone(),
                    descriptor.default_title.clone(),
                    descriptor.icon_key.clone(),
                )
                .with_multi_instance(descriptor.multi_instance)
                .with_supports_document_tab(!matches!(
                    descriptor.workbench_slot,
                    WorkbenchSlot::ExclusiveMainPage
                ))
                .with_supports_exclusive_page(matches!(
                    descriptor.workbench_slot,
                    WorkbenchSlot::ExclusiveMainPage | WorkbenchSlot::DocumentCenter
                ))
                .with_supports_floating_window(true)
                .with_reflection_root(UiNodePath::new(format!(
                    "editor/windows/{}",
                    descriptor.descriptor_id.0
                )));
                activity_windows.push(activity);
            }
        }
    }

    (activity_views, activity_windows)
}

fn activity_descriptor_capacities(descriptors: &[ViewDescriptor]) -> (usize, usize) {
    descriptors.iter().fold(
        (0usize, 0usize),
        |(views, windows), descriptor| match descriptor.kind {
            ViewKind::ActivityView => (views.saturating_add(1), windows),
            ViewKind::ActivityWindow => (views, windows.saturating_add(1)),
        },
    )
}

fn drawer_slot(workbench_slot: WorkbenchSlot) -> Option<ActivityDrawerSlot> {
    match workbench_slot {
        WorkbenchSlot::LeftTopDrawer => Some(ActivityDrawerSlot::LeftTop),
        WorkbenchSlot::LeftBottomDrawer => Some(ActivityDrawerSlot::LeftBottom),
        WorkbenchSlot::RightTopDrawer => Some(ActivityDrawerSlot::RightTop),
        WorkbenchSlot::RightBottomDrawer => Some(ActivityDrawerSlot::RightBottom),
        WorkbenchSlot::BottomDrawer => Some(ActivityDrawerSlot::Bottom),
        WorkbenchSlot::DocumentCenter
        | WorkbenchSlot::FloatingWindow
        | WorkbenchSlot::ExclusiveMainPage => None,
    }
}
