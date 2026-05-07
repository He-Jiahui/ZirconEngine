use std::num::NonZeroU32;
use std::sync::Arc;

use slint::Model;
use softbuffer::{Context, Rect, Surface};
use winit::window::Window;
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};

use super::data::{FrameRect, HostWindowPresentationData};
use super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::painter::{
    debug_refresh_overlay_frame, paint_host_frame, presentation_top_bar_frame,
    repaint_host_frame_region, union_frames, HostRgbaFrame,
};

pub(super) struct SoftbufferHostPresenter {
    #[allow(dead_code)]
    context: Context<Arc<dyn Window>>,
    surface: Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: (u32, u32),
    backbuffer: Option<HostRgbaFrame>,
    diagnostics: HostRefreshDiagnostics,
    last_debug_overlay_text: Option<String>,
    last_logged_presentation: Option<String>,
    last_logged_size: Option<(u32, u32)>,
}

#[derive(Clone, Debug, PartialEq)]
struct RepaintOutcome {
    damage: Option<FrameRect>,
    painted_pixels: u64,
    full_paint: bool,
    region_paint: bool,
}

struct PlannedPresent {
    presentation: HostWindowPresentationData,
    damage: Option<FrameRect>,
    diagnostics: HostRefreshDiagnostics,
    overlay_text: String,
}

impl SoftbufferHostPresenter {
    pub(super) fn new(window: Arc<dyn Window>) -> Result<Self, softbuffer::SoftBufferError> {
        let context = Context::new(window.clone())?;
        let mut surface = Surface::new(&context, window.clone())?;
        let size = current_window_size(window.as_ref());
        resize_surface(&mut surface, size)?;
        Ok(Self {
            context,
            surface,
            size,
            backbuffer: None,
            diagnostics: HostRefreshDiagnostics::default(),
            last_debug_overlay_text: None,
            last_logged_presentation: None,
            last_logged_size: None,
        })
    }

    pub(super) fn resize(&mut self, size: (u32, u32)) -> Result<(), softbuffer::SoftBufferError> {
        let size = clamp_size(size);
        resize_surface(&mut self.surface, size)?;
        self.size = size;
        self.backbuffer = None;
        self.last_debug_overlay_text = None;
        Ok(())
    }

    pub(super) fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> Result<HostRefreshDiagnostics, softbuffer::SoftBufferError> {
        let size = current_window_size(self.surface.window().as_ref());
        if self.size != size {
            self.resize(size)?;
        }

        let planned = plan_present_for_diagnostics(
            &self.diagnostics,
            self.can_region_repaint(),
            self.last_debug_overlay_text.as_deref(),
            presentation,
            damage,
            invalidation,
            size,
        );
        let outcome = self.repaint_backbuffer(&planned.presentation, planned.damage, size);
        debug_assert_eq!(
            planned.diagnostics.painted_pixel_count,
            self.diagnostics
                .painted_pixel_count
                .saturating_add(outcome.painted_pixels)
        );
        self.last_debug_overlay_text = Some(planned.overlay_text);
        self.diagnostics = planned.diagnostics;
        if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
            let summary = presentation_summary(&planned.presentation);
            if self.diagnostics.present_count <= 8
                || self.last_logged_size != Some(size)
                || self.last_logged_presentation.as_deref() != Some(summary.as_str())
            {
                write_diagnostic_log(
                    "editor_host_presenter",
                    format!(
                        "present frame={} frame_size={}x{} damage={} painted_pixels={} full_paints={} region_paints={} total_painted_pixels={} {}",
                        self.diagnostics.present_count,
                        size.0,
                        size.1,
                        outcome.damage
                            .as_ref()
                            .map(frame_summary)
                            .unwrap_or_else(|| "full".to_string()),
                        outcome.painted_pixels,
                        self.diagnostics.full_paint_count,
                        self.diagnostics.region_paint_count,
                        self.diagnostics.painted_pixel_count,
                        summary
                    ),
                );
                self.last_logged_size = Some(size);
                self.last_logged_presentation = Some(summary);
            }
        }
        let frame = self
            .backbuffer
            .as_ref()
            .expect("presenter repaint path always creates a backbuffer");
        let window = self.surface.window().clone();
        let mut buffer = self.surface.buffer_mut()?;
        copy_rgba_to_softbuffer(frame, &mut *buffer, outcome.damage.as_ref(), size);

        window.pre_present_notify();
        let result = if let Some(damage) = softbuffer_damage_rect(outcome.damage.as_ref(), size) {
            buffer.present_with_damage(&[damage])
        } else {
            buffer.present()
        };
        result?;
        Ok(self
            .diagnostics_snapshot()
            .with_invalidation_diagnostics(invalidation))
    }

    pub(super) fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics {
        self.diagnostics.clone()
    }

    fn can_region_repaint(&self) -> bool {
        self.backbuffer
            .as_ref()
            .is_some_and(|frame| frame.width() == self.size.0 && frame.height() == self.size.1)
    }

    fn repaint_backbuffer(
        &mut self,
        presentation: &HostWindowPresentationData,
        damage: Option<FrameRect>,
        size: (u32, u32),
    ) -> RepaintOutcome {
        if self.can_region_repaint() {
            if let Some(damage) = damage {
                if let Some(frame) = self.backbuffer.as_mut() {
                    if let Some(damage) = repaint_host_frame_region(frame, presentation, &damage) {
                        return RepaintOutcome {
                            painted_pixels: damage_pixel_count(&damage, size),
                            damage: Some(damage),
                            full_paint: false,
                            region_paint: true,
                        };
                    }
                }
            }
        }

        self.backbuffer = Some(paint_host_frame(self.size.0, self.size.1, presentation));
        RepaintOutcome {
            damage: None,
            painted_pixels: (size.0 as u64) * (size.1 as u64),
            full_paint: true,
            region_paint: false,
        }
    }
}

fn plan_present_for_diagnostics(
    current: &HostRefreshDiagnostics,
    can_region_repaint: bool,
    last_debug_overlay_text: Option<&str>,
    presentation: &HostWindowPresentationData,
    damage: Option<FrameRect>,
    invalidation: HostInvalidationDiagnostics,
    size: (u32, u32),
) -> PlannedPresent {
    let mut damage = if can_region_repaint
        && damage
            .as_ref()
            .is_some_and(|damage| pixel_bounds(damage, size).is_some())
    {
        damage
    } else {
        None
    };

    // The overlay text includes the painted pixel total, and text width can expand
    // the region damage. Iterate until the text and expanded damage describe the
    // same present so the same-frame overlay matches the recorded diagnostics.
    for _ in 0..8 {
        let outcome = repaint_outcome_for_damage(damage.clone(), size);
        let mut diagnostics = current.clone();
        diagnostics.record_present(
            outcome.painted_pixels,
            outcome.full_paint,
            outcome.region_paint,
        );
        let overlay_text = diagnostics
            .clone()
            .with_invalidation_diagnostics(invalidation)
            .overlay_text();
        let expanded_damage = if outcome.region_paint {
            damage_with_debug_overlay(
                damage.clone(),
                last_debug_overlay_text,
                &overlay_text,
                size,
                presentation,
            )
        } else {
            None
        };
        if expanded_damage == damage {
            let mut presentation = presentation.clone();
            presentation.host_shell.debug_refresh_rate = overlay_text.clone().into();
            return PlannedPresent {
                presentation,
                damage: expanded_damage,
                diagnostics,
                overlay_text,
            };
        }
        damage = expanded_damage;
    }

    let outcome = repaint_outcome_for_damage(damage.clone(), size);
    let mut diagnostics = current.clone();
    diagnostics.record_present(
        outcome.painted_pixels,
        outcome.full_paint,
        outcome.region_paint,
    );
    let overlay_text = diagnostics
        .clone()
        .with_invalidation_diagnostics(invalidation)
        .overlay_text();
    let mut presentation = presentation.clone();
    presentation.host_shell.debug_refresh_rate = overlay_text.clone().into();
    PlannedPresent {
        presentation,
        damage,
        diagnostics,
        overlay_text,
    }
}

fn repaint_outcome_for_damage(damage: Option<FrameRect>, size: (u32, u32)) -> RepaintOutcome {
    if let Some(damage) = damage {
        return RepaintOutcome {
            painted_pixels: damage_pixel_count(&damage, size),
            damage: Some(damage),
            full_paint: false,
            region_paint: true,
        };
    }

    RepaintOutcome {
        damage: None,
        painted_pixels: (size.0 as u64) * (size.1 as u64),
        full_paint: true,
        region_paint: false,
    }
}

fn damage_with_debug_overlay(
    damage: Option<FrameRect>,
    last_debug_overlay_text: Option<&str>,
    debug_overlay_text: &str,
    size: (u32, u32),
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let damage = damage?;
    if last_debug_overlay_text == Some(debug_overlay_text) {
        return Some(damage);
    }
    let overlay = debug_refresh_overlay_frame(
        &presentation_top_bar_frame(size.0, size.1, presentation),
        debug_overlay_text,
    )?;
    Some(union_frames(&damage, &overlay))
}

fn copy_rgba_to_softbuffer(
    frame: &HostRgbaFrame,
    buffer: &mut [u32],
    damage: Option<&FrameRect>,
    size: (u32, u32),
) {
    let (x0, y0, x1, y1) = damage
        .and_then(|damage| pixel_bounds(damage, size))
        .unwrap_or((0, 0, size.0, size.1));
    let width = size.0 as usize;
    for y in y0..y1 {
        let row_start = y as usize * width;
        for x in x0..x1 {
            let pixel_index = row_start + x as usize;
            let byte_offset = pixel_index * 4;
            let rgba = &frame.as_bytes()[byte_offset..byte_offset + 4];
            let red = rgba[0] as u32;
            let green = rgba[1] as u32;
            let blue = rgba[2] as u32;
            if let Some(pixel) = buffer.get_mut(pixel_index) {
                *pixel = (red << 16) | (green << 8) | blue;
            }
        }
    }
}

fn softbuffer_damage_rect(frame: Option<&FrameRect>, size: (u32, u32)) -> Option<Rect> {
    let frame = frame?;
    let (x0, y0, x1, y1) = pixel_bounds(frame, size)?;
    Some(Rect {
        x: x0,
        y: y0,
        width: NonZeroU32::new(x1.saturating_sub(x0))?,
        height: NonZeroU32::new(y1.saturating_sub(y0))?,
    })
}

fn pixel_bounds(frame: &FrameRect, size: (u32, u32)) -> Option<(u32, u32, u32, u32)> {
    let x0 = frame.x.floor().max(0.0).min(size.0 as f32) as u32;
    let y0 = frame.y.floor().max(0.0).min(size.1 as f32) as u32;
    let x1 = (frame.x + frame.width).ceil().max(0.0).min(size.0 as f32) as u32;
    let y1 = (frame.y + frame.height).ceil().max(0.0).min(size.1 as f32) as u32;
    (x0 < x1 && y0 < y1).then_some((x0, y0, x1, y1))
}

fn damage_pixel_count(frame: &FrameRect, size: (u32, u32)) -> u64 {
    pixel_bounds(frame, size)
        .map(|(x0, y0, x1, y1)| x1.saturating_sub(x0) as u64 * y1.saturating_sub(y0) as u64)
        .unwrap_or(0)
}

fn presentation_summary(presentation: &HostWindowPresentationData) -> String {
    let layout = &presentation.host_layout;
    let scene = &presentation.host_scene_data;
    format!(
        "project_path={} viewport_label={} status={} center={} status_bar={} document={} viewport={} left={} right={} bottom={} page_tabs={} document_tabs={} left_tabs={} right_tabs={} bottom_tabs={} floating_windows={} document_pane_kind={} left_pane_kind={} right_pane_kind={} bottom_pane_kind={}",
        presentation.host_shell.project_path,
        presentation.host_shell.viewport_label,
        presentation.host_shell.status_secondary,
        frame_summary(&layout.center_band_frame),
        frame_summary(&layout.status_bar_frame),
        frame_summary(&layout.document_region_frame),
        frame_summary(&layout.viewport_content_frame),
        frame_summary(&layout.left_region_frame),
        frame_summary(&layout.right_region_frame),
        frame_summary(&layout.bottom_region_frame),
        scene.page_chrome.tabs.row_count(),
        scene.document_dock.tabs.row_count(),
        scene.left_dock.tabs.row_count(),
        scene.right_dock.tabs.row_count(),
        scene.bottom_dock.tabs.row_count(),
        scene.floating_layer.floating_windows.row_count(),
        scene.document_dock.pane.kind,
        scene.left_dock.pane.kind,
        scene.right_dock.pane.kind,
        scene.bottom_dock.pane.kind,
    )
}

fn frame_summary(frame: &super::data::FrameRect) -> String {
    format!(
        "{:.1},{:.1},{:.1},{:.1}",
        frame.x, frame.y, frame.width, frame.height
    )
}

fn current_window_size(window: &dyn Window) -> (u32, u32) {
    let size = window.surface_size();
    clamp_size((size.width, size.height))
}

fn resize_surface(
    surface: &mut Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: (u32, u32),
) -> Result<(), softbuffer::SoftBufferError> {
    surface.resize(non_zero(size.0), non_zero(size.1))
}

fn clamp_size(size: (u32, u32)) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).expect("value is clamped to non-zero")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region_damage() -> FrameRect {
        FrameRect {
            x: 10.0,
            y: 80.0,
            width: 20.0,
            height: 10.0,
        }
    }

    fn top_bar_probe_damage() -> FrameRect {
        FrameRect {
            x: 10.0,
            y: 40.0,
            width: 20.0,
            height: 2.0,
        }
    }

    fn presentation_with_top_bar_height(height: f32) -> HostWindowPresentationData {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_layout.center_band_frame = FrameRect {
            x: 0.0,
            y: height,
            width: 200.0,
            height: 120.0 - height,
        };
        presentation
    }

    #[test]
    fn region_copy_updates_only_damaged_softbuffer_pixels() {
        let mut frame = HostRgbaFrame::filled(4, 3, [0, 0, 0, 255]);
        let damage = FrameRect {
            x: 1.0,
            y: 1.0,
            width: 2.0,
            height: 1.0,
        };
        frame.fill_rect(&damage, [255, 32, 8, 255]);
        let mut buffer = vec![0x00ff00; 12];

        copy_rgba_to_softbuffer(&frame, &mut buffer, Some(&damage), (4, 3));

        assert_eq!(buffer[5], 0xff2008);
        assert_eq!(buffer[6], 0xff2008);
        for (index, pixel) in buffer.iter().enumerate() {
            if index != 5 && index != 6 {
                assert_eq!(*pixel, 0x00ff00, "pixel {index} should remain untouched");
            }
        }
    }

    #[test]
    fn softbuffer_damage_rect_clamps_to_surface_bounds() {
        let damage = FrameRect {
            x: -4.2,
            y: 1.2,
            width: 12.6,
            height: 3.4,
        };

        let rect = softbuffer_damage_rect(Some(&damage), (8, 4))
            .expect("damage should overlap the surface");

        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 1);
        assert_eq!(rect.width.get(), 8);
        assert_eq!(rect.height.get(), 3);
    }

    #[test]
    fn overlay_text_change_expands_region_damage_without_full_repaint() {
        let damage = region_damage();

        let expanded = damage_with_debug_overlay(
            Some(damage.clone()),
            Some("FPS 59"),
            "FPS 60",
            (200, 120),
            &HostWindowPresentationData::default(),
        )
        .expect("changed overlay should keep region repaint damage");

        assert_eq!(expanded.x, 10.0);
        assert_eq!(expanded.y, 6.0);
        assert!(expanded.width > damage.width);
        assert!(expanded.height > damage.height);
    }

    #[test]
    fn unchanged_overlay_text_keeps_existing_region_damage() {
        let damage = region_damage();

        let unchanged = damage_with_debug_overlay(
            Some(damage.clone()),
            Some("FPS 60"),
            "FPS 60",
            (200, 120),
            &HostWindowPresentationData::default(),
        );

        assert_eq!(unchanged, Some(damage));
    }

    #[test]
    fn overlay_text_change_does_not_turn_full_repaint_into_region_damage() {
        let damage = damage_with_debug_overlay(
            None,
            Some("FPS 59"),
            "FPS 60",
            (200, 120),
            &HostWindowPresentationData::default(),
        );

        assert_eq!(damage, None);
    }

    #[test]
    fn overlay_text_change_expands_region_damage_to_presentation_top_bar_height() {
        let damage = top_bar_probe_damage();
        let presentation = presentation_with_top_bar_height(58.0);

        let expanded = damage_with_debug_overlay(
            Some(damage.clone()),
            Some("FPS 59"),
            "FPS 60",
            (200, 120),
            &presentation,
        )
        .expect("changed overlay should keep region repaint damage");

        assert_eq!(expanded.y, 6.0);
        assert_eq!(expanded.height, 46.0);
    }

    #[test]
    fn presenter_diagnostics_plan_same_frame_overlay_pixels_match_expanded_region_damage() {
        use super::super::diagnostics::STARTUP_REFRESH_DIAGNOSTICS_OVERLAY;

        let planned = plan_present_for_diagnostics(
            &HostRefreshDiagnostics::default(),
            true,
            Some(STARTUP_REFRESH_DIAGNOSTICS_OVERLAY),
            &presentation_with_top_bar_height(58.0),
            Some(top_bar_probe_damage()),
            HostInvalidationDiagnostics::default(),
            (200, 120),
        );
        let damage = planned
            .damage
            .as_ref()
            .expect("changed overlay text should expand region damage");
        let expected_pixels = damage_pixel_count(damage, (200, 120));

        assert_eq!(planned.diagnostics.present_count, 1);
        assert_eq!(planned.diagnostics.full_paint_count, 0);
        assert_eq!(planned.diagnostics.region_paint_count, 1);
        assert_eq!(planned.diagnostics.painted_pixel_count, expected_pixels);
        assert_eq!(damage.y, 6.0);
        assert_eq!(damage.height, 46.0);
        assert!(
            planned
                .overlay_text
                .contains(&format!("pixels {expected_pixels}")),
            "overlay text should report the same painted pixels as diagnostics: {}",
            planned.overlay_text
        );
        assert_eq!(
            planned.presentation.host_shell.debug_refresh_rate.as_str(),
            planned.overlay_text
        );
    }

    #[test]
    fn presenter_diagnostics_plan_full_repaint_records_full_pixels_in_same_frame_overlay() {
        let planned = plan_present_for_diagnostics(
            &HostRefreshDiagnostics::default(),
            false,
            None,
            &HostWindowPresentationData::default(),
            Some(region_damage()),
            HostInvalidationDiagnostics {
                slow_path_rebuild_count: 2,
                render_rebuild_count: 3,
                paint_only_request_count: 4,
            },
            (200, 120),
        );

        assert_eq!(planned.damage, None);
        assert_eq!(planned.diagnostics.full_paint_count, 1);
        assert_eq!(planned.diagnostics.region_paint_count, 0);
        assert_eq!(planned.diagnostics.painted_pixel_count, 24_000);
        assert!(planned.overlay_text.contains("pixels 24000"));
        assert!(planned.overlay_text.contains("slow 2"));
        assert!(planned.overlay_text.contains("render 3"));
        assert!(planned.overlay_text.contains("paint-only 4"));
    }
}
