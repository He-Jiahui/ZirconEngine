use zircon_runtime::rhi::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
    UiSurfacePresentStats, UiSurfacePresenter, UiSurfaceRect, UiSurfaceTextStyle,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::command_stream::{build_chrome_command_stream, ChromeCommandKind, ChromeCommandStream};
use super::error::HostPresenterResult;
use super::host_chrome_presenter::HostChromePresenter;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(super) struct GpuChromePresenter<P: UiSurfacePresenter> {
    surface: P,
    size: (u32, u32),
    diagnostics: HostRefreshDiagnostics,
    last_upload_bytes: u64,
    last_draw_calls: u64,
}

impl<P: UiSurfacePresenter> HostChromePresenter for GpuChromePresenter<P> {
    fn resize(&mut self, size: (u32, u32)) -> HostPresenterResult<()> {
        GpuChromePresenter::resize(self, size)
    }

    fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        let stream = build_chrome_command_stream(presentation, self.size, damage.as_ref(), true);
        self.present_stream(&stream, invalidation)
    }

    fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics {
        GpuChromePresenter::diagnostics_snapshot(self)
    }
}

impl<P: UiSurfacePresenter> GpuChromePresenter<P> {
    pub(super) fn new(surface: P, size: (u32, u32)) -> Self {
        Self {
            surface,
            size: clamp_size(size),
            diagnostics: HostRefreshDiagnostics::default(),
            last_upload_bytes: 0,
            last_draw_calls: 0,
        }
    }

    pub(super) fn resize(&mut self, size: (u32, u32)) -> HostPresenterResult<()> {
        let size = clamp_size(size);
        self.surface.resize(size.0, size.1)?;
        self.size = size;
        Ok(())
    }

    pub(super) fn present_stream(
        &mut self,
        stream: &ChromeCommandStream,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        let draw_list = ui_surface_draw_list_from_stream(stream);
        let stats = self.surface.present(&draw_list)?;
        self.record_present_stats(stream, &stats);

        let painted_pixels = stream
            .damage()
            .map(|damage| damage_pixel_count(damage, stream.surface_size()))
            .unwrap_or_else(|| full_surface_pixels(stream.surface_size()));
        record_current_ui_perf_counter(UiPerfCounter::PaintedPixels, painted_pixels as f64);
        if stream.is_full_rebuild() {
            record_current_ui_perf_counter(UiPerfCounter::FullPaintCount, 1.0);
        } else {
            record_current_ui_perf_counter(UiPerfCounter::RegionPaintCount, 1.0);
        }
        self.diagnostics.record_present(
            painted_pixels,
            stream.is_full_rebuild(),
            !stream.is_full_rebuild(),
        );
        Ok(self
            .diagnostics
            .clone()
            .with_invalidation_diagnostics(invalidation))
    }

    pub(super) fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics {
        self.diagnostics.clone()
    }

    pub(super) fn last_upload_bytes(&self) -> u64 {
        self.last_upload_bytes
    }

    pub(super) fn last_draw_calls(&self) -> u64 {
        self.last_draw_calls
    }

    fn record_present_stats(
        &mut self,
        stream: &ChromeCommandStream,
        stats: &UiSurfacePresentStats,
    ) {
        self.last_upload_bytes = stats.image_upload_bytes;
        self.last_draw_calls = stats.draw_calls;
        record_current_ui_perf_counter(
            UiPerfCounter::GpuUploadBytes,
            stats.image_upload_bytes as f64,
        );
        record_current_ui_perf_counter(UiPerfCounter::GpuDrawCalls, stats.draw_calls as f64);
        record_current_ui_perf_counter(
            UiPerfCounter::GpuVisibleCommands,
            stats.visible_command_count as f64,
        );
        record_current_ui_perf_counter(
            UiPerfCounter::GpuVisibleDrawItems,
            stats.visible_draw_item_count as f64,
        );
        record_current_ui_perf_counter(
            UiPerfCounter::GpuBatchLayers,
            stats.batch_layer_count as f64,
        );
        record_current_ui_perf_counter(
            UiPerfCounter::GpuBatchDependencies,
            stats.batch_dependency_count as f64,
        );
        if stream.is_full_rebuild() {
            record_current_ui_perf_counter(UiPerfCounter::ChromeCommandFullRebuildCount, 1.0);
        } else {
            record_current_ui_perf_counter(UiPerfCounter::ChromeCommandPatchCount, 1.0);
        }
    }
}

fn ui_surface_draw_list_from_stream(stream: &ChromeCommandStream) -> UiSurfaceDrawList {
    UiSurfaceDrawList::new(
        stream.surface_size(),
        stream.damage().map(ui_rect),
        stream
            .commands()
            .iter()
            .map(ui_surface_command_from_chrome)
            .collect(),
    )
}

fn ui_surface_command_from_chrome(
    command: &super::command_stream::ChromeCommand,
) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index: command.z_index,
        frame: ui_rect(&command.frame),
        clip: command.clip.as_ref().map(ui_rect),
        kind: match &command.kind {
            ChromeCommandKind::Quad {
                color,
                corner_radius,
            } => UiSurfaceCommandKind::Quad {
                color: *color,
                corner_radius: *corner_radius,
            },
            ChromeCommandKind::Border {
                color,
                width,
                corner_radius,
            } => UiSurfaceCommandKind::Border {
                color: *color,
                width: *width,
                corner_radius: *corner_radius,
            },
            ChromeCommandKind::Text {
                text,
                color,
                size,
                line_height,
                style,
            } => UiSurfaceCommandKind::Text {
                text: text.clone(),
                color: *color,
                font_size: *size,
                line_height: *line_height,
                style: ui_text_style(*style),
            },
            ChromeCommandKind::Image { payload } => UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: payload.resource_key.clone(),
                    width: payload.width,
                    height: payload.height,
                    upload_bytes: payload.upload_bytes,
                    rgba: payload.rgba.clone(),
                },
            },
            ChromeCommandKind::Clip => UiSurfaceCommandKind::Clip,
        },
    }
}

fn ui_text_style(style: UiTextRunPaintStyle) -> UiSurfaceTextStyle {
    match (style.strong, style.emphasis) {
        (true, true) => UiSurfaceTextStyle::StrongEmphasis,
        (true, false) => UiSurfaceTextStyle::Strong,
        (false, true) => UiSurfaceTextStyle::Emphasis,
        (false, false) => UiSurfaceTextStyle::Regular,
    }
}

fn ui_rect(frame: &FrameRect) -> UiSurfaceRect {
    UiSurfaceRect::new(frame.x, frame.y, frame.width, frame.height)
}

fn full_surface_pixels(size: (u32, u32)) -> u64 {
    u64::from(size.0.max(1)) * u64::from(size.1.max(1))
}

fn damage_pixel_count(frame: &super::super::data::FrameRect, size: (u32, u32)) -> u64 {
    let x0 = frame.x.floor().max(0.0).min(size.0.max(1) as f32) as u32;
    let y0 = frame.y.floor().max(0.0).min(size.1.max(1) as f32) as u32;
    let x1 = (frame.x + frame.width)
        .ceil()
        .max(0.0)
        .min(size.0.max(1) as f32) as u32;
    let y1 = (frame.y + frame.height)
        .ceil()
        .max(0.0)
        .min(size.1.max(1) as f32) as u32;
    u64::from(x1.saturating_sub(x0)) * u64::from(y1.saturating_sub(y0))
}

fn clamp_size(size: (u32, u32)) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::data::FrameRect;
    use crate::ui::retained_host::host_contract::presenter::command_stream::{
        ChromeCommandLayer, ChromeImagePayload,
    };
    use crate::ui::retained_host::host_contract::presenter::error::HostPresenterError;
    use zircon_runtime::rhi::RhiError;

    #[derive(Default)]
    struct RecordingSurfacePresenter {
        fail_present: bool,
        last: UiSurfacePresentStats,
    }

    impl UiSurfacePresenter for RecordingSurfacePresenter {
        fn resize(&mut self, width: u32, height: u32) -> Result<(), RhiError> {
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
            stats.presented_frame_count = 1;
            self.last = stats.clone();
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
    fn gpu_surface_commands_preserve_chrome_corner_radius() {
        let mut stream = ChromeCommandStream::full_rebuild((64, 64));
        stream.push_quad(
            ChromeCommandLayer::Static,
            1,
            FrameRect {
                x: 4.0,
                y: 6.0,
                width: 20.0,
                height: 12.0,
            },
            None,
            [255, 0, 0, 255],
            9.0,
        );
        stream.push_border(
            ChromeCommandLayer::Static,
            2,
            FrameRect {
                x: 4.0,
                y: 24.0,
                width: 20.0,
                height: 12.0,
            },
            None,
            [0, 255, 0, 255],
            2.0,
            8.0,
        );

        let draw_list = ui_surface_draw_list_from_stream(&stream);

        assert!(matches!(
            draw_list.commands[0].kind,
            UiSurfaceCommandKind::Quad {
                color: [255, 0, 0, 255],
                corner_radius: 9.0,
            }
        ));
        assert!(matches!(
            draw_list.commands[1].kind,
            UiSurfaceCommandKind::Border {
                color: [0, 255, 0, 255],
                width: 2.0,
                corner_radius: 8.0,
            }
        ));
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
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(vec![128; 16]),
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
}
