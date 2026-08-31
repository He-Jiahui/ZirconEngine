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
fn frame_update_only_runs_tick_without_requesting_a_present() {
    let redraw = HostRedrawRequest::frame_update_only_for_scenario(UiPerfScenario::AssetRefresh);

    assert!(redraw.request_redraw());
    assert!(redraw.requires_frame_update());
    assert!(!redraw.requires_present());
    assert_eq!(redraw.damage_region(), None);
    assert_eq!(redraw.scenario(), UiPerfScenario::AssetRefresh);
}

#[test]
fn visual_damage_absorbs_a_frame_update_only_request() {
    let frame = FrameRect {
        x: 8.0,
        y: 10.0,
        width: 20.0,
        height: 12.0,
    };
    let redraw = HostRedrawRequest::region_for_scenario(UiPerfScenario::Click, frame.clone())
        .merge(HostRedrawRequest::frame_update_only_for_scenario(
            UiPerfScenario::AssetRefresh,
        ));

    assert!(redraw.requires_frame_update());
    assert!(redraw.requires_present());
    assert_eq!(redraw.damage_region(), Some(&frame));
    assert_eq!(redraw.scenario(), UiPerfScenario::AssetRefresh);
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
fn interactive_frame_update_authority_survives_damage_and_maintenance_merge() {
    let interactive = HostRedrawRequest::region_with_frame_update(FrameRect {
        x: 24.0,
        y: 6.0,
        width: 10.0,
        height: 20.0,
    })
    .into_interactive_frame_update();
    let redraw = HostRedrawRequest::region(FrameRect {
        x: 8.0,
        y: 10.0,
        width: 20.0,
        height: 12.0,
    })
    .merge(interactive)
    .merge(HostRedrawRequest::frame_update_only_for_scenario(
        UiPerfScenario::AssetRefresh,
    ));

    assert!(redraw.requires_frame_update());
    assert!(redraw.prefers_interactive_frame_update());
    assert!(redraw.requires_present());
}

#[test]
fn pure_damage_and_present_retries_do_not_claim_interactive_frame_updates() {
    let damage = HostRedrawRequest::region(FrameRect {
        x: 8.0,
        y: 10.0,
        width: 20.0,
        height: 12.0,
    })
    .into_interactive_frame_update();
    assert!(!damage.prefers_interactive_frame_update());

    let retry = HostRedrawRequest::region_with_frame_update(FrameRect {
        x: 8.0,
        y: 10.0,
        width: 20.0,
        height: 12.0,
    })
    .into_interactive_frame_update()
    .into_present_retry(UiPerfScenario::Click);
    assert!(!retry.requires_frame_update());
    assert!(!retry.prefers_interactive_frame_update());
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

#[test]
fn redraw_region_retains_distant_damage_before_presenting_the_same_bounding_frame() {
    let redraw = HostRedrawRequest::region(FrameRect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    })
    .merge(HostRedrawRequest::region(FrameRect {
        x: 100.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    }));

    assert_eq!(
        redraw.damage_region(),
        Some(&FrameRect {
            x: 0.0,
            y: 0.0,
            width: 110.0,
            height: 10.0,
        })
    );
    assert_eq!(
        redraw.damage_region_metrics(),
        Some(HostDamageRegionMetrics {
            rect_count: 2,
            source_rect_count: 2,
            simplification_count: 0,
            represented_area: 200.0,
            bounding_area: 1_100.0,
            bounding_overdraw_area: 900.0,
        })
    );
}

#[test]
fn redraw_region_eliminates_contained_damage_without_losing_source_pressure() {
    let redraw = HostRedrawRequest::region(FrameRect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    })
    .merge(HostRedrawRequest::region(FrameRect {
        x: 2.0,
        y: 2.0,
        width: 2.0,
        height: 2.0,
    }));

    assert_eq!(
        redraw.damage_region_metrics(),
        Some(HostDamageRegionMetrics {
            rect_count: 1,
            source_rect_count: 2,
            simplification_count: 0,
            represented_area: 100.0,
            bounding_area: 100.0,
            bounding_overdraw_area: 0.0,
        })
    );
}

#[test]
fn redraw_region_simplifies_the_fourth_rect_by_least_added_area() {
    let redraw = HostRedrawRequest::region(FrameRect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    })
    .merge(HostRedrawRequest::region(FrameRect {
        x: 20.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    }))
    .merge(HostRedrawRequest::region(FrameRect {
        x: 100.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    }))
    .merge(HostRedrawRequest::region(FrameRect {
        x: 32.0,
        y: 0.0,
        width: 8.0,
        height: 10.0,
    }));

    assert_eq!(
        redraw.damage_region_metrics(),
        Some(HostDamageRegionMetrics {
            rect_count: 3,
            source_rect_count: 4,
            simplification_count: 1,
            represented_area: 400.0,
            bounding_area: 1_100.0,
            bounding_overdraw_area: 700.0,
        })
    );
}

#[test]
fn redraw_region_reports_exact_overlap_area_for_the_bounded_representation() {
    let redraw = HostRedrawRequest::region(FrameRect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    })
    .merge(HostRedrawRequest::region(FrameRect {
        x: 5.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    }))
    .merge(HostRedrawRequest::region(FrameRect {
        x: 0.0,
        y: 5.0,
        width: 10.0,
        height: 10.0,
    }));

    let metrics = redraw
        .damage_region_metrics()
        .expect("region redraw should publish bounded damage metrics");
    assert_eq!(metrics.rect_count, 3);
    assert_eq!(metrics.source_rect_count, 3);
    assert_eq!(metrics.simplification_count, 0);
    assert_eq!(metrics.represented_area, 200.0);
    assert_eq!(metrics.bounding_area, 225.0);
    assert_eq!(metrics.bounding_overdraw_area, 25.0);
}

#[test]
fn redraw_region_preserves_the_legacy_f32_bounding_merge_order_after_simplification() {
    let redraw = HostRedrawRequest::region(FrameRect {
        x: 5_596.247_6,
        y: -1_638.452_5,
        width: 474.118_01,
        height: 852.153_5,
    })
    .merge(HostRedrawRequest::region(FrameRect {
        x: -2_750.721_7,
        y: -369.854_98,
        width: 91.951_256,
        height: 749.147_2,
    }))
    .merge(HostRedrawRequest::region(FrameRect {
        x: 6_684.028_3,
        y: -2_837.741_2,
        width: 178.859_07,
        height: 981.773_5,
    }))
    .merge(HostRedrawRequest::region(FrameRect {
        x: -7_558.676_3,
        y: -904.441_6,
        width: 456.333_5,
        height: 108.017_91,
    }));

    assert_eq!(
        redraw.damage_region(),
        Some(&FrameRect {
            x: -7_558.676_3,
            y: -2_837.741_2,
            width: 14_421.564,
            height: 3_217.033_4,
        })
    );
    assert_eq!(
        redraw
            .damage_region_metrics()
            .expect("region metrics")
            .simplification_count,
        1
    );
}
