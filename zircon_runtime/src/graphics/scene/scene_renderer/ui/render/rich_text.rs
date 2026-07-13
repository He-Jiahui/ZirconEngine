use crate::core::framework::render::{InlineBaseline, InlineObjectRef, RichParseResult, StyledRun};
use crate::core::math::Vec4;
use crate::graphics::text::rich::parse_rich_text;
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiResolvedTextLine, UiRichTextFormat, UiTextDecorations, UiTextPaintRun,
    UiTextRange, UiTextWritingMode,
};

use super::super::image::ScreenSpaceUiImageBatch;
use super::background::ScreenSpaceUiBackgroundTracker;
use super::{parse_color, push_rect, PlannedScreenSpaceUi};

pub(super) struct RichTextRunPresentation {
    pub font: Option<String>,
    pub font_family: Option<String>,
    pub font_weight: u16,
    pub font_size: f32,
    pub line_height: f32,
    pub color: [f32; 4],
    pub text_decorations: UiTextDecorations,
}

pub(super) fn parse_command_rich_text(command: &UiRenderCommand) -> Option<RichParseResult> {
    (!matches!(command.style.rich_text_format, UiRichTextFormat::Plain)).then(|| {
        parse_rich_text(
            command.text.as_deref().unwrap_or_default(),
            command.style.rich_text_format,
        )
    })
}

pub(super) fn run_for_range(parsed: &RichParseResult, range: UiTextRange) -> Option<&StyledRun> {
    if range.start == range.end {
        return None;
    }
    parsed.runs.iter().find(|run| {
        let Ok(start) = usize::try_from(run.byte_range.0) else {
            return false;
        };
        let Ok(end) = usize::try_from(run.byte_range.1) else {
            return false;
        };
        start <= range.start && range.end <= end
    })
}

pub(super) fn inline_frame(
    inline: &InlineObjectRef,
    run_frame: UiFrame,
    line: &UiResolvedTextLine,
    writing_mode: UiTextWritingMode,
) -> UiFrame {
    let inline_extent = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        line.frame.width
    } else {
        line.frame.height
    };
    let (size, baseline) = match inline {
        InlineObjectRef::Image { size, baseline, .. } => (*size, *baseline),
        InlineObjectRef::Widget { size, .. } => (*size, InlineBaseline::Baseline),
        InlineObjectRef::Icon { .. } => (
            crate::core::math::Vec2::new(inline_extent, inline_extent),
            InlineBaseline::Baseline,
        ),
    };
    if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        let x = match baseline {
            InlineBaseline::Baseline | InlineBaseline::Center => {
                line.frame.x + (line.frame.width - size.x) * 0.5
            }
            InlineBaseline::Top => line.frame.x,
            InlineBaseline::Bottom => line.frame.right() - size.x,
        };
        UiFrame::new(x, run_frame.y, size.x, size.y)
    } else {
        let y = match baseline {
            InlineBaseline::Baseline => line.frame.y + line.baseline - size.y,
            InlineBaseline::Center => line.frame.y + (line.frame.height - size.y) * 0.5,
            InlineBaseline::Top => line.frame.y,
            InlineBaseline::Bottom => line.frame.bottom() - size.y,
        };
        UiFrame::new(run_frame.x, y, size.x, size.y)
    }
}

pub(super) fn inline_layout_frame(
    command: &UiRenderCommand,
    range: UiTextRange,
) -> Option<UiFrame> {
    let layout = command.text_layout.as_ref()?;
    let line = layout.lines.iter().find(|line| {
        line.source_range.start <= range.start && range.end <= line.source_range.end
    })?;
    let parsed = parse_rich_text(command.text.as_deref()?, command.style.rich_text_format);
    let inline = run_for_range(&parsed, range)?.inline.as_ref()?;
    let visual_start = line
        .runs
        .iter()
        .find(|run| run.source_range == range)?
        .visual_range
        .start;
    let prefix = line.text.get(..visual_start)?;
    let grapheme_count = prefix.graphemes(true).count();
    let main_offset = line
        .glyph_advances
        .iter()
        .take(grapheme_count)
        .copied()
        .sum::<f32>();
    let run_frame = if matches!(layout.writing_mode, UiTextWritingMode::VerticalRl) {
        UiFrame::new(
            line.frame.x,
            line.frame.y + main_offset,
            line.frame.width,
            0.0,
        )
    } else {
        UiFrame::new(
            line.frame.x + main_offset,
            line.frame.y,
            0.0,
            line.frame.height,
        )
    };
    Some(inline_frame(inline, run_frame, line, layout.writing_mode))
}

pub(super) fn plan_inline_run(
    command: &UiRenderCommand,
    run: &UiTextPaintRun,
    rich_run: Option<&StyledRun>,
    viewport: UiFrame,
    fallback_color: [f32; 4],
    backgrounds: &ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) -> bool {
    let Some(inline) = rich_run.and_then(|rich_run| rich_run.inline.as_ref()) else {
        return false;
    };
    let Some(line) = command.text_layout.as_ref().and_then(|layout| {
        layout.lines.iter().find(|line| {
            line.source_range.start <= run.source_range.start
                && run.source_range.end <= line.source_range.end
        })
    }) else {
        return true;
    };
    let inline_frame = inline_layout_frame(command, run.source_range).unwrap_or_else(|| {
        inline_frame(
            inline,
            run.frame,
            line,
            command
                .text_layout
                .as_ref()
                .map(|layout| layout.writing_mode)
                .unwrap_or(command.style.text_writing_mode),
        )
    });
    if viewport.intersection(inline_frame).is_none() {
        return true;
    }
    match inline {
        InlineObjectRef::Image { texture, .. } => plan.images.push(ScreenSpaceUiImageBatch {
            texture: *texture,
            frame: inline_frame,
            clip_frame: command.clip_frame,
            tint: [1.0, 1.0, 1.0, command.opacity.clamp(0.0, 1.0)],
        }),
        InlineObjectRef::Icon { glyph, font } => {
            let writing_mode = command
                .text_layout
                .as_ref()
                .map(|layout| layout.writing_mode)
                .unwrap_or(command.style.text_writing_mode);
            let font_size = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
                inline_frame.width
            } else {
                inline_frame.height
            }
            .max(1.0);
            let color = rich_run
                .and_then(|rich_run| rich_run.style.color)
                .map(|color| rgba(color, command.opacity))
                .unwrap_or(fallback_color);
            super::push_text_batch(
                command,
                glyph.to_string(),
                inline_frame,
                Some(run.source_range),
                Vec::new(),
                None,
                Some(font.as_str().to_string()),
                run.font_weight,
                font_size,
                font_size,
                color,
                zircon_runtime_interface::ui::surface::UiTextAlign::Left,
                line.direction,
                writing_mode,
                zircon_runtime_interface::ui::surface::UiTextWrap::None,
                run.style.clone(),
                decorations_for_rich_run(command, rich_run),
                viewport,
                backgrounds,
                plan,
            );
        }
        InlineObjectRef::Widget { .. } => {
            let frame = viewport.intersection(inline_frame).unwrap_or(inline_frame);
            push_rect(&mut plan.vertices, frame, fallback_color, viewport);
        }
    }
    true
}

pub(super) fn prepare_text_run(
    command: &UiRenderCommand,
    run: &UiTextPaintRun,
    rich_run: Option<&StyledRun>,
    viewport: UiFrame,
    fallback_color: [f32; 4],
    plan: &mut PlannedScreenSpaceUi,
) -> RichTextRunPresentation {
    let font_size = rich_run
        .and_then(|rich_run| rich_run.style.font_size)
        .unwrap_or(run.font_size);
    let color = rich_run
        .and_then(|rich_run| rich_run.style.color)
        .map(|color| rgba(color, command.opacity))
        .or_else(|| parse_color(run.color.as_deref(), fallback_color, command.opacity))
        .unwrap_or(fallback_color);
    if let Some(background) = rich_run
        .and_then(|rich_run| rich_run.style.bg_color)
        .map(|color| rgba(color, command.opacity))
    {
        if let Some(frame) = viewport.intersection(run.frame) {
            push_rect(&mut plan.vertices, frame, background, viewport);
        }
    }
    RichTextRunPresentation {
        font: run.font.clone().or_else(|| command.style.font.clone()),
        font_family: rich_run
            .and_then(|rich_run| rich_run.style.family.as_ref())
            .map(|family| family.as_str().to_string())
            .or_else(|| run.font_family.clone())
            .or_else(|| command.style.font_family.clone()),
        font_weight: rich_run
            .and_then(|rich_run| rich_run.style.weight)
            .unwrap_or(run.font_weight),
        font_size,
        line_height: if run.font_size > 0.0 {
            run.line_height * (font_size / run.font_size)
        } else {
            run.line_height
        },
        color,
        text_decorations: decorations_for_rich_run(command, rich_run),
    }
}

pub(super) fn decorations_for_rich_run(
    command: &UiRenderCommand,
    rich_run: Option<&StyledRun>,
) -> UiTextDecorations {
    let mut decorations = command.style.text_decorations.clone();
    if let Some(rich_run) = rich_run {
        if let Some(underline) = rich_run.style.underline {
            decorations.underline = underline;
        }
        if let Some(strike) = rich_run.style.strike {
            decorations.strikethrough = strike;
        }
        if rich_run.link.is_some() {
            decorations.underline = true;
        }
    }
    decorations
}

pub(super) fn rgba(color: Vec4, opacity: f32) -> [f32; 4] {
    [
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
        (color.w * opacity).clamp(0.0, 1.0),
    ]
}
