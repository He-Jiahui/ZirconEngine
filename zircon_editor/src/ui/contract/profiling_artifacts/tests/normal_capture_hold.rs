use super::*;

fn controller(hwnd: u64) -> NormalCaptureController {
    NormalCaptureController::for_native_target(Some(RenderNativeSurfaceTarget::Win32 {
        hwnd,
        hinstance: None,
    }))
}

fn held(deadline: Instant) -> NormalCaptureController {
    NormalCaptureController {
        native_hwnd: Some(9),
        held: Some(Held {
            nonce: "test".into(),
            cursor: HostPresentationGenerationCursor::default(),
            size: (100, 100),
            deadline,
            evidence: json!({"pid":std::process::id(),"nonce":"test","caseSha256":"a".repeat(64)}),
        }),
        consumed_nonce: Some("test".into()),
        ..Default::default()
    }
}
fn capture_request() -> Request {
    Request {
        nonce: "fresh".into(),
        case_sha256: "a".repeat(64),
        pid: std::process::id(),
        requested_at_unix_ms: unix_ms(),
        deadline_ms: 1000,
        physical_size: [100, 100],
        release: false,
        gpu_readback: true,
        hwnd: Some(9),
    }
}

#[test]
fn idle_capture_demand_requires_a_new_nonce_and_matching_actual_extent() {
    let mut controller = controller(9);
    let mut value = capture_request();
    assert!(controller.request_requires_present(&value, (100, 100)));
    assert!(!controller.request_requires_present(&value, (101, 100)));
    value.hwnd = None;
    assert!(!controller.request_requires_present(&value, (100, 100)));
    value.hwnd = Some(9);
    value.release = true;
    assert!(!controller.request_requires_present(&value, (100, 100)));
    value.release = false;
    controller.attempted_readback_nonce = Some(value.nonce.clone());
    assert!(!controller.request_requires_present(&value, (100, 100)));
    controller.attempted_readback_nonce = None;
    controller.consumed_nonce = Some(value.nonce.clone());
    assert!(!controller.request_requires_present(&value, (100, 100)));
}

#[test]
fn an_existing_hold_and_pending_readback_do_not_schedule_another_capture_present() {
    let mut controller = held(Instant::now() + Duration::from_secs(1));
    let mut value = capture_request();
    value.nonce = "test".into();
    assert!(!controller.request_requires_present(&value, (100, 100)));
    value.nonce = "fresh".into();
    assert!(controller.request_requires_present(&value, (100, 100)));
    controller.readback_pending = Some(PendingReadback {
        nonce: value.nonce.clone(),
        case_sha256: value.case_sha256.clone(),
        hwnd: 9,
        cursor: HostPresentationGenerationCursor::default(),
        size: (100, 100),
        deadline: Instant::now() + Duration::from_secs(1),
        presented_frame_count: None,
        scene_image: None,
        scene_unavailable_reason: None,
    });
    assert!(!controller.request_requires_present(&value, (100, 100)));
}

#[test]
fn capture_requests_are_routed_before_a_foreign_presenter_consumes_the_nonce() {
    let cursor = HostPresentationGenerationCursor::default();
    let mut target = controller(9);
    let mut foreign = controller(10);
    let request = capture_request();
    assert!(target.request_requires_present(&request, (100, 100)));
    assert!(!foreign.request_requires_present(&request, (100, 100)));
    assert!(foreign
        .prepare_frame_readback_for_request(cursor, (100, 100), request)
        .is_none());
    assert!(foreign.attempted_readback_nonce.is_none());
    assert!(foreign.readback_pending.is_none());
    assert!(foreign.consumed_nonce.is_none());

    let prepared = target
        .prepare_frame_readback_for_request(cursor, (100, 100), capture_request())
        .expect("the requested native owner should prepare the actual readback");
    assert_eq!(prepared.nonce, "fresh");
    assert_eq!(target.readback_pending.as_ref().unwrap().hwnd, 9);
    assert_eq!(target.attempted_readback_nonce.as_deref(), Some("fresh"));

    // A foreign request must not consume the nonce: this presenter can still
    // prepare it when a request of the same PID and extent targets its HWND.
    let mut request = capture_request();
    request.hwnd = Some(10);
    assert!(foreign.request_requires_present(&request, (100, 100)));
    assert!(foreign
        .prepare_frame_readback_for_request(cursor, (100, 100), request)
        .is_some());
    assert_eq!(foreign.readback_pending.as_ref().unwrap().hwnd, 10);
}

#[test]
fn unbound_and_zero_native_targets_cannot_consume_capture_requests() {
    let cursor = HostPresentationGenerationCursor::default();
    for mut controller in [NormalCaptureController::default(), controller(0)] {
        assert!(!controller.request_requires_present(&capture_request(), (100, 100)));
        assert!(controller
            .prepare_frame_readback_for_request(cursor, (100, 100), capture_request())
            .is_none());
        assert!(controller.attempted_readback_nonce.is_none());
        assert!(controller.readback_pending.is_none());
    }
}

#[test]
fn foreign_replacement_and_release_leave_the_existing_owner_hold_intact() {
    let cursor = HostPresentationGenerationCursor::default();
    let mut owner = held(Instant::now() + Duration::from_secs(1));
    let mut foreign = capture_request();
    foreign.hwnd = Some(10);
    assert!(owner.before_present_for_request(cursor, (100, 100), Some(&foreign)));
    assert_eq!(owner.held.as_ref().unwrap().nonce, "test");
    foreign.release = true;
    foreign.nonce = "test".into();
    assert!(owner.before_present_for_request(cursor, (100, 100), Some(&foreign)));
    assert_eq!(owner.held.as_ref().unwrap().nonce, "test");

    let fresh = capture_request();
    assert!(!owner.before_present_for_request(cursor, (100, 100), Some(&fresh)));
    assert!(owner.held.is_none());
    assert!(owner.request_requires_present(&fresh, (100, 100)));

    let mut owner = held(Instant::now() + Duration::from_secs(1));
    let mut release = capture_request();
    release.nonce = "test".into();
    release.release = true;
    assert!(!owner.before_present_for_request(cursor, (100, 100), Some(&release)));
    assert!(owner.held.is_none());
}

#[test]
fn obsolete_or_foreign_acknowledgements_cannot_replace_the_live_request_ack() {
    let owner = controller(9);
    let foreign = controller(10);
    let request = capture_request();
    let evidence = json!({
        "pid": request.pid,
        "nonce": request.nonce.as_str(),
        "caseSha256": request.case_sha256.as_str(),
    });
    assert!(owner.publication_matches_request(&request, &evidence));
    assert!(!foreign.publication_matches_request(&request, &evidence));

    let mut obsolete = evidence.clone();
    obsolete["nonce"] = json!("old-request");
    assert!(!owner.publication_matches_request(&request, &obsolete));
    obsolete = evidence.clone();
    obsolete["caseSha256"] = json!("b".repeat(64));
    assert!(!owner.publication_matches_request(&request, &obsolete));
    obsolete = evidence.clone();
    obsolete["pid"] = json!(u64::from(request.pid) + 1);
    assert!(!owner.publication_matches_request(&request, &obsolete));

    let mut next = capture_request();
    next.hwnd = Some(10);
    assert!(!owner.publication_matches_request(&next, &evidence));
    assert!(foreign.publication_matches_request(&next, &evidence));
}

#[test]
fn generation_and_extent_changes_release_the_real_present_gate() {
    let cursor = HostPresentationGenerationCursor::default();
    let mut controller = held(Instant::now() + Duration::from_secs(1));
    assert!(controller.before_present(cursor, (100, 100)));
    assert!(!controller.before_present(
        HostPresentationGenerationCursor::new(1, 0, 0, 0, 0, 0),
        (100, 100)
    ));
    assert!(controller.held.is_none());
    let mut controller = held(Instant::now() + Duration::from_secs(1));
    assert!(!controller.before_present(cursor, (101, 100)));
}
#[test]
fn expired_hold_does_not_delay_an_input_driven_present() {
    let mut controller = held(Instant::now() - Duration::from_millis(1));
    assert!(!controller.before_present(HostPresentationGenerationCursor::default(), (100, 100)));
    assert_eq!(controller.consumed_nonce.as_deref(), Some("test"));
}

#[test]
fn idle_owned_capture_release_wakes_consumed_nonce_once_without_a_business_input() {
    let cursor = HostPresentationGenerationCursor::default();
    let mut owner = held(Instant::now() + Duration::from_secs(1));
    let mut release = capture_request();
    release.nonce = "test".into();
    release.release = true;
    owner.attempted_readback_nonce = Some(release.nonce.clone());
    assert!(owner.request_requires_present(&release, (100, 100)));
    assert!(!owner.before_present_for_request(cursor, (100, 100), Some(&release)));
    assert!(owner.held.is_none());
    assert_eq!(owner.consumed_nonce.as_deref(), Some("test"));
    assert!(!owner.request_requires_present(&release, (100, 100)));
    assert!(owner
        .prepare_frame_readback_for_request(cursor, (100, 100), release)
        .is_none());
}

#[test]
fn idle_owned_capture_release_rejects_foreign_or_changed_publication_identity() {
    for mutation in 0..5 {
        let mut owner = held(Instant::now() + Duration::from_secs(1));
        let mut release = capture_request();
        release.nonce = "test".into();
        release.release = true;
        match mutation {
            0 => release.hwnd = Some(10),
            1 => release.pid += 1,
            2 => release.nonce = "other".into(),
            3 => release.case_sha256 = "b".repeat(64),
            _ => release.physical_size = [101, 100],
        }
        assert!(
            !owner.request_requires_present(&release, (100, 100)),
            "mutation {mutation}"
        );
        if mutation != 2 {
            assert!(owner.before_present_for_request(
                HostPresentationGenerationCursor::default(),
                (100, 100),
                Some(&release)
            ));
            assert!(owner.held.is_some());
        }
    }
}

#[test]
fn normal_viewport_media_hold_is_invalidated_by_each_actual_cursor_component() {
    for component in 0..7 {
        let mut values = [0; 7];
        values[component] = 1;
        let cursor = HostPresentationGenerationCursor::new_with_geometry(
            values[0], values[1], values[2], values[3], values[4], values[5], values[6],
        );
        let mut owner = held(Instant::now() + Duration::from_secs(1));
        assert!(!owner.before_present_for_request(cursor, (100, 100), None));
        assert!(owner.held.is_none(), "cursor component {component}");
    }
}

fn scene_result() -> (
    NormalSceneImageSelection,
    zircon_runtime::rhi::UiSurfaceFrameReadback,
) {
    use zircon_runtime::rhi::*;
    let request = UiSurfaceImageReadbackRequest {
        command_index: 12,
        resource_key: "actual-external-scene".into(),
        resource_generation: 5,
        physical_size: (1, 1),
        expected_pixel_format: None,
    };
    let frame = UiSurfaceRect::new(10.0, 20.0, 100.0, 100.0);
    let selected = NormalSceneImageSelection {
        request: request.clone(),
        evidence: json!({}),
        command_frame: frame,
        command_clip: Some(frame),
    };
    let submission = SubmissionTicket::new(
        DeviceId::new(9),
        DeviceGeneration::initial(),
        RenderQueueClass::Graphics,
        7,
    );
    let mut tracker = DiagnosticReadbackTracker::new(
        DeviceId::new(9),
        DeviceGeneration::initial(),
        DiagnosticReadbackBudget::default(),
    );
    tracker.begin_frame(4).unwrap();
    let window = tracker.admit(DiagnosticReadbackKind::Texture, 16).unwrap();
    let scene = tracker.admit(DiagnosticReadbackKind::Texture, 4).unwrap();
    tracker.bind_active_frame(submission).unwrap();
    let result = UiSurfaceFrameReadback {
        request: UiSurfaceFrameReadbackRequest {
            nonce: "same-actual-frame".into(),
            physical_size: (2, 2),
            expected_draw_generation: Some(3),
            deadline_ms: 1000,
            image: Some(request.clone()),
        },
        surface_epoch: 2,
        present_index: 4,
        submission: Some(submission),
        receipt: tracker.terminalize(window, DiagnosticReadbackTerminal::Succeeded),
        pixel_format: Some(UiSurfaceFramePixelFormat::Rgba8Srgb),
        color8: Some(vec![255; 16]),
        opaque_pixels: Some(true),
        surface_alpha_mode: "Opaque".into(),
        native_target: Some(RenderNativeSurfaceTarget::Win32 {
            hwnd: 9,
            hinstance: None,
        }),
        failure: None,
        image: Some(UiSurfaceImageReadback {
            request,
            command_frame: Some(frame),
            command_clip: Some(frame),
            receipt: tracker.terminalize(scene, DiagnosticReadbackTerminal::Succeeded),
            pixel_format: Some(UiSurfaceFramePixelFormat::Rgba8Srgb),
            alpha_mode: Some(UiSurfaceImageAlphaMode::Opaque),
            color8: Some(vec![40, 80, 120, 255]),
            opaque_pixels: Some(true),
            failure: None,
        }),
    };
    (selected, result)
}

#[test]
fn normal_viewport_media_requires_distinct_success_receipts_for_the_same_actual_submission() {
    let (selected, result) = scene_result();
    assert!(scene_readback_matches(Some(&selected), &result));
    for mutation in 0..19 {
        let (selected, mut result) = scene_result();
        let image = result.image.as_mut().unwrap();
        match mutation {
            0 => image.request.resource_key = "neighbor".into(),
            1 => image.request.resource_generation += 1,
            2 => image.request.command_index += 1,
            3 => image.request.physical_size = (2, 1),
            4 => image.command_frame.as_mut().unwrap().x += 1.0,
            5 => image.command_clip = None,
            6 => image.receipt = result.receipt,
            7 => {
                image.color8.as_mut().unwrap().pop();
            }
            8 => image.color8.as_mut().unwrap()[3] = 0,
            9 => image.pixel_format = None,
            10 => image.failure = Some("actual resource copy failed".into()),
            11 => result.submission = None,
            12 => result.request.image.as_mut().unwrap().resource_key = "other".into(),
            13 => image.receipt = None,
            14 => result.receipt = None,
            15 => image.alpha_mode = None,
            16 => image.opaque_pixels = Some(false),
            17 => result.receipt = image.receipt,
            _ => {
                result.submission = Some(zircon_runtime::rhi::SubmissionTicket::new(
                    zircon_runtime::rhi::DeviceId::new(9),
                    zircon_runtime::rhi::DeviceGeneration::initial(),
                    zircon_runtime::rhi::RenderQueueClass::Graphics,
                    8,
                ))
            }
        }
        assert!(
            !scene_readback_matches(Some(&selected), &result),
            "mutation {mutation}"
        );
    }
    let (mut selected, mut result) = scene_result();
    selected.request.expected_pixel_format =
        Some(zircon_runtime::rhi::UiSurfaceFramePixelFormat::Bgra8Srgb);
    result.request.image = Some(selected.request.clone());
    result.image.as_mut().unwrap().request = selected.request.clone();
    assert!(!scene_readback_matches(Some(&selected), &result));
}

#[test]
fn normal_viewport_media_absence_preserves_the_existing_window_readback_contract() {
    let (_, mut result) = scene_result();
    assert!(!scene_readback_matches(None, &result));
    result.request.image = None;
    assert!(!scene_readback_matches(None, &result));
    result.image = None;
    assert!(scene_readback_matches(None, &result));
}
