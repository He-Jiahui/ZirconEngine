use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiRenderCommandKind};

use super::color::parse_hex_color;

const BACKGROUND_COVERAGE_EPSILON: f32 = 0.01;

#[derive(Default)]
pub(super) struct ScreenSpaceUiBackgroundTracker {
    order: u64,
    candidates: Vec<ScreenSpaceUiOpaqueBackground>,
    blockers: Vec<ScreenSpaceUiBackgroundBlocker>,
}

#[derive(Clone, Copy)]
struct ScreenSpaceUiOpaqueBackground {
    order: u64,
    frame: UiFrame,
    color: [f32; 4],
}

#[derive(Clone, Copy)]
struct ScreenSpaceUiBackgroundBlocker {
    order: u64,
    frame: UiFrame,
}

impl ScreenSpaceUiBackgroundTracker {
    pub(super) fn with_framebuffer_background(viewport: UiFrame, color: Option<[f32; 4]>) -> Self {
        let mut tracker = Self::default();
        if let Some(color) = color {
            tracker.candidates.push(ScreenSpaceUiOpaqueBackground {
                order: 0,
                frame: viewport,
                color,
            });
        }
        tracker
    }

    pub(super) fn observe_command(&mut self, command: &UiRenderCommand, viewport: UiFrame) {
        self.order = self.order.saturating_add(1);
        if command.opacity <= 0.0 {
            return;
        }

        let Some(frame) = command_visible_frame(command.frame, command.clip_frame, viewport) else {
            return;
        };
        if let Some((frame, color)) = command_opaque_fill_background(command, frame) {
            self.candidates.push(ScreenSpaceUiOpaqueBackground {
                order: self.order,
                frame,
                color,
            });
            return;
        }

        if command_blocks_inherited_background(command) {
            self.blockers.push(ScreenSpaceUiBackgroundBlocker {
                order: self.order,
                frame,
            });
        }
    }

    fn color_for_frame(
        &self,
        frame: UiFrame,
        clip_frame: Option<UiFrame>,
        viewport: UiFrame,
    ) -> Option<[f32; 4]> {
        let frame = command_visible_frame(frame, clip_frame, viewport)?;
        self.candidates
            .iter()
            .rev()
            .find(|background| {
                frame_covers(background.frame, frame)
                    && !self.blockers.iter().any(|blocker| {
                        blocker.order > background.order && frames_intersect(blocker.frame, frame)
                    })
            })
            .map(|background| background.color)
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
