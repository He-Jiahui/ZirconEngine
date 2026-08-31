use zircon_runtime_interface::ui::component::UiValue;

use super::super::ViewTemplateNodeData;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ViewTemplateNodePatch {
    pub(crate) selected: Option<bool>,
    pub(crate) focused: Option<bool>,
    pub(crate) surface_variant: Option<String>,
    pub(crate) text_tone: Option<String>,
}

impl ViewTemplateNodePatch {
    pub(crate) fn visual_state(
        selected: bool,
        focused: bool,
        surface_variant: impl Into<String>,
        text_tone: impl Into<String>,
    ) -> Self {
        Self {
            selected: Some(selected),
            focused: Some(focused),
            surface_variant: Some(surface_variant.into()),
            text_tone: Some(text_tone.into()),
        }
    }

    pub(crate) fn focused(mut self, focused: bool) -> Self {
        self.focused = Some(focused);
        self
    }

    pub(crate) fn text_tone(mut self, text_tone: impl Into<String>) -> Self {
        self.text_tone = Some(text_tone.into());
        self
    }

    pub(crate) fn surface_variant(mut self, surface_variant: impl Into<String>) -> Self {
        self.surface_variant = Some(surface_variant.into());
        self
    }

    pub(crate) fn resolved_against(
        &self,
        authored: &ViewTemplateNodePatch,
    ) -> ViewTemplateNodePatch {
        ViewTemplateNodePatch {
            selected: self.selected.or(authored.selected),
            focused: self.focused.or(authored.focused),
            surface_variant: self
                .surface_variant
                .clone()
                .or_else(|| authored.surface_variant.clone()),
            text_tone: self
                .text_tone
                .clone()
                .or_else(|| authored.text_tone.clone()),
        }
    }

    pub(super) fn authored(node: &ViewTemplateNodeData) -> Self {
        Self {
            selected: Some(node.selected),
            focused: Some(node.focused),
            surface_variant: Some(node.surface_variant.to_string()),
            text_tone: Some(node.text_tone.to_string()),
        }
    }

    pub(super) fn apply(&self, node: &mut ViewTemplateNodeData) {
        if let Some(selected) = self.selected {
            node.selected = selected;
        }
        if let Some(focused) = self.focused {
            node.focused = focused;
        }
        if let Some(surface_variant) = self.surface_variant.as_ref() {
            node.surface_variant = surface_variant.clone();
        }
        if let Some(text_tone) = self.text_tone.as_ref() {
            node.text_tone = text_tone.clone();
        }
    }

    pub(super) fn changed_properties<'a>(
        &'a self,
        previous: &'a Self,
    ) -> impl Iterator<Item = (&'static str, UiValue)> + 'a {
        [
            (self.selected != previous.selected)
                .then(|| ("selected", UiValue::Bool(self.selected.unwrap_or(false)))),
            (self.focused != previous.focused)
                .then(|| ("focused", UiValue::Bool(self.focused.unwrap_or(false)))),
            (self.surface_variant != previous.surface_variant).then(|| {
                (
                    "surface_variant",
                    UiValue::String(self.surface_variant.clone().unwrap_or_default()),
                )
            }),
            (self.text_tone != previous.text_tone).then(|| {
                (
                    "text_tone",
                    UiValue::String(self.text_tone.clone().unwrap_or_default()),
                )
            }),
        ]
        .into_iter()
        .flatten()
    }
}

#[cfg(test)]
#[path = "projection_patch/allocation_free_change_tests.rs"]
mod allocation_free_change_tests;
