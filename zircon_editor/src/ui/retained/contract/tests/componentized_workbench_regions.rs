use super::{componentized_workbench_chrome_regions, FrameRect, HostWindowLayoutData};

#[test]
fn regions_respect_frame_origin_and_clip_status_to_the_surface() {
    let layout = HostWindowLayoutData {
        authoritative: true,
        center_band_frame: FrameRect {
            x: 10.0,
            y: 50.0,
            width: 100.0,
            height: 35.0,
        },
        status_bar_frame: FrameRect {
            x: 5.0,
            y: 85.0,
            width: 120.0,
            height: 30.0,
        },
        ..Default::default()
    };
    let bounds = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 80.0,
    };

    let regions = componentized_workbench_chrome_regions(&layout, &bounds)
        .expect("visible componentized layout should publish chrome regions");

    assert_eq!(
        regions.top_chrome,
        FrameRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 30.0,
        }
    );
    assert_eq!(
        regions.status_bar,
        FrameRect {
            x: 10.0,
            y: 85.0,
            width: 100.0,
            height: 15.0,
        }
    );
}

#[test]
fn missing_layout_regions_preserve_the_legacy_fallback() {
    assert!(componentized_workbench_chrome_regions(
        &HostWindowLayoutData::default(),
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }
    )
    .is_none());
}
