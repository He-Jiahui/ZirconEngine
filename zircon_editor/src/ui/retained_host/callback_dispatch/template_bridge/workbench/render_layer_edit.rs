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
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::layout::UiSize;

    #[test]
    fn render_layer_mask_parser_accepts_decimal_hex_and_binary() {
        assert_eq!(parse_render_layer_mask("17").unwrap(), 17);
        assert_eq!(parse_render_layer_mask("0x11").unwrap(), 17);
        assert_eq!(parse_render_layer_mask("0b10001").unwrap(), 17);
        assert!(parse_render_layer_mask("-1").is_err());
        assert!(parse_render_layer_mask("4294967296").is_err());
    }

    #[test]
    fn render_layer_commit_uses_the_reflected_unsigned_field() {
        let bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
        let binding = bridge
            .render_layer_mask_commit_binding(
                RENDER_LAYER_MASK_CONTROL,
                RENDER_LAYER_MASK_COMMIT,
                "0x20",
            )
            .unwrap()
            .expect("render layer commit should resolve");
        let EditorUiBindingPayload::InspectorFieldBatch { changes, .. } = binding.payload() else {
            panic!("render layer commit must dispatch an inspector field batch");
        };
        assert_eq!(changes[0].field_id, RENDER_LAYER_MASK_FIELD);
        assert_eq!(changes[0].value, UiBindingValue::Unsigned(32));
    }

    #[test]
    fn render_layer_edit_updates_only_the_retained_draft_value() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
        assert_eq!(
            bridge
                .edit_inspector_render_layer_mask(
                    RENDER_LAYER_MASK_CONTROL,
                    RENDER_LAYER_MASK_EDIT,
                    " 0x20 ",
                )
                .unwrap(),
            Some(true)
        );
        assert_eq!(
            bridge
                .control_string(RENDER_LAYER_MASK_CONTROL, "value")
                .as_deref(),
            Some("0x20")
        );
    }
}
