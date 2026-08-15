use crate::ui::retained_host::primitives::{PhysicalPosition, PhysicalSize};
use std::time::{Duration, Instant};
use winit::dpi::{PhysicalPosition as WinitPhysicalPosition, PhysicalSize as WinitPhysicalSize};
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::ui::window::{
    UiWindowEventKind, UiWindowInputPumpEvent, UiWindowMetrics,
};

use super::super::platform_input::PlatformInputTranslation;
use super::super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use crate::ui::retained_host::ui_perf::UiPerfScenario;
const NATIVE_RESIZE_REFLOW_DEBOUNCE: Duration = Duration::from_millis(80);

impl UiHostWindowEventLoop {
    pub(super) fn handle_surface_resized(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        size: WinitPhysicalSize<u32>,
        translated_event: PlatformInputTranslation,
    ) {
        self.begin_input_outcome(translated_event.sequence);
        let metrics = translated_window_metrics(translated_event.event);
        let physical_size = metrics
            .map(|metrics| {
                PhysicalSize::new(metrics.physical_size.width, metrics.physical_size.height)
            })
            .unwrap_or_else(|| PhysicalSize::new(size.width, size.height));
        let scale_changed = metrics.is_some_and(|metrics| {
            (metrics.scale_factor as f32).to_bits() != self.host.window().scale_factor().to_bits()
        });
        let duplicate_size = physical_size == self.host.window().size();
        if duplicate_size && !scale_changed {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_resize.duplicate_size_suppressed_count",
                1_u8
            );
            self.finish_input_without_damage();
            return;
        }
        if scale_changed {
            if let Some(metrics) = metrics {
                self.host
                    .window()
                    .set_scale_factor(metrics.scale_factor as f32);
            }
        }
        if !duplicate_size {
            self.host.window().set_size(physical_size.clone());
        }
        self.queue_resize_reflow((!duplicate_size).then_some(physical_size));
    }

    pub(super) fn handle_window_scale_factor_changed(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        scale_factor: f64,
        translated_event: PlatformInputTranslation,
    ) {
        self.begin_input_outcome(translated_event.sequence);
        let scale_factor =
            translated_scale_factor(translated_event.event).unwrap_or(scale_factor) as f32;
        if scale_factor.to_bits() == self.host.window().scale_factor().to_bits() {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_resize.duplicate_scale_suppressed_count",
                1_u8
            );
            self.finish_input_without_damage();
            return;
        }
        self.host.window().set_scale_factor(scale_factor);
        self.queue_resize_reflow(None);
    }

    fn queue_resize_reflow(&mut self, pending_presenter_size: Option<PhysicalSize>) {
        self.host.defer_native_resize_reflow();
        self.pending_resize_reflow_deadline = Some(Instant::now() + NATIVE_RESIZE_REFLOW_DEBOUNCE);
        if let Some(physical_size) = pending_presenter_size {
            self.pending_presenter_resize = Some((physical_size.width, physical_size.height));
        }
        let redraw =
            HostRedrawRequest::full_frame_for_scenario(UiPerfScenario::WindowResize, false);
        self.finish_input_outcome(&redraw);
        if self.queue_redraw(redraw) {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    pub(super) fn handle_window_moved(&mut self, position: WinitPhysicalPosition<i32>) {
        self.host
            .window()
            .set_position(PhysicalPosition::new(position.x, position.y));
    }
}

fn translated_window_metrics(event: Option<UiWindowInputPumpEvent>) -> Option<UiWindowMetrics> {
    match event? {
        UiWindowInputPumpEvent::Window(event) => match event.kind {
            UiWindowEventKind::Resized { metrics } => Some(metrics),
            _ => None,
        },
        UiWindowInputPumpEvent::Input(_) => None,
    }
}

fn translated_scale_factor(event: Option<UiWindowInputPumpEvent>) -> Option<f64> {
    match event? {
        UiWindowInputPumpEvent::Window(event) => match event.kind {
            UiWindowEventKind::ScaleFactorChanged { scale_factor }
            | UiWindowEventKind::BackendScaleFactorChanged { scale_factor } => Some(scale_factor),
            _ => None,
        },
        UiWindowInputPumpEvent::Input(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        layout::UiSize,
        window::{
            UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata, UiWindowInputPumpEvent,
            UiWindowMetrics, UiWindowPixelSize,
        },
    };

    use super::{translated_scale_factor, translated_window_metrics};

    #[test]
    fn surface_resize_queues_a_snapshot_present_but_defers_retained_layout() {
        let source = include_str!("resize.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("resize production source");
        let defer = production
            .find("fn queue_resize_reflow")
            .expect("resize events must share one retained reflow queue");
        let retain_latest_size = production
            .find("self.pending_presenter_resize = Some")
            .expect("surface resize must retain only the latest presenter size");
        let queue_snapshot = production
            .find("HostRedrawRequest::full_frame_for_scenario")
            .expect("surface resize must queue an interactive snapshot present");

        assert!(defer < retain_latest_size);
        assert!(retain_latest_size < queue_snapshot);
        assert!(!production.contains("presenter.resize"));
        assert!(production.contains("UiPerfScenario::WindowResize"));
        assert!(production.contains("false,"));
    }

    #[test]
    fn translated_surface_resize_preserves_the_prior_scale_factor() {
        let metrics = UiWindowMetrics::new(
            UiSize::new(1280.0, 720.0),
            UiWindowPixelSize::new(2560, 1440),
            2.0,
        );
        let event = UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            UiWindowEventMetadata::default(),
            UiWindowEventKind::Resized { metrics },
        ));

        assert_eq!(translated_window_metrics(Some(event)), Some(metrics));
    }

    #[test]
    fn translated_scale_event_drives_the_retained_window_scale() {
        let event = UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            UiWindowEventMetadata::default(),
            UiWindowEventKind::ScaleFactorChanged { scale_factor: 2.0 },
        ));

        assert_eq!(translated_scale_factor(Some(event)), Some(2.0));
    }
}
