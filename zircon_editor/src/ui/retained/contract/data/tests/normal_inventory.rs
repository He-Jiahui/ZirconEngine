use super::super::HostPresentationGenerationCursor;
use super::*;
use crate::ui::template_runtime::NormalPresentInventory;
use std::sync::Arc;

fn cursor(values: [u64; 7]) -> HostPresentationGenerationCursor {
    HostPresentationGenerationCursor::new_with_geometry(
        values[0], values[1], values[2], values[3], values[4], values[5], values[6],
    )
}

#[test]
fn normal_inventory_publication_requires_a_successful_present_and_the_full_cursor() {
    let generations = [1, 2, 3, 4, 5, 6, 7];
    let stamp = cursor(generations);
    let presentation = HostWindowPresentationData {
        normal_arranged_inventory: Some((
            stamp,
            Arc::new(NormalPresentInventory::new(Vec::new())),
        )),
        ..Default::default()
    };
    assert!(presentation
        .normal_inventory_for_present(stamp, true)
        .is_some());
    assert!(presentation
        .normal_inventory_for_present(stamp, false)
        .is_none());
    for index in 0..7 {
        let mut changed = generations;
        changed[index] += 1;
        assert!(
            presentation
                .normal_inventory_for_present(cursor(changed), true)
                .is_none(),
            "cursor field {index}"
        );
    }
}

#[test]
fn geometry_publication_clears_inventory_without_mutating_the_old_snapshot() {
    let stamp = cursor([1, 2, 3, 4, 5, 6, 7]);
    let presentation = Arc::new(HostWindowPresentationData {
        normal_arranged_inventory: Some((
            stamp,
            Arc::new(NormalPresentInventory::new(Vec::new())),
        )),
        ..Default::default()
    });
    let geometry = HostWindowGeometryPresentationData::from_presentation(&presentation);
    let next = geometry.apply_to(&presentation);
    assert!(next.normal_arranged_inventory.is_none());
    assert!(presentation
        .normal_inventory_for_present(stamp, true)
        .is_some());
}
