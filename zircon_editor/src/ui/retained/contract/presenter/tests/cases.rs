use super::*;

struct RecordingPresenter {
    present_count: u64,
}

impl RecordingPresenter {
    fn new() -> Self {
        Self { present_count: 0 }
    }
}

impl HostChromePresenter for RecordingPresenter {
    fn resize(&mut self, _size: (u32, u32)) -> HostPresenterResult<()> {
        Ok(())
    }

    fn present(
        &mut self,
        _presentation: &HostWindowPresentationData,
        _presentation_cursor: HostPresentationGenerationCursor,
        _damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        self.present_count = self.present_count.saturating_add(1);
        let mut diagnostics = HostRefreshDiagnostics::default();
        diagnostics.record_present(1, false, true);
        Ok(diagnostics.with_invalidation_diagnostics(invalidation))
    }

    fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics {
        let mut diagnostics = HostRefreshDiagnostics::default();
        for _ in 0..self.present_count {
            diagnostics.record_present(1, false, true);
        }
        diagnostics
    }
}

#[test]
fn host_chrome_presenter_trait_accepts_boxed_backend() {
    let mut presenter: Box<dyn HostChromePresenter> = Box::new(RecordingPresenter::new());

    presenter
        .resize((320, 200))
        .expect("resize should route through trait object");
    let diagnostics = presenter
        .present(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            Some(FrameRect {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            }),
            HostInvalidationDiagnostics {
                slow_path_rebuild_count: 1,
                render_rebuild_count: 2,
                paint_only_request_count: 3,
            },
        )
        .expect("present should route through trait object");

    assert_eq!(diagnostics.present_count, 1);
    assert_eq!(diagnostics.region_paint_count, 1);
    assert_eq!(diagnostics.slow_path_rebuild_count, 1);
    assert_eq!(diagnostics.render_rebuild_count, 2);
    assert_eq!(diagnostics.paint_only_request_count, 3);
    assert_eq!(presenter.diagnostics_snapshot().present_count, 1);
}

#[test]
fn native_resize_present_has_a_conservative_backend_fallback() {
    let mut presenter: Box<dyn HostChromePresenter> = Box::new(RecordingPresenter::new());

    presenter
        .present_during_native_resize(
            &HostWindowPresentationData::default(),
            HostPresentationGenerationCursor::default(),
            HostInvalidationDiagnostics::default(),
        )
        .expect("backends without a resize cache should use an ordinary full present");

    assert_eq!(presenter.diagnostics_snapshot().present_count, 1);
}
