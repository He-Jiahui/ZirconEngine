use super::HostRedrawRequest;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

#[test]
fn committed_viewport_damage_presents_without_consuming_the_next_runtime_tick() {
    let region = HostRedrawRequest::region(FrameRect {
        x: 20.0,
        y: 40.0,
        width: 100.0,
        height: 80.0,
    });
    let expected = region.damage_region_metrics();
    for request in [
        region.clone().merge(HostRedrawRequest::runtime_tick()),
        HostRedrawRequest::runtime_tick().merge(region.clone()),
    ] {
        let (work, ready) = request.split_ready_present();
        assert_eq!(work, HostRedrawRequest::runtime_tick());
        assert!(ready.requires_present());
        assert!(!ready.requires_runtime_tick());
        assert!(!ready.requires_frame_update());
        assert_eq!(ready.damage_region_metrics(), expected);
    }
}

#[test]
fn uncommitted_geometry_keeps_its_damage_and_interactive_update_together() {
    for request in [
        HostRedrawRequest::full_frame_for_scenario(UiPerfScenario::WindowResize, true),
        HostRedrawRequest::region_with_frame_update(FrameRect {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 200.0,
        }),
    ] {
        let request = request.into_interactive_frame_update();
        let expected = request.clone();
        let (work, ready) = request.split_ready_present();
        assert_eq!(work, expected);
        assert_eq!(ready, HostRedrawRequest::none());
    }
}

#[test]
fn maintenance_without_visual_changes_has_no_present() {
    for request in [
        HostRedrawRequest::none(),
        HostRedrawRequest::runtime_tick(),
        HostRedrawRequest::frame_update_only_for_scenario(UiPerfScenario::SessionHeartbeat),
    ] {
        let expected = request.clone();
        let (work, ready) = request.split_ready_present();
        assert_eq!(work, expected);
        assert!(!ready.requires_present());
    }
}

#[test]
fn runtime_demand_merged_with_interactive_work_keeps_the_runtime_pump() {
    for request in [
        HostRedrawRequest::frame_update_only_for_scenario(UiPerfScenario::DrawerResize),
        HostRedrawRequest::full_frame_for_scenario(UiPerfScenario::WindowResize, true),
        HostRedrawRequest::region_with_frame_update(FrameRect {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 200.0,
        }),
    ] {
        let request = request.into_interactive_frame_update();
        for merged in [
            request.clone().merge(HostRedrawRequest::runtime_tick()),
            HostRedrawRequest::runtime_tick().merge(request),
        ] {
            assert!(merged.requires_frame_update());
            assert!(!merged.prefers_interactive_frame_update());
        }
    }
}

#[test]
fn runtime_demand_survives_multiple_interactive_merges_in_any_order() {
    let interactive = [
        HostRedrawRequest::frame_update_only_for_scenario(UiPerfScenario::DrawerResize),
        HostRedrawRequest::full_frame_for_scenario(UiPerfScenario::WindowResize, true),
        HostRedrawRequest::region_with_frame_update(FrameRect {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 200.0,
        }),
    ]
    .map(HostRedrawRequest::into_interactive_frame_update);
    for first in &interactive {
        for second in &interactive {
            for requests in [
                [
                    HostRedrawRequest::runtime_tick(),
                    first.clone(),
                    second.clone(),
                ],
                [
                    first.clone(),
                    HostRedrawRequest::runtime_tick(),
                    second.clone(),
                ],
                [
                    first.clone(),
                    second.clone(),
                    HostRedrawRequest::runtime_tick(),
                ],
            ] {
                let merged = requests
                    .into_iter()
                    .fold(HostRedrawRequest::none(), HostRedrawRequest::merge);
                assert!(merged.requires_frame_update());
                assert!(!merged.prefers_interactive_frame_update());
                assert!(!merged
                    .into_present_retry(UiPerfScenario::Click)
                    .requires_runtime_tick());
            }
        }
    }
}
