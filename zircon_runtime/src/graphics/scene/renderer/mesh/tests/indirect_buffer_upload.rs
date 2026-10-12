use super::changed_element_ranges;

#[test]
fn stable_values_produce_no_dirty_ranges() {
    assert!(changed_element_ranges(&[1_u32, 2, 3], &[1, 2, 3], false).is_empty());
}

#[test]
fn changed_ranges_cover_growth_and_contiguous_changes() {
    assert_eq!(
        changed_element_ranges(&[1_u32, 2, 3], &[1, 7, 8, 9, 10], false),
        vec![1..5]
    );
}

#[test]
fn shrink_only_updates_the_committed_shadow_without_gpu_writes() {
    assert!(changed_element_ranges(&[1_u32, 2, 3], &[1, 2], false).is_empty());
}

#[test]
fn interleaved_changes_remain_exact_until_profile_justifies_coalescing() {
    assert_eq!(
        changed_element_ranges(&[1_u32, 2, 3, 4, 5], &[9, 2, 8, 4, 7], false),
        vec![0..1, 2..3, 4..5]
    );
}

#[test]
fn forced_upload_covers_the_complete_non_empty_payload() {
    assert_eq!(
        changed_element_ranges(&[1_u32, 2], &[1, 2, 3], true),
        vec![0..3]
    );
    assert!(changed_element_ranges::<u32>(&[1, 2], &[], true).is_empty());
}
