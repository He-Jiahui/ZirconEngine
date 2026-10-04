use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedTextLayout, UiTextWritingMode},
};

use crate::text::layout_geometry::{finite_f32_or_geometry, finite_sum, FiniteGeometryAccumulator};
use crate::text::{CompiledRichText, InlineObjectRef, RichInlineWidgetSlotId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiInlineWidgetBinding {
    pub(crate) slot: RichInlineWidgetSlotId,
    pub(crate) frame: Option<UiFrame>,
    pub(crate) valid: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiInlineWidgetLayout {
    bindings: Vec<UiInlineWidgetBinding>,
}

impl UiInlineWidgetLayout {
    pub(crate) fn bindings(&self) -> &[UiInlineWidgetBinding] {
        &self.bindings
    }
}

#[derive(Clone, Copy)]
struct WidgetRunSpec {
    binding_index: usize,
    size: crate::core::math::Vec2,
}

#[derive(Clone, Copy)]
struct WidgetBindingState {
    slot: RichInlineWidgetSlotId,
    declaration_count: usize,
    visible_count: usize,
    frame: Option<UiFrame>,
}

pub(crate) fn inline_widget_layout_from_compiled(
    compiled: &CompiledRichText,
    layout: Option<&UiResolvedTextLayout>,
) -> Option<UiInlineWidgetLayout> {
    let mut states = Vec::<WidgetBindingState>::new();
    let mut state_by_slot = HashMap::<RichInlineWidgetSlotId, usize>::new();
    let mut spec_by_source_range = HashMap::<(usize, usize), WidgetRunSpec>::new();

    for run in compiled.inline_runs() {
        let Some(InlineObjectRef::Widget { slot, size }) = run.inline.as_ref() else {
            continue;
        };
        let binding_index = match state_by_slot.get(slot).copied() {
            Some(index) => {
                states[index].declaration_count = states[index].declaration_count.saturating_add(1);
                index
            }
            None => {
                let index = states.len();
                states.push(WidgetBindingState {
                    slot: *slot,
                    declaration_count: 1,
                    visible_count: 0,
                    frame: None,
                });
                state_by_slot.insert(*slot, index);
                index
            }
        };
        let range = (run.byte_range.0 as usize, run.byte_range.1 as usize);
        if let Some(previous) = spec_by_source_range.insert(
            range,
            WidgetRunSpec {
                binding_index,
                size: *size,
            },
        ) {
            states[previous.binding_index].declaration_count = states[previous.binding_index]
                .declaration_count
                .saturating_add(1);
            states[binding_index].declaration_count =
                states[binding_index].declaration_count.saturating_add(1);
        }
    }

    if states.is_empty() {
        return None;
    }

    if let Some(layout) = layout {
        resolve_visible_widget_frames(layout, &spec_by_source_range, &mut states);
    }

    Some(UiInlineWidgetLayout {
        // 声明唯一性决定绑定有效性；没有唯一可见片段时仍保留有效绑定，交给布局层隐藏子树。
        bindings: states
            .into_iter()
            .map(|state| UiInlineWidgetBinding {
                slot: state.slot,
                frame: (state.declaration_count == 1 && state.visible_count == 1)
                    .then_some(state.frame)
                    .flatten(),
                valid: state.declaration_count == 1,
            })
            .collect(),
    })
}

fn resolve_visible_widget_frames(
    layout: &UiResolvedTextLayout,
    specs: &HashMap<(usize, usize), WidgetRunSpec>,
    states: &mut [WidgetBindingState],
) {
    for line in &layout.lines {
        let mut graphemes = line.text.grapheme_indices(true).peekable();
        let mut advance_index = 0_usize;
        let mut main_offset = FiniteGeometryAccumulator::default();

        for run in &line.runs {
            let Some(offset) = advance_to_visual_start(
                &mut graphemes,
                &line.glyph_advances,
                &mut advance_index,
                &mut main_offset,
                run.visual_range.start,
                line.text.len(),
            ) else {
                continue;
            };
            let Some(spec) = specs.get(&(run.source_range.start, run.source_range.end)) else {
                continue;
            };
            let Some(state) = states.get_mut(spec.binding_index) else {
                continue;
            };
            state.visible_count = state.visible_count.saturating_add(1);
            if state.visible_count == 1 {
                state.frame = Some(widget_frame(
                    line.frame,
                    line.baseline,
                    offset,
                    spec.size,
                    layout.writing_mode,
                ));
            } else {
                state.frame = None;
            }
        }
    }
}

fn advance_to_visual_start<'a>(
    graphemes: &mut std::iter::Peekable<unicode_segmentation::GraphemeIndices<'a>>,
    advances: &[f32],
    advance_index: &mut usize,
    main_offset: &mut FiniteGeometryAccumulator,
    visual_start: usize,
    text_len: usize,
) -> Option<f32> {
    while graphemes
        .peek()
        .is_some_and(|(start, _)| *start < visual_start)
    {
        let _ = graphemes.next();
        let advance = advances.get(*advance_index).copied()?;
        if !advance.is_finite() {
            return None;
        }
        main_offset.add(advance);
        *advance_index = (*advance_index).saturating_add(1);
    }
    let at_boundary = graphemes
        .peek()
        .is_some_and(|(start, _)| *start == visual_start)
        || visual_start == text_len;
    at_boundary.then(|| main_offset.value())
}

fn widget_frame(
    line_frame: UiFrame,
    baseline: f32,
    main_offset: f32,
    size: crate::core::math::Vec2,
    writing_mode: UiTextWritingMode,
) -> UiFrame {
    if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        let x_candidate = line_frame.x + (line_frame.width - size.x) * 0.5;
        let x_exact =
            f64::from(line_frame.x) + (f64::from(line_frame.width) - f64::from(size.x)) * 0.5;
        UiFrame::new(
            finite_f32_or_geometry(x_candidate, x_exact),
            finite_sum([line_frame.y, main_offset]),
            size.x,
            size.y,
        )
    } else {
        UiFrame::new(
            finite_sum([line_frame.x, main_offset]),
            finite_sum([line_frame.y, baseline, -size.y]),
            size.x,
            size.y,
        )
    }
}

#[cfg(test)]
#[path = "tests/inline_widget.rs"]
mod tests;
