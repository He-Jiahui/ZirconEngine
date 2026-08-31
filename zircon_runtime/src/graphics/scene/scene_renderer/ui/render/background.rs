use std::cell::Cell;

use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiRenderCommandKind};

use super::color::parse_hex_color;

const BACKGROUND_COVERAGE_EPSILON: f32 = 0.01;

#[derive(Default)]
pub(super) struct ScreenSpaceUiBackgroundTracker {
    effects: Vec<ScreenSpaceUiBackgroundEffect>,
    query_count: Cell<usize>,
    effect_visit_count: Cell<usize>,
    max_effect_visit_count: Cell<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ScreenSpaceUiBackgroundTrackerStats {
    pub(super) query_count: usize,
    pub(super) effect_visit_count: usize,
    pub(super) max_effect_visit_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ScreenSpaceUiBackgroundEffect {
    OpaqueBackground { frame: UiFrame, color: [f32; 4] },
    Blocker { frame: UiFrame },
}

impl ScreenSpaceUiBackgroundTracker {
    pub(super) fn with_framebuffer_background(viewport: UiFrame, color: Option<[f32; 4]>) -> Self {
        let mut tracker = Self::default();
        if let Some(color) = color {
            tracker
                .effects
                .push(ScreenSpaceUiBackgroundEffect::OpaqueBackground {
                    frame: viewport,
                    color,
                });
        }
        tracker
    }

    pub(super) fn observe_command(&mut self, command: &UiRenderCommand, viewport: UiFrame) {
        if command.opacity <= 0.0 {
            return;
        }

        let Some(frame) = command_visible_frame(command.frame, command.clip_frame, viewport) else {
            return;
        };
        if let Some((frame, color)) = command_opaque_fill_background(command, frame) {
            self.effects
                .push(ScreenSpaceUiBackgroundEffect::OpaqueBackground { frame, color });
            return;
        }

        if command_blocks_inherited_background(command) {
            self.effects
                .push(ScreenSpaceUiBackgroundEffect::Blocker { frame });
        }
    }

    fn color_for_frame(
        &self,
        frame: UiFrame,
        clip_frame: Option<UiFrame>,
        viewport: UiFrame,
    ) -> Option<[f32; 4]> {
        let (color, visit_count) =
            self.color_for_frame_and_visit_count(frame, clip_frame, viewport);
        self.query_count
            .set(self.query_count.get().saturating_add(1));
        self.effect_visit_count
            .set(self.effect_visit_count.get().saturating_add(visit_count));
        self.max_effect_visit_count
            .set(self.max_effect_visit_count.get().max(visit_count));
        color
    }

    fn color_for_frame_and_visit_count(
        &self,
        frame: UiFrame,
        clip_frame: Option<UiFrame>,
        viewport: UiFrame,
    ) -> (Option<[f32; 4]>, usize) {
        let Some(frame) = command_visible_frame(frame, clip_frame, viewport) else {
            return (None, 0);
        };
        let mut visit_count = 0;
        for effect in self.effects.iter().rev() {
            visit_count += 1;
            match effect {
                ScreenSpaceUiBackgroundEffect::OpaqueBackground {
                    frame: background_frame,
                    color,
                } if frame_covers(*background_frame, frame) => {
                    return (Some(*color), visit_count);
                }
                ScreenSpaceUiBackgroundEffect::Blocker {
                    frame: blocker_frame,
                } if frames_intersect(*blocker_frame, frame) => {
                    return (None, visit_count);
                }
                _ => {}
            }
        }
        (None, visit_count)
    }

    #[cfg(test)]
    pub(super) fn color_for_frame_with_visit_count(
        &self,
        frame: UiFrame,
        clip_frame: Option<UiFrame>,
        viewport: UiFrame,
    ) -> (Option<[f32; 4]>, usize) {
        self.color_for_frame_and_visit_count(frame, clip_frame, viewport)
    }

    pub(super) fn stats(&self) -> ScreenSpaceUiBackgroundTrackerStats {
        ScreenSpaceUiBackgroundTrackerStats {
            query_count: self.query_count.get(),
            effect_visit_count: self.effect_visit_count.get(),
            max_effect_visit_count: self.max_effect_visit_count.get(),
        }
    }

    pub(super) fn effect_count(&self) -> usize {
        self.effects.len()
    }

    pub(super) fn effects_since(&self, start: usize) -> &[ScreenSpaceUiBackgroundEffect] {
        self.effects.get(start..).unwrap_or_default()
    }

    pub(super) fn replay_effects(&mut self, effects: &[ScreenSpaceUiBackgroundEffect]) {
        self.effects.extend_from_slice(effects);
    }
}

pub(super) fn text_batch_background_color(
    command: &UiRenderCommand,
    frame: UiFrame,
    viewport: UiFrame,
    backgrounds: &ScreenSpaceUiBackgroundTracker,
) -> Option<[f32; 4]> {
    if command.style.background_color.is_some() {
        return command_opaque_background_color(command);
    }
    backgrounds.color_for_frame(frame, command.clip_frame, viewport)
}

fn command_visible_frame(
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: UiFrame,
) -> Option<UiFrame> {
    let frame = viewport.intersection(frame)?;
    match clip_frame {
        Some(clip) => frame.intersection(clip),
        None => Some(frame),
    }
}

fn command_opaque_fill_background(
    command: &UiRenderCommand,
    frame: UiFrame,
) -> Option<(UiFrame, [f32; 4])> {
    if command.text.as_ref().is_some_and(|text| !text.is_empty())
        || command.image.is_some()
        || matches!(
            command.kind,
            UiRenderCommandKind::Image | UiRenderCommandKind::Text
        )
    {
        return None;
    }

    let color = command_opaque_background_color(command)?;
    inset_frame(frame, command.style.border_width.max(0.0)).map(|frame| (frame, color))
}

fn command_blocks_inherited_background(command: &UiRenderCommand) -> bool {
    command.style.background_color.is_some()
        || command.style.border_color.is_some()
        || command.style.border_width > 0.0
        || matches!(
            command.kind,
            UiRenderCommandKind::Quad | UiRenderCommandKind::Image
        )
        || command.image.is_some()
        || command.text.as_ref().is_some_and(|text| !text.is_empty())
}

fn command_opaque_background_color(command: &UiRenderCommand) -> Option<[f32; 4]> {
    let background = parse_hex_color(command.style.background_color.as_deref()?, command.opacity)?;
    (background[3] >= 1.0).then_some([background[0], background[1], background[2], 1.0])
}

fn inset_frame(frame: UiFrame, inset: f32) -> Option<UiFrame> {
    if inset <= 0.0 {
        return Some(frame);
    }
    let width = frame.width - inset * 2.0;
    let height = frame.height - inset * 2.0;
    (width > 0.0 && height > 0.0).then_some(UiFrame::new(
        frame.x + inset,
        frame.y + inset,
        width,
        height,
    ))
}

fn frame_covers(outer: UiFrame, inner: UiFrame) -> bool {
    outer.x <= inner.x + BACKGROUND_COVERAGE_EPSILON
        && outer.y <= inner.y + BACKGROUND_COVERAGE_EPSILON
        && outer.right() + BACKGROUND_COVERAGE_EPSILON >= inner.right()
        && outer.bottom() + BACKGROUND_COVERAGE_EPSILON >= inner.bottom()
}

fn frames_intersect(a: UiFrame, b: UiFrame) -> bool {
    a.x < b.right()
        && a.right() > b.x
        && a.y < b.bottom()
        && a.bottom() > b.y
        && a.width > 0.0
        && a.height > 0.0
        && b.width > 0.0
        && b.height > 0.0
}
