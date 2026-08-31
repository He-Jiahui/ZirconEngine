use std::str::FromStr;

use zircon_runtime_interface::ui::{
    design_tokens::EditorDesignTokens,
    layout::{
        UiAlign, UiDimension, UiFlexDirection, UiFlexWrap, UiGridLine, UiGridPlacement,
        UiGridTrack, UiGridTrackBreadth, UiJustify, UiLayoutDisplay, UiLayoutEngineFamily,
        UiOverflow, UiPositionMode, UiSlotKind,
    },
};

use super::{
    family_for_slot_kind, unsupported_viewport_unit, CssLikeConstraint, CssLikeConstraintError,
    CssLikeConstraintProperty, CssLikeDimension, CssLikeEdges, CssLikeGap, CssLikeGridTrack,
    CssLikeGridTrackBreadth, CssLikeOverflow, CssLikeSize,
};

#[test]
fn tokenized_flex_constraints_normalize_percentages_before_taffy_mapping() {
    let constraint = CssLikeConstraint {
        display: UiLayoutDisplay::Flex,
        direction: UiFlexDirection::Column,
        wrap: UiFlexWrap::NoWrap,
        gap: Some(CssLikeGap::uniform(
            CssLikeDimension::from_str("$gap.m").expect("gap token parses"),
        )),
        size: CssLikeSize {
            width: CssLikeDimension::from_str("50%").expect("percentage parses"),
            height: CssLikeDimension::Auto,
        },
        flex_grow: 1.0,
        ..CssLikeConstraint::default()
    };

    let style = constraint
        .into_layout_style(&EditorDesignTokens::workbench_dark())
        .expect("normalized constraint maps to the shared layout DTO");

    assert_eq!(constraint.family(), UiLayoutEngineFamily::Flex);
    assert_eq!(style.direction, UiFlexDirection::Column);
    assert_eq!(style.gap.row, UiDimension::Px(8.0));
    assert_eq!(style.gap.column, UiDimension::Px(8.0));
    assert_eq!(style.size.width, UiDimension::Percent(0.5));
    assert_eq!(style.flex_grow, 1.0);
}

#[test]
fn declarations_build_a_tokenized_flex_constraint_without_manual_field_assembly() {
    let constraint = CssLikeConstraint::from_declarations([
        ("display", "flex"),
        ("flex-direction", "column"),
        ("justify-content", "space-between"),
        ("align-items", "center"),
        ("gap", "$gap.m"),
        ("flex-grow", "1"),
        ("flex-basis", "50%"),
        ("min-width", "$--left-drawer-width"),
        ("padding", "$pad.s $pad.m"),
    ])
    .expect("supported CSS-like declarations build a constraint");

    let style = constraint
        .into_layout_style(&EditorDesignTokens::workbench_dark())
        .expect("the parsed constraint projects to the runtime DTO");

    assert_eq!(constraint.family(), UiLayoutEngineFamily::Flex);
    assert_eq!(style.direction, UiFlexDirection::Column);
    assert_eq!(style.justify_content, Some(UiJustify::SpaceBetween));
    assert_eq!(style.align_items, Some(UiAlign::Center));
    assert_eq!(style.gap.row, UiDimension::Px(8.0));
    assert_eq!(style.gap.column, UiDimension::Px(8.0));
    assert_eq!(style.flex_grow, 1.0);
    assert_eq!(style.flex_basis, UiDimension::Percent(0.5));
    assert_eq!(style.min_size.width, UiDimension::Px(332.0));
    assert_eq!(style.padding.top, UiDimension::Px(12.0));
    assert_eq!(style.padding.right, UiDimension::Px(16.0));
    assert_eq!(style.padding.bottom, UiDimension::Px(12.0));
    assert_eq!(style.padding.left, UiDimension::Px(16.0));
}

#[test]
fn declarations_keep_nested_grid_tracks_and_line_span_placement() {
    let constraint = CssLikeConstraint::from_declarations([
        ("display", "grid"),
        ("grid-template-columns", "minmax(120px, 1fr) 25% auto"),
        ("grid-template-rows", "auto minmax($gap.l, 2fr)"),
        ("grid-column", "2 / span 3"),
        ("grid-row", "auto / 4"),
    ])
    .expect("grid declarations build a constraint");

    let style = constraint
        .into_layout_style(&EditorDesignTokens::workbench_dark())
        .expect("the parsed grid constraint projects to the runtime DTO");

    assert_eq!(constraint.family(), UiLayoutEngineFamily::Grid);
    assert_eq!(
        style.grid_template_columns,
        vec![
            UiGridTrack::MinMax {
                min: UiGridTrackBreadth::Px(120.0),
                max: UiGridTrackBreadth::Fr(1.0),
            },
            UiGridTrack::Percent(0.25),
            UiGridTrack::Auto,
        ]
    );
    assert_eq!(
        style.grid_template_rows,
        vec![
            UiGridTrack::Auto,
            UiGridTrack::MinMax {
                min: UiGridTrackBreadth::Px(12.0),
                max: UiGridTrackBreadth::Fr(2.0),
            },
        ]
    );
    assert_eq!(
        style.grid_column,
        Some(UiGridPlacement {
            start: UiGridLine::Line(2),
            end: UiGridLine::Span(3),
        })
    );
    assert_eq!(
        style.grid_row,
        Some(UiGridPlacement {
            start: UiGridLine::Auto,
            end: UiGridLine::Line(4),
        })
    );
}

#[test]
fn declarations_reject_unknown_and_invalid_css_values_before_runtime_projection() {
    assert!(matches!(
        CssLikeConstraint::from_declarations([("grid-auto-flow", "row")]),
        Err(CssLikeConstraintError::KnownUnsupportedProperty {
            property: "grid-auto-flow"
        })
    ));
    assert!(matches!(
        CssLikeConstraint::from_declarations([("flex-direction", "diagonal")]),
        Err(CssLikeConstraintError::InvalidValue {
            property: "flex-direction",
            ..
        })
    ));
    assert_eq!(
        CssLikeConstraint::from_declarations([("padding", "auto")])
            .expect_err("padding auto must not enter the shared layout DTO"),
        CssLikeConstraintError::AutoNotAllowed {
            property: "padding"
        }
    );
    assert_eq!(
        CssLikeConstraint::from_declarations([("min-width", "auto")])
            .expect_err("explicit min-width auto is not a supported author value"),
        CssLikeConstraintError::AutoNotAllowed {
            property: "min-width"
        }
    );
    assert!(matches!(
        CssLikeConstraint::from_declarations([("grid-template-columns", "repeat(2, 1fr)")]),
        Err(CssLikeConstraintError::KnownUnsupportedSyntax {
            property: "grid-template",
            value: "repeat",
        })
    ));
    assert!(matches!(
        CssLikeConstraint::from_declarations([("overflow", "clip")]),
        Err(CssLikeConstraintError::KnownUnsupportedSyntax {
            property: "overflow",
            value: "clip",
        })
    ));
    assert!(matches!(
        CssLikeConstraint::from_declarations([("width", "10vw")]),
        Err(CssLikeConstraintError::KnownUnsupportedUnit { unit: "vw" })
    ));
    assert!(matches!(
        CssLikeConstraint::from_declarations([("grid-template-columns", "10vh")]),
        Err(CssLikeConstraintError::KnownUnsupportedUnit { unit: "vh" })
    ));
}

#[test]
fn declarations_apply_source_order_for_gap_shorthands() {
    let constraint = CssLikeConstraint::from_declarations([
        ("row-gap", "4px"),
        ("gap", "8px"),
        ("column-gap", "12px"),
    ])
    .expect("valid declarations apply in source order");
    let style = constraint
        .into_layout_style(&EditorDesignTokens::workbench_dark())
        .expect("source-order declaration result projects to the runtime DTO");

    assert_eq!(style.gap.row, UiDimension::Px(8.0));
    assert_eq!(style.gap.column, UiDimension::Px(12.0));
}

#[test]
fn declarations_normalize_box_position_and_overflow_shorthands() {
    let constraint = CssLikeConstraint::from_declarations([
        ("flex-wrap", "wrap-reverse"),
        ("align-self", "flex-end"),
        ("align-content", "stretch"),
        ("width", "240px"),
        ("height", "auto"),
        ("max-height", "80%"),
        ("aspect-ratio", "16 / 9"),
        ("margin", "auto 8px 12px"),
        ("inset", "10% auto"),
        ("top", "5px"),
        ("position", "absolute"),
        ("overflow", "hidden scroll"),
        ("overflow-x", "visible"),
    ])
    .expect("box, positioning, and overflow declarations build a constraint");

    let style = constraint
        .into_layout_style(&EditorDesignTokens::workbench_dark())
        .expect("the parsed box model projects to the runtime DTO");

    assert_eq!(style.wrap, UiFlexWrap::WrapReverse);
    assert_eq!(style.align_self, Some(UiAlign::End));
    assert_eq!(style.align_content, Some(UiAlign::Stretch));
    assert_eq!(style.size.width, UiDimension::Px(240.0));
    assert_eq!(style.size.height, UiDimension::Auto);
    assert_eq!(style.max_size.height, UiDimension::Percent(0.8));
    assert!((style.aspect_ratio.expect("aspect ratio is set") - 16.0 / 9.0).abs() < f32::EPSILON);
    assert_eq!(style.margin.top, UiDimension::Auto);
    assert_eq!(style.margin.right, UiDimension::Px(8.0));
    assert_eq!(style.margin.bottom, UiDimension::Px(12.0));
    assert_eq!(style.margin.left, UiDimension::Px(8.0));
    assert_eq!(style.position, UiPositionMode::Absolute);
    assert_eq!(style.inset.top, UiDimension::Px(5.0));
    assert_eq!(style.inset.right, UiDimension::Auto);
    assert_eq!(style.inset.bottom, UiDimension::Percent(0.1));
    assert_eq!(style.inset.left, UiDimension::Auto);
    assert_eq!(style.overflow.x, UiOverflow::Visible);
    assert_eq!(style.overflow.y, UiOverflow::Scroll);
}

#[test]
fn slot_kinds_select_their_declared_layout_family() {
    assert_eq!(
        family_for_slot_kind(UiSlotKind::Free),
        UiLayoutEngineFamily::Free
    );
    assert_eq!(
        family_for_slot_kind(UiSlotKind::Container),
        UiLayoutEngineFamily::Container
    );
    assert_eq!(
        family_for_slot_kind(UiSlotKind::Overlay),
        UiLayoutEngineFamily::Overlay
    );
    assert_eq!(
        family_for_slot_kind(UiSlotKind::Linear),
        UiLayoutEngineFamily::Flex
    );
    assert_eq!(
        family_for_slot_kind(UiSlotKind::Grid),
        UiLayoutEngineFamily::Grid
    );
    assert_eq!(
        family_for_slot_kind(UiSlotKind::Flow),
        UiLayoutEngineFamily::Wrap
    );
    assert_eq!(
        family_for_slot_kind(UiSlotKind::Canvas),
        UiLayoutEngineFamily::Canvas
    );
    assert_eq!(
        family_for_slot_kind(UiSlotKind::Scrollable),
        UiLayoutEngineFamily::Scrollable
    );
    assert_eq!(
        family_for_slot_kind(UiSlotKind::Splitter),
        UiLayoutEngineFamily::Flex
    );
    assert_eq!(
        family_for_slot_kind(UiSlotKind::Scale),
        UiLayoutEngineFamily::Container
    );
    for kind in [
        UiSlotKind::Free,
        UiSlotKind::Container,
        UiSlotKind::Overlay,
        UiSlotKind::Linear,
        UiSlotKind::Grid,
        UiSlotKind::Flow,
        UiSlotKind::Canvas,
        UiSlotKind::Scrollable,
        UiSlotKind::Splitter,
        UiSlotKind::Scale,
    ] {
        assert_eq!(family_for_slot_kind(kind), kind.layout_engine_family());
    }
}

#[test]
fn grid_tracks_keep_percent_fr_and_minmax_semantics_in_the_shared_dto() {
    let constraint = CssLikeConstraint {
        display: UiLayoutDisplay::Grid,
        grid_template_columns: vec![
            CssLikeGridTrack::Dimension(
                CssLikeDimension::from_str("25%").expect("percentage parses"),
            ),
            CssLikeGridTrack::Fr(1.0),
            CssLikeGridTrack::MinMax {
                min: CssLikeGridTrackBreadth::Dimension(
                    CssLikeDimension::from_str("120px").expect("pixel size parses"),
                ),
                max: CssLikeGridTrackBreadth::Fr(2.0),
            },
        ],
        ..CssLikeConstraint::default()
    };

    let style = constraint
        .into_layout_style(&EditorDesignTokens::workbench_dark())
        .expect("grid tracks map to the shared DTO");

    assert_eq!(constraint.family(), UiLayoutEngineFamily::Grid);
    assert_eq!(
        style.grid_template_columns,
        vec![
            UiGridTrack::Percent(0.25),
            UiGridTrack::Fr(1.0),
            UiGridTrack::MinMax {
                min: UiGridTrackBreadth::Px(120.0),
                max: UiGridTrackBreadth::Fr(2.0),
            },
        ]
    );
}

#[test]
fn block_constraints_keep_the_declared_display_and_family() {
    let constraint = CssLikeConstraint {
        display: UiLayoutDisplay::Block,
        ..CssLikeConstraint::default()
    };

    let style = constraint
        .into_layout_style(&EditorDesignTokens::workbench_dark())
        .expect("block constraints map to the shared DTO");

    assert_eq!(constraint.family(), UiLayoutEngineFamily::Block);
    assert_eq!(style.display, UiLayoutDisplay::Block);
}

#[test]
fn auto_margin_is_preserved_for_taffy_centering() {
    let constraint = CssLikeConstraint {
        margin: CssLikeEdges::all(CssLikeDimension::Auto),
        ..CssLikeConstraint::default()
    };

    let style = constraint
        .into_layout_style(&EditorDesignTokens::workbench_dark())
        .expect("margin auto is allowed by the shared layout DTO");

    assert_eq!(style.margin.left, UiDimension::Auto);
    assert_eq!(style.margin.right, UiDimension::Auto);
}

#[test]
fn gap_auto_is_rejected_instead_of_silently_reaching_taffy() {
    let constraint = CssLikeConstraint {
        gap: Some(CssLikeGap::uniform(CssLikeDimension::Auto)),
        ..CssLikeConstraint::default()
    };

    assert_eq!(
        constraint
            .into_layout_style(&EditorDesignTokens::workbench_dark())
            .expect_err("gap auto is outside the constraint language"),
        CssLikeConstraintError::AutoNotAllowed { property: "gap" }
    );
}

#[test]
fn padding_auto_is_rejected_instead_of_silently_reaching_taffy() {
    let constraint = CssLikeConstraint {
        padding: Some(CssLikeEdges::all(CssLikeDimension::Auto)),
        ..CssLikeConstraint::default()
    };

    assert_eq!(
        constraint
            .into_layout_style(&EditorDesignTokens::workbench_dark())
            .expect_err("padding auto is outside the constraint language"),
        CssLikeConstraintError::AutoNotAllowed {
            property: "padding"
        }
    );
}

#[test]
fn align_content_baseline_is_rejected_before_layout_backend_selection() {
    let constraint = CssLikeConstraint {
        align_content: Some(UiAlign::Baseline),
        ..CssLikeConstraint::default()
    };

    assert_eq!(
        constraint
            .into_layout_style(&EditorDesignTokens::workbench_dark())
            .expect_err("align-content baseline is unsupported by the shared DTO mapping"),
        CssLikeConstraintError::UnsupportedAlignment {
            property: "align-content",
            value: "baseline"
        }
    );
}

#[test]
fn t3_css_properties_produce_a_known_unsupported_diagnostic() {
    assert_eq!(
        CssLikeConstraintProperty::from_str("grid-auto-flow")
            .expect_err("T3 properties must not be silently accepted"),
        CssLikeConstraintError::KnownUnsupportedProperty {
            property: "grid-auto-flow"
        }
    );
    assert_eq!(
        CssLikeDimension::from_str("10vw")
            .expect_err("viewport units are a registered T3 extension candidate"),
        CssLikeConstraintError::KnownUnsupportedUnit { unit: "vw" }
    );
}

#[test]
fn t3_value_syntax_produces_a_known_unsupported_diagnostic() {
    assert_eq!(
        CssLikeGridTrack::from_author_value("repeat(auto-fit, 1fr)")
            .expect_err("repeat is not silently accepted as a grid track"),
        CssLikeConstraintError::KnownUnsupportedSyntax {
            property: "grid-template",
            value: "repeat"
        }
    );
    assert_eq!(
        CssLikeGridTrack::from_author_value("fit-content(120px)")
            .expect_err("fit-content is not silently accepted as a grid track"),
        CssLikeConstraintError::KnownUnsupportedSyntax {
            property: "grid-template",
            value: "fit-content"
        }
    );
    assert_eq!(
        CssLikeGridTrack::from_author_value("minmax(1fr, 2fr)")
            .expect_err("a fractional minimum would be silently downgraded by Taffy"),
        CssLikeConstraintError::KnownUnsupportedSyntax {
            property: "grid-template",
            value: "minmax-fr-minimum"
        }
    );
    assert_eq!(
        CssLikeOverflow::from_str("clip")
            .expect_err("overflow clip is a registered extension candidate"),
        CssLikeConstraintError::KnownUnsupportedSyntax {
            property: "overflow",
            value: "clip"
        }
    );
}

#[test]
fn percentages_outside_the_normalized_range_are_rejected() {
    assert!(matches!(
        CssLikeDimension::from_str("101%"),
        Err(CssLikeConstraintError::InvalidNumericValue {
            property: "percent",
            ..
        })
    ));
}

#[test]
fn non_finite_and_negative_pixel_dimensions_are_rejected_during_parsing() {
    for invalid in ["-1px", "NaNpx", "infpx"] {
        assert!(matches!(
            CssLikeDimension::from_str(invalid),
            Err(CssLikeConstraintError::InvalidNumericValue { property: "px", .. })
        ));
    }

    assert_eq!(
        CssLikeOverflow::from_str("scroll").expect("supported overflow parses"),
        CssLikeOverflow::Scroll
    );
    assert_eq!(CssLikeOverflow::Scroll.resolve(), UiOverflow::Scroll);
}

#[test]
fn optimization_batch_gq_editor429_viewport_unit_dispatch_preserves_rules() {
    for (value, unit) in [
        ("10vw", "vw"),
        ("10vh", "vh"),
        ("10vmin", "vmin"),
        ("10vmax", "vmax"),
    ] {
        assert_eq!(unsupported_viewport_unit(value), Some(unit));
        assert_eq!(
            CssLikeDimension::from_str(value),
            Err(CssLikeConstraintError::KnownUnsupportedUnit { unit })
        );
    }

    for value in ["", "10px", "preview", "10vmaxx"] {
        assert_eq!(unsupported_viewport_unit(value), None);
    }
}

#[test]
#[ignore = "release benchmark submitted to the validation coordinator"]
fn optimization_batch_gq_editor429_viewport_unit_suffix_dispatch_benchmark() {
    use std::hint::black_box;
    use std::time::Instant;

    const MARKER: &str = "EDITOR429_VIEWPORT_UNIT_SUFFIX_DISPATCH_BENCH_V1";
    const SAMPLES: usize = 31;
    const ITERATIONS: usize = 100_000;
    let value = "123456789vmax";
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    let mut legacy_samples = Vec::with_capacity(SAMPLES);

    for _ in 0..SAMPLES {
        let started_at = Instant::now();
        for _ in 0..ITERATIONS {
            assert_eq!(unsupported_viewport_unit(black_box(value)), Some("vmax"));
        }
        optimized_samples.push(started_at.elapsed().as_nanos() / ITERATIONS as u128);

        let started_at = Instant::now();
        for _ in 0..ITERATIONS {
            let value = black_box(value);
            let unit = ["vw", "vh", "vmin", "vmax"]
                .into_iter()
                .find(|unit| value.ends_with(unit));
            assert_eq!(unit, Some("vmax"));
        }
        legacy_samples.push(started_at.elapsed().as_nanos() / ITERATIONS as u128);
    }

    let optimized_p95_ns = p95(&mut optimized_samples);
    let legacy_p95_ns = p95(&mut legacy_samples);
    eprintln!(
        "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[samples.len().saturating_mul(95).div_ceil(100) - 1]
}
