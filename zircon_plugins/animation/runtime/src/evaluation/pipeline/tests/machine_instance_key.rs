use zircon_runtime::core::resource::ResourceId;

use super::*;

#[test]
fn machine_instance_key_separates_lineages_and_rejects_cycles() {
    let root_id = ResourceId::new();
    let child_id = ResourceId::new();
    let root = MachineInstanceKey::root(7, root_id);
    let child = root.nested("Locomotion", child_id).unwrap();
    let sibling = root.nested("Combat", child_id).unwrap();

    assert_ne!(root, child);
    assert_ne!(child, sibling);
    assert_eq!(child.entity(), 7);
    assert!(child.nested("Cycle", root_id).is_none());
}
