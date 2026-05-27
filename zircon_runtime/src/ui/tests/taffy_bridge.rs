use crate::ui::layout::{taffy_display_for_family, taffy_style_for_container};
use taffy::style::{Dimension, Display, FlexDirection, FlexWrap, LengthPercentage};
use zircon_runtime_interface::ui::layout::{
    AxisConstraint, BoxConstraints, UiContainerKind, UiGridBoxConfig, UiLayoutEngineBackend,
    UiLayoutEngineCapability, UiLayoutEngineFallbackReason, UiLayoutEngineFamily,
    UiLayoutEngineRequest, UiLayoutEngineSelection, UiLayoutEngineSupport, UiLinearBoxConfig,
    UiMasonryBoxConfig, UiScrollableBoxConfig, UiSizeBoxConfig, UiVirtualListConfig,
    UiWrapBoxConfig,
};

#[test]
fn taffy_bridge_maps_horizontal_vertical_grid_and_wrap_families() {
    let horizontal = taffy_style_for_container(
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 6.0 }),
        sample_constraints(),
    )
    .expect("horizontal flex should be taffy-owned");
    assert_eq!(horizontal.display, Display::Flex);
    assert_eq!(horizontal.flex_direction, FlexDirection::Row);
    assert_eq!(horizontal.gap.width, LengthPercentage::length(6.0));
    assert_eq!(horizontal.size.width, Dimension::length(320.0));
    assert_eq!(horizontal.min_size.width, Dimension::length(80.0));
    assert_eq!(horizontal.max_size.width, Dimension::length(640.0));

    let vertical = taffy_style_for_container(
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 8.0 }),
        sample_constraints(),
    )
    .expect("vertical flex should be taffy-owned");
    assert_eq!(vertical.display, Display::Flex);
    assert_eq!(vertical.flex_direction, FlexDirection::Column);
    assert_eq!(vertical.gap.height, LengthPercentage::length(8.0));

    let grid = taffy_style_for_container(
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 3,
            rows: 2,
            column_gap: 10.0,
            row_gap: 12.0,
        }),
        sample_constraints(),
    )
    .expect("grid should be taffy-owned");
    assert_eq!(grid.display, Display::Grid);
    assert_eq!(grid.gap.width, LengthPercentage::length(10.0));
    assert_eq!(grid.gap.height, LengthPercentage::length(12.0));

    let wrap = taffy_style_for_container(
        UiContainerKind::WrapBox(UiWrapBoxConfig {
            horizontal_gap: 4.0,
            vertical_gap: 5.0,
            item_min_width: 96.0,
        }),
        sample_constraints(),
    )
    .expect("wrap should be taffy-owned");
    assert_eq!(wrap.display, Display::Flex);
    assert_eq!(wrap.flex_wrap, FlexWrap::Wrap);
    assert_eq!(wrap.gap.width, LengthPercentage::length(4.0));
    assert_eq!(wrap.gap.height, LengthPercentage::length(5.0));
}

#[test]
fn taffy_bridge_rejects_zircon_owned_overlay_scroll_size_box_and_virtual_list_semantics() {
    assert!(taffy_style_for_container(UiContainerKind::Overlay, sample_constraints()).is_none());
    assert!(taffy_style_for_container(
        UiContainerKind::SizeBox(UiSizeBoxConfig { aspect_ratio: 1.0 }),
        sample_constraints(),
    )
    .is_none());
    assert!(taffy_style_for_container(
        UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
            axis: Default::default(),
            gap: 2.0,
            scrollbar_visibility: Default::default(),
            virtualization: None,
        }),
        sample_constraints(),
    )
    .is_none());
    assert!(taffy_style_for_container(
        UiContainerKind::MasonryBox(UiMasonryBoxConfig {
            columns: 3,
            gap: 8.0,
            sequential: true,
        }),
        sample_constraints(),
    )
    .is_none());
    assert!(taffy_style_for_container(
        UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
            axis: Default::default(),
            gap: 2.0,
            scrollbar_visibility: Default::default(),
            virtualization: Some(UiVirtualListConfig {
                item_extent: 24.0,
                overscan: 2,
            }),
        }),
        sample_constraints(),
    )
    .is_none());

    let selection = UiLayoutEngineSelection::select(
        &UiLayoutEngineRequest::from_container_kind(UiContainerKind::Overlay),
        &UiLayoutEngineCapability::taffy_flex_grid_block(),
        &UiLayoutEngineCapability::legacy_zircon(),
    );
    assert_eq!(selection.requested_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(
        selection.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(selection.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        selection.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );

    let masonry_selection = UiLayoutEngineSelection::select(
        &UiLayoutEngineRequest::from_container_kind(UiContainerKind::MasonryBox(
            UiMasonryBoxConfig {
                columns: 3,
                gap: 8.0,
                sequential: true,
            },
        )),
        &UiLayoutEngineCapability::taffy_flex_grid_block(),
        &UiLayoutEngineCapability::legacy_zircon(),
    );
    assert_eq!(
        masonry_selection.request.family,
        UiLayoutEngineFamily::Masonry
    );
    assert_eq!(
        masonry_selection.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(
        masonry_selection.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
}

#[test]
fn taffy_bridge_rejects_non_finite_style_inputs() {
    assert!(taffy_style_for_container(
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: f32::INFINITY }),
        sample_constraints(),
    )
    .is_none());
    assert!(taffy_style_for_container(
        UiContainerKind::WrapBox(UiWrapBoxConfig {
            horizontal_gap: 4.0,
            vertical_gap: 5.0,
            item_min_width: f32::NAN,
        }),
        sample_constraints(),
    )
    .is_none());
    assert!(taffy_style_for_container(
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 3,
            rows: 2,
            column_gap: 10.0,
            row_gap: f32::INFINITY,
        }),
        sample_constraints(),
    )
    .is_none());

    let mut constraints = sample_constraints();
    constraints.width.preferred = f32::INFINITY;
    assert!(taffy_style_for_container(
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 8.0 }),
        constraints,
    )
    .is_none());
}

#[test]
fn taffy_bridge_keeps_block_display_explicit_and_container_zircon_owned() {
    assert_eq!(
        taffy_display_for_family(UiLayoutEngineFamily::Block),
        Some(Display::Block)
    );
    assert_eq!(
        taffy_display_for_family(UiLayoutEngineFamily::Container),
        None
    );
    assert!(taffy_style_for_container(UiContainerKind::Container, sample_constraints()).is_none());

    let selection = UiLayoutEngineSelection::select(
        &UiLayoutEngineRequest::from_container_kind(UiContainerKind::Container),
        &UiLayoutEngineCapability::taffy_flex_grid_block(),
        &UiLayoutEngineCapability::legacy_zircon(),
    );
    assert_eq!(
        selection.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(
        selection.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
}

fn sample_constraints() -> BoxConstraints {
    BoxConstraints {
        width: AxisConstraint {
            min: 80.0,
            preferred: 320.0,
            max: 640.0,
            ..AxisConstraint::default()
        },
        height: AxisConstraint {
            min: 40.0,
            preferred: 180.0,
            max: 360.0,
            ..AxisConstraint::default()
        },
    }
}
