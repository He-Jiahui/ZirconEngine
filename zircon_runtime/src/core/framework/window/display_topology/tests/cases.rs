use std::num::{NonZeroU32, NonZeroU64};

use super::{
    DisplayColorSpace, DisplayFeatureState, DisplayId, DisplayIdentityError, DisplayKind,
    DisplayLogicalInsets, DisplayLogicalRect, DisplayObservation, DisplayOrientation,
    DisplayOutputCapabilities, DisplayPhysicalRect, DisplaySnapshot, DisplayTopologyError,
    DisplayTopologyGeneration, DisplayTopologyReplacementError, DisplayTopologySnapshot,
};

fn display_id(value: &str) -> DisplayId {
    DisplayId::new(DisplayKind::PhysicalOutput, value).expect("display fixture identity is valid")
}

fn snapshot(id: &str) -> DisplaySnapshot {
    DisplaySnapshot::new(
        display_id(id),
        DisplayObservation {
            physical_bounds: DisplayPhysicalRect::new(
                0,
                0,
                NonZeroU32::new(3840).expect("fixture width is nonzero"),
                NonZeroU32::new(2160).expect("fixture height is nonzero"),
            ),
            usable_logical_bounds: DisplayLogicalRect::new(0.0, 0.0, 1920.0, 1080.0)
                .expect("fixture logical geometry is valid"),
            scale_factor: 2.0,
            refresh_rate_millihertz: NonZeroU32::new(60_000),
            orientation: DisplayOrientation::Landscape,
            safe_area: Some(
                DisplayLogicalInsets::new(0.0, 24.0, 0.0, 0.0).expect("fixture safe area is valid"),
            ),
            output_capabilities: DisplayOutputCapabilities {
                hdr: DisplayFeatureState::Available,
                variable_refresh_rate: DisplayFeatureState::Unknown,
                wide_color_gamut: DisplayFeatureState::Available,
                color_space: DisplayColorSpace::DisplayP3,
            },
        },
    )
    .expect("fixture display snapshot is valid")
}

fn generation(value: u64) -> DisplayTopologyGeneration {
    DisplayTopologyGeneration::new(value).expect("topology fixture generation is nonzero")
}

#[test]
fn identity_is_stable_keyed_and_bounded() {
    assert_eq!(
        DisplayId::new(DisplayKind::PhysicalOutput, " \t "),
        Err(DisplayIdentityError::Empty),
        "whitespace cannot become a stable display key"
    );
    let overlong = "a".repeat(513);
    assert_eq!(
        DisplayId::new(DisplayKind::PhysicalOutput, overlong),
        Err(DisplayIdentityError::TooLong {
            maximum_bytes: 512,
            actual_bytes: 513,
        })
    );
    let physical = display_id("edid:panel-a");
    let logical = DisplayId::new(DisplayKind::LogicalScreen, "edid:panel-a")
        .expect("logical display fixture identity is valid");
    assert_eq!(physical.as_str(), "edid:panel-a");
    assert_eq!(physical.kind(), DisplayKind::PhysicalOutput);
    assert_ne!(
        physical, logical,
        "display domains must not alias by raw key"
    );
    assert_eq!(physical.to_string(), "physical_output:edid:panel-a");
}

#[test]
fn snapshot_resolves_primary_and_secondary_displays_without_index_routing() {
    let primary = snapshot("edid:panel-a");
    let secondary = snapshot("connector:display-port-1");
    let primary_id = primary.id().clone();
    let secondary_id = secondary.id().clone();
    let topology = DisplayTopologySnapshot::new(
        generation(9),
        vec![primary, secondary],
        Some(primary_id.clone()),
    )
    .expect("valid topology publishes atomically");

    assert_eq!(topology.generation().get(), 9);
    assert_eq!(topology.len(), 2);
    assert_eq!(topology.primary_display_id(), Some(&primary_id));
    assert_eq!(topology.primary_display().unwrap().id(), &primary_id);
    assert_eq!(topology.get(&secondary_id).unwrap().scale_factor(), 2.0);
    assert!(topology.contains(&secondary_id));
    assert_eq!(topology.displays().len(), 2);
}

#[test]
fn topology_rejects_duplicate_and_unknown_primary_without_publishing_partial_index() {
    let duplicate = snapshot("edid:panel-a");
    let error = DisplayTopologySnapshot::new(
        generation(1),
        vec![snapshot("edid:panel-a"), duplicate],
        None,
    )
    .expect_err("duplicate backend identities must not be addressable by position");
    assert_eq!(
        error,
        DisplayTopologyError::DuplicateDisplay {
            display: display_id("edid:panel-a"),
        }
    );

    let error = DisplayTopologySnapshot::new(
        generation(2),
        vec![snapshot("edid:panel-a")],
        Some(display_id("edid:missing")),
    )
    .expect_err("primary selection must resolve inside the same snapshot generation");
    assert_eq!(
        error,
        DisplayTopologyError::UnknownPrimaryDisplay {
            display: display_id("edid:missing"),
        }
    );
}

#[test]
fn invalid_observed_geometry_and_scale_fail_instead_of_becoming_default_values() {
    assert_eq!(
        DisplayLogicalRect::new(0.0, 0.0, f64::NAN, 100.0),
        Err(DisplayTopologyError::NonFiniteLogicalGeometry)
    );
    assert_eq!(
        DisplayLogicalInsets::new(0.0, -1.0, 0.0, 0.0),
        Err(DisplayTopologyError::NegativeSafeAreaInsets)
    );

    let id = display_id("edid:panel-a");
    let error = DisplaySnapshot::new(
        id.clone(),
        DisplayObservation {
            physical_bounds: DisplayPhysicalRect::new(0, 0, NonZeroU32::MIN, NonZeroU32::MIN),
            usable_logical_bounds: DisplayLogicalRect::new(0.0, 0.0, 1.0, 1.0)
                .expect("positive logical extent"),
            scale_factor: 0.0,
            refresh_rate_millihertz: None,
            orientation: DisplayOrientation::Unknown,
            safe_area: None,
            output_capabilities: DisplayOutputCapabilities::default(),
        },
    )
    .expect_err("zero scale must not be silently normalized");
    assert_eq!(
        error,
        DisplayTopologyError::NonPositiveScaleFactor {
            display: id,
            scale_factor: 0.0,
        }
    );

    let id = display_id("edid:panel-safe-area");
    let error = DisplaySnapshot::new(
        id.clone(),
        DisplayObservation {
            physical_bounds: DisplayPhysicalRect::new(0, 0, NonZeroU32::MIN, NonZeroU32::MIN),
            usable_logical_bounds: DisplayLogicalRect::new(0.0, 0.0, 100.0, 100.0)
                .expect("positive logical extent"),
            scale_factor: 1.0,
            refresh_rate_millihertz: None,
            orientation: DisplayOrientation::Unknown,
            safe_area: Some(
                DisplayLogicalInsets::new(51.0, 0.0, 50.0, 0.0).expect("nonnegative fixture inset"),
            ),
            output_capabilities: DisplayOutputCapabilities::default(),
        },
    )
    .expect_err("safe area must fit the observed usable bounds");
    assert_eq!(
        error,
        DisplayTopologyError::SafeAreaExceedsUsableBounds { display: id }
    );
}

#[test]
fn topology_generation_cannot_be_zero() {
    assert_eq!(DisplayTopologyGeneration::new(0), None);
    assert_eq!(generation(NonZeroU64::MIN.get()).get(), 1);
}

#[test]
fn topology_replacement_reports_hotplug_changes_in_stable_snapshot_order() {
    let primary = display_id("edid:panel-a");
    let previous = DisplayTopologySnapshot::new(
        generation(4),
        vec![snapshot("edid:panel-a"), snapshot("connector:hdmi-a")],
        Some(primary.clone()),
    )
    .expect("previous topology is valid");
    let changed_primary = DisplaySnapshot::new(
        primary.clone(),
        DisplayObservation {
            physical_bounds: DisplayPhysicalRect::new(
                0,
                0,
                NonZeroU32::new(3840).expect("fixture width is nonzero"),
                NonZeroU32::new(2160).expect("fixture height is nonzero"),
            ),
            usable_logical_bounds: DisplayLogicalRect::new(0.0, 0.0, 1536.0, 864.0)
                .expect("fixture logical geometry is valid"),
            scale_factor: 2.0,
            refresh_rate_millihertz: NonZeroU32::new(60_000),
            orientation: DisplayOrientation::Landscape,
            safe_area: None,
            output_capabilities: DisplayOutputCapabilities::default(),
        },
    )
    .expect("changed primary snapshot is valid");
    let current = DisplayTopologySnapshot::new(
        generation(5),
        vec![changed_primary, snapshot("connector:display-port-1")],
        Some(display_id("connector:display-port-1")),
    )
    .expect("current topology is valid");

    let replacement = current
        .replacement_from(&previous)
        .expect("generation-five snapshot replaces generation four");
    assert_eq!(replacement.previous_generation(), generation(4));
    assert_eq!(replacement.current_generation(), generation(5));
    assert_eq!(
        replacement.added(),
        &[display_id("connector:display-port-1")]
    );
    assert_eq!(replacement.changed(), &[primary]);
    assert_eq!(replacement.removed(), &[display_id("connector:hdmi-a")]);
    assert!(replacement.primary_changed());
    assert!(!replacement.is_empty());
}

#[test]
fn topology_replacement_requires_a_strictly_new_generation() {
    let previous = DisplayTopologySnapshot::new(
        generation(7),
        vec![snapshot("edid:panel-a")],
        Some(display_id("edid:panel-a")),
    )
    .expect("previous topology is valid");
    let stale = DisplayTopologySnapshot::new(
        generation(7),
        vec![snapshot("edid:panel-a")],
        Some(display_id("edid:panel-a")),
    )
    .expect("stale topology shape is otherwise valid");

    assert_eq!(
        stale.replacement_from(&previous),
        Err(DisplayTopologyReplacementError::GenerationNotAdvanced {
            previous: generation(7),
            current: generation(7),
        })
    );
}
