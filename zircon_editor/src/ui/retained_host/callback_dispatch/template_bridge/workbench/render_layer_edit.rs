use zircon_runtime_interface::ui::{binding::UiBindingValue, component::UiValue};

use crate::core::editor_event::InspectorFieldChange;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const RENDER_LAYER_MASK_CONTROL: &str = "WorkbenchInspectorRenderLayerMask";
const RENDER_LAYER_MASK_EDIT: &str = "Inspector/RenderLayerMaskEdit";
const RENDER_LAYER_MASK_COMMIT: &str = "Inspector/RenderLayerMaskCommit";
const RENDER_LAYER_MASK_FIELD: &str = "zircon_runtime::scene::components::RenderLayerMask.mask";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn render_layer_mask_commit_binding(
        &self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Result<Option<EditorUiBinding>, String> {
        if binding_id != RENDER_LAYER_MASK_COMMIT
            || (!control_id.is_empty() && control_id != RENDER_LAYER_MASK_CONTROL)
            || !self.has_control(RENDER_LAYER_MASK_CONTROL)
        {
            return Ok(None);
        }
        let mask = parse_render_layer_mask(value)?;
        Ok(Some(EditorUiBinding::new(
            "Inspector",
            "RenderLayerMaskCommit",
            EditorUiEventKind::Submit,
            EditorUiBindingPayload::inspector_field_batch(
                "entity://selected",
                [InspectorFieldChange::new(
                    RENDER_LAYER_MASK_FIELD,
                    UiBindingValue::Unsigned(u64::from(mask)),
                )],
            ),
        )))
    }

    pub(crate) fn edit_inspector_render_layer_mask(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        if !matches!(
            binding_id,
            RENDER_LAYER_MASK_EDIT | RENDER_LAYER_MASK_COMMIT
        ) {
            return Ok(None);
        }
        if (!control_id.is_empty() && control_id != RENDER_LAYER_MASK_CONTROL)
            || !self.has_control(RENDER_LAYER_MASK_CONTROL)
        {
            return Ok(Some(false));
        }

        self.mutate_control_property(
            RENDER_LAYER_MASK_CONTROL,
            "value",
            UiValue::String(value.trim().to_string()),
        )?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(Some(true))
    }
}

fn parse_render_layer_mask(value: &str) -> Result<u32, String> {
    let value = value.trim();
    let parsed = if let Some(value) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u32::from_str_radix(value, 16)
    } else if let Some(value) = value
        .strip_prefix("0b")
        .or_else(|| value.strip_prefix("0B"))
    {
        u32::from_str_radix(value, 2)
    } else {
        value.parse::<u32>()
    };
    parsed.map_err(|_| {
        format!("Inspector render layer mask `{value}` must be an unsigned 32-bit value")
    })
}

#[cfg(test)]
#[path = "tests/render_layer_edit.rs"]
mod tests;
