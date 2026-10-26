use super::super::template_component_family::{is_component_family, TemplateComponentFamily};
use super::{retained_pane_input::find_retained_input_pane, UiHostWindow};
use crate::ui::retained_host::host_contract::data::{
    HostTextInputFocusData, HostWindowPresentationData, PaneData, UiAssetPaletteInputTarget,
};
use std::{rc::Rc, sync::Arc};

fn palette_search_pane<'a>(
    presentation: &'a HostWindowPresentationData,
    target: &UiAssetPaletteInputTarget,
) -> Option<&'a PaneData> {
    let (pane, content) = find_retained_input_pane(presentation, target.instance_id())?;
    if pane.kind.as_str() != "UiAssetEditor"
        || !content.width.is_finite()
        || !content.height.is_finite()
        || content.width <= 0.0
        || content.height <= 0.0
    {
        return None;
    }
    let current = pane.ui_asset.collections.palette_search.as_ref()?;
    current.same_source(target.snapshot()).then_some(pane)
}

pub(crate) fn palette_search_focus_current(
    presentation: &HostWindowPresentationData,
    focus: &HostTextInputFocusData,
) -> bool {
    let Some(target) = focus.palette_search_target.as_ref() else {
        return false;
    };
    let Some(pane) = palette_search_pane(presentation, target) else {
        return false;
    };
    if focus.dispatch_kind.as_str() != "ui_asset_palette_search"
        || focus.control_id.is_empty()
        || focus.edit_action_id.is_empty()
    {
        return false;
    }
    // The pane/source can survive removal or replacement of its actual field.
    // Follow the same model chosen by paint/hit testing, including template overrides.
    pane.template_nodes().is_some_and(|nodes| {
        nodes.iter().any(|node| {
            node.control_id == focus.control_id
                && node.dispatch_kind == focus.dispatch_kind
                && node.edit_action_id == focus.edit_action_id
                && node.commit_action_id == focus.commit_action_id
                // A hot-replaced node can keep its ID/routes while no longer
                // taking text input. Reuse the native hit/paint family authority.
                && is_component_family(node, TemplateComponentFamily::TextInput)
                && !node.disabled
                && node.frame.x.is_finite()
                && node.frame.y.is_finite()
                && node.frame.width.is_finite()
                && node.frame.height.is_finite()
                && node.frame.width > 0.0
                && node.frame.height > 0.0
        })
    })
}

impl UiHostWindow {
    /// Retire only the Palette field, without invoking app callbacks or publishing layout.
    pub(crate) fn retire_palette_search_input(&self) -> bool {
        let mut state = self.state.borrow_mut();
        if state.text_input_focus.palette_search_target.is_none() {
            return false;
        }
        state.replace_text_input_focus(Default::default())
    }

    pub(crate) fn same_host_instance(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }

    pub(crate) fn palette_search_input_is_current(
        &self,
        target: &UiAssetPaletteInputTarget,
        event: &str,
    ) -> bool {
        if !self.window().is_visible() {
            return false;
        }
        let generation = self.get_host_presentation_generation();
        let Some(pane) = palette_search_pane(generation.structure(), target) else {
            return false;
        };
        match event {
            // Query/page allocations change on every keystroke; keep the one focused
            // field alive only while its actual source allocation and pane remain valid.
            "query" => {
                generation.text_input_focus().palette_search_target.as_ref() == Some(target)
                    && palette_search_focus_current(
                        generation.structure(),
                        generation.text_input_focus(),
                    )
            }
            "selected" | "page" => pane
                .body_surface_frame
                .as_ref()
                .is_some_and(|frame| Arc::ptr_eq(frame, target.shell_frame())),
            _ => false,
        }
    }
}

#[cfg(test)]
#[path = "tests/palette_search_input.rs"]
mod tests;
