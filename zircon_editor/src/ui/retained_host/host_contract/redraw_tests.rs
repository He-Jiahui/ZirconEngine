use super::*;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

#[test]
fn redraw_region_merge_unions_damage_without_frame_update() {
    let redraw = HostRedrawRequest::region(FrameRect {
        x: 8.0,
        y: 10.0,
        width: 20.0,
        height: 12.0,
    })
    .merge(HostRedrawRequest::region(FrameRect {
        x: 24.0,
        y: 6.0,
        width: 10.0,
        height: 20.0,
    }));

    assert_eq!(
        redraw.damage_region(),
        Some(&FrameRect {
            x: 8.0,
            y: 6.0,
            width: 26.0,
            height: 20.0,
        })
    );
    assert!(!redraw.requires_frame_update());
}

#[test]
fn redraw_full_merge_overrides_region_and_preserves_frame_update() {
    let redraw = HostRedrawRequest::region(FrameRect {
        x: 8.0,
        y: 10.0,
        width: 20.0,
        height: 12.0,
    })
    .merge(HostRedrawRequest::full_frame_for_scenario(
        UiPerfScenario::Startup,
        true,
    ));

    assert!(redraw.request_redraw());
    assert!(redraw.requires_frame_update());
    assert_eq!(redraw.damage_region(), None);
}

#[test]
fn redraw_region_can_request_frame_update_without_losing_damage() {
    let redraw = HostRedrawRequest::region_with_frame_update(FrameRect {
        x: 4.0,
        y: 8.0,
        width: 80.0,
        height: 28.0,
    });

    assert!(redraw.request_redraw());
    assert!(redraw.requires_frame_update());
    assert_eq!(
        redraw.damage_region(),
        Some(&FrameRect {
            x: 4.0,
            y: 8.0,
            width: 80.0,
            height: 28.0,
        })
    );
}

#[test]
fn redraw_region_merge_preserves_frame_update_request() {
    let redraw = HostRedrawRequest::region(FrameRect {
        x: 8.0,
        y: 10.0,
        width: 20.0,
        height: 12.0,
    })
    .merge(HostRedrawRequest::region_with_frame_update(FrameRect {
        x: 24.0,
        y: 6.0,
        width: 10.0,
        height: 20.0,
    }));

    assert!(redraw.requires_frame_update());
    assert_eq!(
        redraw.damage_region(),
        Some(&FrameRect {
            x: 8.0,
            y: 6.0,
            width: 26.0,
            height: 20.0,
        })
    );
}

#[test]
fn redraw_merge_uses_latest_frame_update_scenario() {
    let click_frame = FrameRect {
        x: 8.0,
        y: 10.0,
        width: 20.0,
        height: 12.0,
    };
    let resize_frame = FrameRect {
        x: 24.0,
        y: 6.0,
        width: 10.0,
        height: 20.0,
    };

    let redraw = HostRedrawRequest::region_for_scenario_with_frame_update(
        UiPerfScenario::Click,
        click_frame.clone(),
        true,
    )
    .merge(HostRedrawRequest::region_for_scenario_with_frame_update(
        UiPerfScenario::DrawerResize,
        resize_frame.clone(),
        true,
    ));

    assert!(redraw.requires_frame_update());
    assert_eq!(redraw.scenario(), UiPerfScenario::DrawerResize);

    let redraw = HostRedrawRequest::full_frame_for_scenario(UiPerfScenario::Click, true).merge(
        HostRedrawRequest::region_for_scenario_with_frame_update(
            UiPerfScenario::AssetRefresh,
            resize_frame,
            true,
        ),
    );

    assert!(redraw.requires_frame_update());
    assert_eq!(redraw.scenario(), UiPerfScenario::AssetRefresh);
}
