use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use winit::window::Window;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use crate::entry::runtime_library::RuntimeFrame;

pub(crate) struct ReferenceCpuPresenter {
    _context: Context<Arc<dyn Window>>,
    surface: Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: ZrRuntimeViewportSizeV1,
    metrics: ReferenceCpuPresenterMetrics,
}

impl ReferenceCpuPresenter {
    pub(crate) fn new(window: Arc<dyn Window>) -> Result<Self, softbuffer::SoftBufferError> {
        let context = Context::new(window.clone())?;
        let mut surface = Surface::new(&context, window.clone())?;
        let size = current_window_size(window.as_ref());
        resize_surface(&mut surface, size)?;
        Ok(Self {
            _context: context,
            surface,
            size,
            metrics: ReferenceCpuPresenterMetrics::default(),
        })
    }

    pub(crate) fn resize(
        &mut self,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<(), softbuffer::SoftBufferError> {
        let size = clamp_size(size);
        resize_surface(&mut self.surface, size)?;
        self.size = size;
        Ok(())
    }

    pub(crate) fn present(
        &mut self,
        frame: &RuntimeFrame<'_>,
        capture_started_at: Instant,
    ) -> Result<(), softbuffer::SoftBufferError> {
        zircon_runtime::profile_scope!("app", "reference_cpu_presenter", "present");
        let frame_size = ZrRuntimeViewportSizeV1::new(frame.width().max(1), frame.height().max(1));
        if self.size != frame_size {
            self.resize(frame_size)?;
        }

        let (surface, metrics) = (&mut self.surface, &mut self.metrics);
        let window = surface.window().clone();
        let mut buffer = surface.buffer_mut()?;
        let copied_bytes = frame
            .rgba()
            .len()
            .checked_div(4)
            .unwrap_or_default()
            .min(buffer.len())
            .saturating_mul(4);
        {
            zircon_runtime::profile_scope!("app", "reference_cpu_presenter", "copy_rgba");
            copy_rgba_to_xrgb(&mut buffer, frame.rgba());
        }
        metrics.record_copied_bytes(copied_bytes);

        window.pre_present_notify();
        zircon_runtime::profile_scope!("app", "reference_cpu_presenter", "softbuffer_present");
        buffer.present()?;
        metrics.record_presented(capture_started_at.elapsed());
        Ok(())
    }

    pub(crate) fn record_dropped_frame(&mut self) {
        self.metrics.record_dropped_frame();
    }

    pub(crate) fn publish_summary(&self) {
        self.metrics.publish_summary();
    }
}

#[derive(Default)]
struct ReferenceCpuPresenterMetrics {
    presented_frames: u64,
    copied_bytes: u64,
    dropped_frames: u64,
    total_latency_micros: u64,
    last_latency_micros: u64,
}

impl ReferenceCpuPresenterMetrics {
    fn record_copied_bytes(&mut self, copied_bytes: usize) {
        self.copied_bytes = self.copied_bytes.saturating_add(copied_bytes as u64);
        zircon_runtime::profile_counter!(
            "app",
            "runtime_entry.reference_cpu_presenter.copy_bytes",
            self.copied_bytes
        );
    }

    fn record_presented(&mut self, latency: Duration) {
        self.presented_frames = self.presented_frames.saturating_add(1);
        let latency_micros = duration_micros(latency);
        self.total_latency_micros = self.total_latency_micros.saturating_add(latency_micros);
        self.last_latency_micros = latency_micros;
        zircon_runtime::profile_counter!(
            "app",
            "runtime_entry.reference_cpu_presenter.latency_micros",
            self.last_latency_micros
        );
        zircon_runtime::profile_counter!(
            "app",
            "runtime_entry.reference_cpu_presenter.dropped_frames",
            self.dropped_frames
        );
    }

    fn record_dropped_frame(&mut self) {
        self.dropped_frames = self.dropped_frames.saturating_add(1);
        zircon_runtime::profile_counter!(
            "app",
            "runtime_entry.reference_cpu_presenter.dropped_frames",
            self.dropped_frames
        );
    }

    fn publish_summary(&self) {
        zircon_runtime::diagnostic_log::write_log(
            "runtime_surface_present",
            format!(
                "runtime_reference_cpu_presenter_summary capability=degraded frames={} copy_bytes={} latency_micros_total={} latency_micros_last={} dropped_frames={}",
                self.presented_frames,
                self.copied_bytes,
                self.total_latency_micros,
                self.last_latency_micros,
                self.dropped_frames,
            ),
        );
    }
}

fn duration_micros(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

fn copy_rgba_to_xrgb(surface: &mut [u32], rgba: &[u8]) -> bool {
    let covers_surface = surface
        .len()
        .checked_mul(4)
        .is_some_and(|required_bytes| rgba.len() >= required_bytes);
    if !covers_surface {
        surface.fill(0);
    }
    for (pixel, rgba) in surface.iter_mut().zip(rgba.chunks_exact(4)) {
        let red = rgba[0] as u32;
        let green = rgba[1] as u32;
        let blue = rgba[2] as u32;
        *pixel = (red << 16) | (green << 8) | blue;
    }
    !covers_surface
}

fn current_window_size(window: &dyn Window) -> ZrRuntimeViewportSizeV1 {
    let size = window.surface_size();
    ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1))
}

fn resize_surface(
    surface: &mut Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: ZrRuntimeViewportSizeV1,
) -> Result<(), softbuffer::SoftBufferError> {
    surface.resize(non_zero(size.width), non_zero(size.height))
}

fn clamp_size(size: ZrRuntimeViewportSizeV1) -> ZrRuntimeViewportSizeV1 {
    ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1))
}

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).expect("value is clamped to non-zero")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{copy_rgba_to_xrgb, duration_micros, ReferenceCpuPresenterMetrics};

    #[test]
    fn complete_rgba_frame_overwrites_the_surface_without_a_preclear() {
        let mut surface = [0x00ff_00ff, 0x00ff_00ff];

        let cleared = copy_rgba_to_xrgb(&mut surface, &[1, 2, 3, 255, 4, 5, 6, 255]);

        assert!(!cleared);
        assert_eq!(surface, [0x0001_0203, 0x0004_0506]);
    }

    #[test]
    fn truncated_rgba_frame_clears_uncovered_surface_pixels() {
        let mut surface = [0x00ff_00ff, 0x00ff_00ff];

        let cleared = copy_rgba_to_xrgb(&mut surface, &[1, 2, 3, 255]);

        assert!(cleared);
        assert_eq!(surface, [0x0001_0203, 0]);
    }

    #[test]
    fn reference_cpu_metrics_accumulate_copy_latency_and_drops_without_overflowing() {
        let mut metrics = ReferenceCpuPresenterMetrics::default();

        metrics.record_copied_bytes(1024);
        metrics.record_presented(Duration::from_micros(73));
        metrics.record_dropped_frame();

        assert_eq!(metrics.presented_frames, 1);
        assert_eq!(metrics.copied_bytes, 1024);
        assert_eq!(metrics.total_latency_micros, 73);
        assert_eq!(metrics.last_latency_micros, 73);
        assert_eq!(metrics.dropped_frames, 1);
    }

    #[test]
    fn reference_cpu_latency_conversion_saturates_at_u64_max() {
        assert_eq!(
            duration_micros(Duration::MAX),
            u64::MAX,
            "long-running diagnostic capture should not wrap its latency counter"
        );
    }
}
