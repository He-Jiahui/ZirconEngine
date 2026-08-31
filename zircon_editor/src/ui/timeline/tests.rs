use super::{
    build_timeline_ruler_ticks, keyframes_in_range, lane_kind_for_value, section_overlap_verdict,
    TimelineElementRef, TimelineKey, TimelineLaneKind, TimelineRange, TimelineSection,
    TimelineSectionOverlapPolicy, TimelineSelection, TimelineSnapSettings,
};

#[test]
fn ruler_uses_a_stable_readable_tick_interval() {
    let ticks = build_timeline_ruler_ticks(TimelineRange::new(0.0, 10.0), 1_000.0, 80.0);

    assert_eq!(ticks.first().map(|tick| tick.time), Some(0.0));
    assert_eq!(ticks.get(1).map(|tick| tick.time), Some(1.0));
    assert_eq!(ticks.last().map(|tick| tick.time), Some(10.0));
}

#[test]
fn snapping_uses_the_nearest_grid_or_authored_boundary_inside_threshold() {
    let range = TimelineRange::new(0.0, 2.0);
    let settings = TimelineSnapSettings::new(Some(0.25), 0.1);

    assert_eq!(settings.snap(0.46, range, &[0.5]), 0.5);
    assert_eq!(settings.snap(0.39, range, &[0.33]), 0.33);
    assert_eq!(settings.snap(2.5, range, &[]), 2.0);
}

#[test]
fn section_overlap_policy_allows_touching_boundaries_and_rejects_true_overlap_when_requested() {
    let existing = vec![TimelineSection::new(
        "existing",
        "Existing",
        TimelineRange::new(0.0, 5.0),
    )];
    let touching = TimelineSection::new("touching", "Touching", TimelineRange::new(5.0, 8.0));
    let overlapping = TimelineSection::new("overlap", "Overlap", TimelineRange::new(4.0, 6.0));

    assert!(
        section_overlap_verdict(&existing, &touching, TimelineSectionOverlapPolicy::Forbid,)
            .is_allowed()
    );
    assert!(section_overlap_verdict(
        &existing,
        &overlapping,
        TimelineSectionOverlapPolicy::Forbid,
    )
    .is_rejected());
    assert!(
        section_overlap_verdict(&existing, &overlapping, TimelineSectionOverlapPolicy::Allow,)
            .is_allowed()
    );
}

#[test]
fn selection_deduplicates_elements_across_tracks_and_preserves_typed_identity() {
    let first = TimelineElementRef::Key {
        track_id: "transform.position".to_string(),
        key_id: "x-12".to_string(),
    };
    let second = TimelineElementRef::Section {
        track_id: "animation.base".to_string(),
        section_id: "walk".to_string(),
    };
    let mut selection = TimelineSelection::default();

    assert!(selection.replace([first.clone(), second.clone(), first.clone()]));
    assert_eq!(selection.elements().len(), 2);
    assert!(selection.contains(&first));
    assert!(selection.contains(&second));
}

#[test]
fn keyframes_and_value_kinds_project_to_their_shared_lane_contracts() {
    let range = TimelineRange::new(0.25, 0.75);
    let keys = vec![
        TimelineKey::new("before", 0.0, "Before"),
        TimelineKey::new("inside", 0.5, "Inside"),
        TimelineKey::new("after", 1.0, "After"),
    ];

    assert_eq!(keyframes_in_range(&keys, range), vec![&keys[1]]);
    assert_eq!(lane_kind_for_value("float"), TimelineLaneKind::Curve);
    assert_eq!(lane_kind_for_value("vector3"), TimelineLaneKind::Curve);
    assert_eq!(lane_kind_for_value("integer"), TimelineLaneKind::Keyframe);
    assert_eq!(
        lane_kind_for_value("quaternion"),
        TimelineLaneKind::Keyframe
    );
    assert_eq!(lane_kind_for_value("event"), TimelineLaneKind::Event);
    assert_eq!(
        lane_kind_for_value("unregistered"),
        TimelineLaneKind::Keyframe
    );
}
