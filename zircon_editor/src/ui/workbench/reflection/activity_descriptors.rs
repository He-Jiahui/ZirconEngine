use crate::ui::{ActivityViewDescriptor, ActivityWindowDescriptor};
use zircon_runtime_interface::ui::event_ui::UiNodePath;

use crate::ui::workbench::view::{DockPolicy, PreferredHost, ViewDescriptor, ViewKind};

use super::drawer_slot_preference::drawer_slot_preference;

pub fn activity_descriptors_from_views(
    descriptors: &[ViewDescriptor],
) -> (Vec<ActivityViewDescriptor>, Vec<ActivityWindowDescriptor>) {
    let mut activity_views = Vec::new();
    let mut activity_windows = Vec::new();

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
                if let Some(slot) = descriptor.preferred_drawer_slot {
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
                    descriptor.preferred_host,
                    PreferredHost::ExclusiveMainPage
                ))
                .with_supports_exclusive_page(matches!(
                    descriptor.preferred_host,
                    PreferredHost::ExclusiveMainPage | PreferredHost::DocumentCenter
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
