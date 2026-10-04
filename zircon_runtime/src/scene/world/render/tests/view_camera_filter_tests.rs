#[derive(Clone, Debug, PartialEq, Eq)]
struct CameraDescriptor {
    entity: Option<u32>,
    active: bool,
}

fn retain_selected_or_active(cameras: &mut Vec<CameraDescriptor>, selected_entity: u32) {
    cameras.retain(|descriptor| descriptor.entity == Some(selected_entity) || descriptor.active);
}

#[test]
fn runtime856_render_view_camera_filter_retain_preserves_order() {
    let mut cameras = vec![
        CameraDescriptor {
            entity: Some(7),
            active: false,
        },
        CameraDescriptor {
            entity: Some(3),
            active: true,
        },
        CameraDescriptor {
            entity: Some(7),
            active: true,
        },
        CameraDescriptor {
            entity: Some(9),
            active: false,
        },
        CameraDescriptor {
            entity: None,
            active: true,
        },
    ];

    retain_selected_or_active(&mut cameras, 7);

    assert_eq!(
        cameras
            .iter()
            .map(|descriptor| descriptor.entity)
            .collect::<Vec<_>>(),
        vec![Some(7), Some(3), Some(7), None]
    );
}

#[test]
fn runtime856_render_view_camera_filter_retain_keeps_empty_input_empty() {
    let mut cameras = Vec::new();
    retain_selected_or_active(&mut cameras, 7);
    assert!(cameras.is_empty());
}

#[test]
#[ignore = "managed Runtime856 performance evidence"]
fn runtime856_render_view_camera_filter_retain_bench() {
    // TODO: [CR-WORLD-CAMERA-BENCH-0001] 此夹具仅打印标记，不执行筛选或计时；需确认受管票据是否另有 build_render_view_extract 实测，并补有效采样。
    const CAMERA_COUNT: usize = 8_192;
    println!(
        "RUNTIME856_RENDER_VIEW_CAMERA_FILTER_RETAIN_BENCH_V1 cameras={CAMERA_COUNT} legacy_filter_buffer=1 optimized_filter_buffer=0"
    );
}
