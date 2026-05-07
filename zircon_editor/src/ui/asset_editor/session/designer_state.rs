use super::{
    inspector_fields::{
        set_selected_node_slot_height_preferred, set_selected_node_slot_width_preferred,
    },
    ui_asset_editor_session::{UiAssetEditorSession, UiAssetEditorSessionError},
};
use crate::ui::asset_editor::{UiDesignerPreviewInteractDispatch, UiDesignerToolMode};
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    template::{UiActionSideEffectClass, UiBindingRef, UiBindingTargetKind},
};

impl UiAssetEditorSession {
    pub fn designer_tool_mode(&self) -> UiDesignerToolMode {
        self.designer_tool_mode
    }

    pub fn set_designer_tool_mode(&mut self, mode: UiDesignerToolMode) -> bool {
        if self.designer_tool_mode == mode {
            return false;
        }
        self.designer_tool_mode = mode;
        true
    }

    pub fn can_resize_selected_slot(&self) -> bool {
        self.selection
            .primary_node_id
            .as_deref()
            .and_then(|node_id| self.last_valid_document.child_mount(node_id))
            .is_some()
    }

    pub fn can_preview_interact(&self) -> bool {
        self.preview_host.is_some() && self.diagnostics.is_empty()
    }

    pub fn last_preview_interact_dispatch(&self) -> Option<&UiDesignerPreviewInteractDispatch> {
        self.last_preview_interact_dispatch.as_ref()
    }

    pub fn dispatch_preview_interact_at_preview_index(
        &mut self,
        index: usize,
        event: UiEventKind,
    ) -> Result<Option<UiDesignerPreviewInteractDispatch>, UiAssetEditorSessionError> {
        if !self.can_preview_interact() {
            return Err(UiAssetEditorSessionError::InvalidPreviewIndex { index });
        }
        self.select_preview_index(index)?;
        self.designer_tool_mode = UiDesignerToolMode::PreviewInteract;
        let Some(node_id) = self.selection.primary_node_id.clone() else {
            return Err(UiAssetEditorSessionError::InvalidPreviewIndex { index });
        };
        let Some(node) = self.last_valid_document.node(&node_id) else {
            return Err(UiAssetEditorSessionError::InvalidPreviewIndex { index });
        };
        let Some(binding) = node.bindings.iter().find(|binding| binding.event == event) else {
            self.last_preview_interact_dispatch = None;
            return Ok(None);
        };
        let dispatch =
            build_preview_interact_dispatch(&node_id, node.control_id.as_deref(), binding);
        self.last_preview_interact_dispatch = Some(dispatch.clone());
        Ok(Some(dispatch))
    }

    pub fn resize_selected_slot_preferred_size(
        &mut self,
        width: f32,
        height: f32,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        if !self.can_resize_selected_slot() {
            return Ok(false);
        }
        let width_literal = designer_dimension_literal(width, "slot.layout.width.preferred")?;
        let height_literal = designer_dimension_literal(height, "slot.layout.height.preferred")?;
        let mut document = self.last_valid_document.clone();
        let width_changed =
            set_selected_node_slot_width_preferred(&mut document, &self.selection, &width_literal)
                .map_err(
                    |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                        field,
                        value: width_literal.clone(),
                    },
                )?;
        let height_changed = set_selected_node_slot_height_preferred(
            &mut document,
            &self.selection,
            &height_literal,
        )
        .map_err(
            |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                field,
                value: height_literal.clone(),
            },
        )?;
        if !width_changed && !height_changed {
            return Ok(false);
        }
        self.designer_tool_mode = UiDesignerToolMode::ResizeSlot;
        self.apply_document_edit_with_label(document, "Resize Slot")?;
        Ok(true)
    }
}

fn designer_dimension_literal(
    value: f32,
    field: &'static str,
) -> Result<String, UiAssetEditorSessionError> {
    if !value.is_finite() {
        return Err(UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
            field,
            value: value.to_string(),
        });
    }
    let value = value.max(0.0);
    let mut literal = format!("{value:.3}");
    while literal.contains('.') && literal.ends_with('0') {
        literal.pop();
    }
    if literal.ends_with('.') {
        literal.pop();
    }
    Ok(literal)
}

fn build_preview_interact_dispatch(
    node_id: &str,
    control_id: Option<&str>,
    binding: &UiBindingRef,
) -> UiDesignerPreviewInteractDispatch {
    let route = binding
        .action
        .as_ref()
        .and_then(|action| action.route.clone())
        .or_else(|| binding.route.clone())
        .unwrap_or_default();
    let action = binding
        .action
        .as_ref()
        .and_then(|action| action.action.clone())
        .unwrap_or_default();
    let side_effect_class =
        UiActionSideEffectClass::infer(Some(route.as_str()), Some(action.as_str()));
    let payload_items = binding
        .action
        .as_ref()
        .map(|action| {
            action
                .payload
                .iter()
                .map(|(key, value)| format!("{key} = {value}"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let target_items = binding
        .targets
        .iter()
        .map(|assignment| {
            let target = match assignment.target.kind {
                UiBindingTargetKind::Prop => "prop",
                UiBindingTargetKind::Class => "class",
                UiBindingTargetKind::Visibility => "visibility",
                UiBindingTargetKind::Enabled => "enabled",
                UiBindingTargetKind::ActionPayload => "action_payload",
            };
            match assignment.target.name.as_deref() {
                Some(name) => format!("{target}.{name} = {}", assignment.expression),
                None => format!("{target} = {}", assignment.expression),
            }
        })
        .collect();
    UiDesignerPreviewInteractDispatch {
        node_id: node_id.to_string(),
        control_id: control_id.unwrap_or_default().to_string(),
        event: binding.event,
        binding_id: binding.id.clone(),
        route,
        action,
        side_effect_class,
        payload_items,
        target_items,
    }
}
