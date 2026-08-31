use std::sync::Arc;

use crate::core::framework::window::{
    DisplayId, DisplayKind, DisplayTopologyGeneration, DisplayTopologyReplacementError,
};
use crate::platform::test_support::platform_driver;

use super::fixtures::display_topology;

#[test]
fn display_topology_publish_is_atomic_and_reader_snapshots_remain_immutable() {
    let driver = platform_driver();
    let initial = driver.display_topology_snapshot();
    assert_eq!(initial.generation().get(), 1);
    assert!(initial.is_empty());

    let replacement = driver
        .publish_display_topology(display_topology(2, "edid:panel-a"))
        .expect("newer topology publishes");
    let current = driver.display_topology_snapshot();

    assert_eq!(replacement.previous_generation().get(), 1);
    assert_eq!(replacement.current_generation().get(), 2);
    assert_eq!(
        replacement.added(),
        &[DisplayId::new(DisplayKind::PhysicalOutput, "edid:panel-a").unwrap()]
    );
    assert_eq!(current.generation().get(), 2);
    assert_eq!(current.len(), 1);
    assert_eq!(initial.generation().get(), 1);
    assert!(initial.is_empty());
    assert!(!Arc::ptr_eq(&initial, &current));
}

#[test]
fn rejected_stale_topology_leaves_the_current_snapshot_published() {
    let driver = platform_driver();
    driver
        .publish_display_topology(display_topology(2, "edid:panel-a"))
        .expect("first topology publishes");

    assert_eq!(
        driver.publish_display_topology(display_topology(2, "edid:panel-b")),
        Err(DisplayTopologyReplacementError::GenerationNotAdvanced {
            previous: DisplayTopologyGeneration::new(2).expect("generation is nonzero"),
            current: DisplayTopologyGeneration::new(2).expect("generation is nonzero"),
        })
    );

    let current = driver.display_topology_snapshot();
    assert_eq!(current.generation().get(), 2);
    assert!(current.contains(&DisplayId::new(DisplayKind::PhysicalOutput, "edid:panel-a").unwrap()));
    assert!(
        !current.contains(&DisplayId::new(DisplayKind::PhysicalOutput, "edid:panel-b").unwrap())
    );
}
