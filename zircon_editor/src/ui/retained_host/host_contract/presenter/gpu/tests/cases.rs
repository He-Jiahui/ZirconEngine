use super::*;
use crate::ui::retained_host::host_contract::chrome_command_stream::{
    ChromeCommandLayer, ChromeCommandStream, ChromeImagePayload,
};
use crate::ui::retained_host::host_contract::data::{FrameRect, HostPresentationGenerationCursor};
use crate::ui::retained_host::host_contract::presenter::error::HostPresenterError;
use zircon_runtime::rhi::{
    RhiError, UiSurfaceDrawList, UiSurfacePresentOutcome, UiSurfacePresentStats,
    UiSurfacePresenter, UiSurfaceRect,
};

#[derive(Default)]
struct RecordingSurfacePresenter {
    fail_present: bool,
    next_outcome: Option<UiSurfacePresentOutcome>,
    last: UiSurfacePresentStats,
    last_draw_list: Option<UiSurfaceDrawList>,
    resize_count: u64,
}

impl UiSurfacePresenter for RecordingSurfacePresenter {
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RhiError> {
        self.resize_count = self.resize_count.saturating_add(1);
        self.last.surface_size = (width.max(1), height.max(1));
        Ok(())
    }

    fn present(
        &mut self,
        draw_list: &UiSurfaceDrawList,
    ) -> Result<UiSurfacePresentStats, RhiError> {
        if self.fail_present {
            return Err(RhiError::SurfaceUnavailable("test".to_string()));
        }
        let mut stats = draw_list.stats();
        stats.outcome = self.next_outcome.take().unwrap_or_default();
        stats.presented_frame_count = 1;
        self.last = stats.clone();
        self.last_draw_list = Some(draw_list.clone());
        Ok(stats)
    }

    fn last_present_stats(&self) -> UiSurfacePresentStats {
        self.last.clone()
    }
}

#[test]
fn gpu_presenter_propagates_runtime_surface_failure() {
    let mut presenter = GpuChromePresenter::new(
        RecordingSurfacePresenter {
            fail_present: true,
            ..RecordingSurfacePresenter::default()
        },
        (64, 64),
    );
    let stream = ChromeCommandStream::full_rebuild((64, 64));

    let error = presenter
        .present_stream(&stream, HostInvalidationDiagnostics::default())
        .expect_err("runtime surface failure must not be hidden");

    assert!(matches!(error, HostPresenterError::Rhi(_)));
}

#[test]
fn gpu_presenter_retries_an_unsubmitted_frame_without_publishing_a_damage_baseline() {
    let mut presenter = GpuChromePresenter::new(
        RecordingSurfacePresenter {
            next_outcome: Some(UiSurfacePresentOutcome::RetryableNoSubmit),
            ..RecordingSurfacePresenter::default()
        },
        (64, 64),
    );
    let damage = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 8.0,
        height: 5.0,
    };

    let error = presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            Some(damage.clone()),
            HostInvalidationDiagnostics::default(),
        )
        .expect_err("an unsubmitted swapchain frame must remain retryable");

    assert!(matches!(error, HostPresenterError::RetryableSurfacePresent));
    assert!(!presenter.surface_cache_initialized);
    assert_eq!(presenter.diagnostics.present_count, 0);

    let diagnostics = presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            Some(damage),
            HostInvalidationDiagnostics::default(),
        )
        .expect("the next surface acquisition should submit the full bootstrap frame");
    assert_eq!(diagnostics.present_count, 1);
    assert_eq!(
        presenter
            .surface
            .last_draw_list
            .as_ref()
            .expect("retry should reach the surface presenter")
            .damage,
        None
    );
}

#[test]
fn gpu_presenter_records_upload_bytes_draw_calls_and_damage_diagnostics() {
    let mut presenter = GpuChromePresenter::new(RecordingSurfacePresenter::default(), (64, 64));
    let damage = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 8.0,
        height: 5.0,
    };
    let mut stream = ChromeCommandStream::patch((64, 64), damage);
    stream.push_quad(
        ChromeCommandLayer::Dynamic,
        1,
        FrameRect {
            x: 4.0,
            y: 6.0,
            width: 8.0,
            height: 5.0,
        },
        None,
        [255, 0, 0, 255],
        0.0,
    );
    stream.push_image(
        2,
        FrameRect {
            x: 10.0,
            y: 10.0,
            width: 2.0,
            height: 2.0,
        },
        None,
        ChromeImagePayload {
            resource_key: "viewport".to_string(),
            resource_generation: 0,
            width: 2,
            height: 2,
            upload_bytes: 16,
            rgba: Some(vec![128; 16].into()),
            atlas_uv: None,
        },
    );

    let diagnostics = presenter
        .present_stream(&stream, HostInvalidationDiagnostics::default())
        .expect("surface presenter should accept the command stream");

    assert_eq!(diagnostics.present_count, 1);
    assert_eq!(diagnostics.full_paint_count, 0);
    assert_eq!(diagnostics.region_paint_count, 1);
    assert_eq!(diagnostics.painted_pixel_count, 40);
    assert_eq!(presenter.last_upload_bytes(), 16);
    assert_eq!(presenter.last_draw_calls(), 2);
}

#[test]
fn gpu_presenter_damage_present_uses_patch_after_surface_cache_is_ready() {
    let mut presenter = GpuChromePresenter::new(RecordingSurfacePresenter::default(), (64, 64));
    let damage = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 8.0,
        height: 5.0,
    };

    let diagnostics = presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            Some(damage.clone()),
            HostInvalidationDiagnostics::default(),
        )
        .expect("GPU presenter should bootstrap the direct-surface cache");

    let draw_list = presenter
        .surface
        .last_draw_list
        .as_ref()
        .expect("surface presenter should receive the submitted draw list");
    assert_eq!(draw_list.damage, None);
    assert!(draw_list.commands.iter().any(|command| {
        command.frame.x <= 0.0
            && command.frame.y <= 0.0
            && command.frame.width >= 64.0
            && command.frame.height >= 64.0
    }));
    assert_eq!(diagnostics.full_paint_count, 1);
    assert_eq!(diagnostics.region_paint_count, 0);
    assert_eq!(diagnostics.painted_pixel_count, 64 * 64);

    let diagnostics = presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            Some(damage.clone()),
            HostInvalidationDiagnostics::default(),
        )
        .expect("GPU presenter should submit damage once the cache is ready");

    let draw_list = presenter
        .surface
        .last_draw_list
        .as_ref()
        .expect("surface presenter should receive the submitted draw list");
    assert_eq!(
        draw_list.damage,
        Some(UiSurfaceRect::new(
            damage.x,
            damage.y,
            damage.width,
            damage.height
        ))
    );
    assert_eq!(diagnostics.full_paint_count, 1);
    assert_eq!(diagnostics.region_paint_count, 1);
    assert_eq!(diagnostics.painted_pixel_count, 64 * 64 + 40);
}

#[test]
fn gpu_presenter_keeps_live_full_and_damage_streams_unversioned() {
    let mut presenter = GpuChromePresenter::new(RecordingSurfacePresenter::default(), (64, 64));

    presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            None,
            HostInvalidationDiagnostics {
                slow_path_rebuild_count: 17,
                ..HostInvalidationDiagnostics::default()
            },
        )
        .expect("GPU presenter should submit the live full draw list");

    assert_eq!(
        presenter
            .surface
            .last_draw_list
            .as_ref()
            .expect("surface presenter should receive the submitted full draw list")
            .generation(),
        None
    );

    presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            Some(FrameRect {
                x: 1.0,
                y: 2.0,
                width: 8.0,
                height: 9.0,
            }),
            HostInvalidationDiagnostics {
                slow_path_rebuild_count: 17,
                ..HostInvalidationDiagnostics::default()
            },
        )
        .expect("GPU presenter should submit the live damage draw list");

    let draw_list = presenter
        .surface
        .last_draw_list
        .as_ref()
        .expect("surface presenter should receive the submitted draw list");
    assert_eq!(draw_list.generation(), None);
}

#[test]
fn gpu_presenter_resize_invalidates_damage_cache() {
    let mut presenter = GpuChromePresenter::new(RecordingSurfacePresenter::default(), (64, 64));
    let damage = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 8.0,
        height: 5.0,
    };

    presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            None,
            HostInvalidationDiagnostics::default(),
        )
        .unwrap();
    presenter.resize((128, 96)).unwrap();
    presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            Some(damage),
            HostInvalidationDiagnostics::default(),
        )
        .unwrap();

    let draw_list = presenter
        .surface
        .last_draw_list
        .as_ref()
        .expect("surface presenter should receive the submitted draw list");
    assert_eq!(draw_list.surface_size, (128, 96));
    assert_eq!(draw_list.damage, None);
}

#[test]
fn gpu_presenter_same_size_resize_preserves_the_damage_cache() {
    let mut presenter = GpuChromePresenter::new(RecordingSurfacePresenter::default(), (64, 64));
    presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            None,
            HostInvalidationDiagnostics::default(),
        )
        .unwrap();
    assert!(presenter.surface_cache_initialized);

    presenter.resize((64, 64)).unwrap();

    assert_eq!(presenter.surface.resize_count, 0);
    assert!(presenter.surface_cache_initialized);
}

#[test]
fn gpu_presenter_builds_one_command_snapshot_per_native_resize_transaction() {
    let mut presenter = GpuChromePresenter::new(RecordingSurfacePresenter::default(), (320, 200));
    let damage = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 8.0,
        height: 5.0,
    };

    presenter.resize((300, 180)).unwrap();
    presenter
        .present_during_native_resize(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            HostInvalidationDiagnostics::default(),
        )
        .unwrap();
    let first_generation = presenter
        .surface
        .last_draw_list
        .as_ref()
        .and_then(UiSurfaceDrawList::generation)
        .expect("resize snapshot should be versioned");
    presenter.resize((280, 160)).unwrap();
    presenter
        .present_during_native_resize(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            HostInvalidationDiagnostics::default(),
        )
        .unwrap();

    assert_eq!(presenter.native_resize_snapshot_build_count, 1);
    assert_eq!(presenter.native_resize_snapshot_reuse_count, 1);
    assert_eq!(
        presenter
            .surface
            .last_draw_list
            .as_ref()
            .and_then(UiSurfaceDrawList::generation),
        Some(first_generation)
    );
    assert_eq!(
        presenter
            .surface
            .last_draw_list
            .as_ref()
            .expect("resize should submit the retained draw list")
            .surface_size,
        (280, 160)
    );
    assert_eq!(
        presenter
            .surface
            .last_draw_list
            .as_ref()
            .expect("resize should retain the generation projection")
            .projection_size(),
        (320, 200)
    );
    assert!(!presenter.surface_cache_initialized);

    presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            Some(damage),
            HostInvalidationDiagnostics::default(),
        )
        .unwrap();

    assert!(presenter.native_resize_draw_list.is_none());
    assert_eq!(
        presenter
            .surface
            .last_draw_list
            .as_ref()
            .expect("first ordinary present after resize must submit a draw list")
            .damage,
        None
    );

    presenter.resize((260, 140)).unwrap();
    presenter
        .present_during_native_resize(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            HostInvalidationDiagnostics::default(),
        )
        .unwrap();
    assert_ne!(
        presenter
            .surface
            .last_draw_list
            .as_ref()
            .and_then(UiSurfaceDrawList::generation),
        Some(first_generation)
    );
}
