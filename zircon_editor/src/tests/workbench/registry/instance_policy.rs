use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::{
    ViewDescriptor, ViewDescriptorId, ViewHost, ViewKind, ViewRegistry, WorkbenchSlot,
};

#[test]
fn view_registry_reuses_single_instance_and_allows_multi_instance() {
    let mut registry = ViewRegistry::default();
    registry
        .register_view(ViewDescriptor::new(
            ViewDescriptorId::new("editor.hierarchy"),
            ViewKind::ActivityView,
            "Hierarchy",
        ))
        .unwrap();
    registry
        .register_view(
            ViewDescriptor::new(
                ViewDescriptorId::new("editor.prefab"),
                ViewKind::ActivityWindow,
                "Prefab Editor",
            )
            .with_multi_instance(true)
            .with_workbench_slot(WorkbenchSlot::DocumentCenter),
        )
        .unwrap();

    let first = registry
        .open_descriptor(ViewDescriptorId::new("editor.hierarchy"))
        .unwrap();
    let second = registry
        .open_descriptor(ViewDescriptorId::new("editor.hierarchy"))
        .unwrap();
    let prefab_a = registry
        .open_descriptor(ViewDescriptorId::new("editor.prefab"))
        .unwrap();
    let prefab_b = registry
        .open_descriptor(ViewDescriptorId::new("editor.prefab"))
        .unwrap();

    assert_eq!(first.instance_id, second.instance_id);
    assert_ne!(prefab_a.instance_id, prefab_b.instance_id);
}

#[test]
fn workbench_slots_materialize_their_single_canonical_view_hosts() {
    let mut registry = ViewRegistry::default();
    let cases = [
        (WorkbenchSlot::LeftTopDrawer, "left-top"),
        (WorkbenchSlot::LeftBottomDrawer, "left-bottom"),
        (WorkbenchSlot::RightTopDrawer, "right-top"),
        (WorkbenchSlot::RightBottomDrawer, "right-bottom"),
        (WorkbenchSlot::BottomDrawer, "bottom"),
        (WorkbenchSlot::DocumentCenter, "document"),
        (WorkbenchSlot::FloatingWindow, "floating"),
        (WorkbenchSlot::ExclusiveMainPage, "exclusive"),
    ];
    for (slot, id) in cases {
        registry
            .register_view(
                ViewDescriptor::new(
                    ViewDescriptorId::new(format!("editor.slot.{id}")),
                    ViewKind::ActivityView,
                    id,
                )
                .with_workbench_slot(slot),
            )
            .unwrap();
        let instance = registry
            .open_descriptor(ViewDescriptorId::new(format!("editor.slot.{id}")))
            .unwrap();
        match (slot, instance.host) {
            (WorkbenchSlot::LeftTopDrawer, ViewHost::Drawer(ActivityDrawerSlot::LeftTop))
            | (WorkbenchSlot::LeftBottomDrawer, ViewHost::Drawer(ActivityDrawerSlot::LeftBottom))
            | (WorkbenchSlot::RightTopDrawer, ViewHost::Drawer(ActivityDrawerSlot::RightTop))
            | (
                WorkbenchSlot::RightBottomDrawer,
                ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
            )
            | (WorkbenchSlot::BottomDrawer, ViewHost::Drawer(ActivityDrawerSlot::Bottom))
            | (WorkbenchSlot::DocumentCenter, ViewHost::Document(_, _))
            | (WorkbenchSlot::FloatingWindow, ViewHost::FloatingWindow(_, _))
            | (WorkbenchSlot::ExclusiveMainPage, ViewHost::ExclusivePage(_)) => {}
            (slot, host) => panic!("slot {slot:?} materialized unexpected host {host:?}"),
        }
    }
}
