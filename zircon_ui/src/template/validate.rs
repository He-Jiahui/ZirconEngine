use crate::{UiComponentTemplate, UiTemplateDocument, UiTemplateError, UiTemplateNode};

#[derive(Default)]
pub struct UiTemplateValidator;

impl UiTemplateValidator {
    pub fn validate_document(document: &UiTemplateDocument) -> Result<(), UiTemplateError> {
        validate_node(document, &document.root, None)
    }
}

fn validate_node(
    document: &UiTemplateDocument,
    node: &UiTemplateNode,
    containing_template: Option<(&str, &UiComponentTemplate)>,
) -> Result<(), UiTemplateError> {
    if node.node_kind_count() != 1 {
        return Err(UiTemplateError::InvalidNodeDefinition {
            detail: "every template node must define exactly one of component/template/slot"
                .to_string(),
        });
    }

    if let Some(slot_name) = &node.slot {
        if !node.bindings.is_empty()
            || !node.children.is_empty()
            || !node.slots.is_empty()
            || node.control_id.is_some()
        {
            return Err(UiTemplateError::InvalidNodeDefinition {
                detail: format!("slot placeholder {slot_name} cannot declare bindings, children, slot fills, or control ids"),
            });
        }
        if let Some((template_id, template)) = containing_template {
            if !template.slots.contains_key(slot_name) {
                return Err(UiTemplateError::UndeclaredSlotPlaceholder {
                    template_id: template_id.to_string(),
                    slot_name: slot_name.clone(),
                });
            }
        }
        return Ok(());
    }

    if let Some(component) = &node.component {
        if !node.slots.is_empty() {
            return Err(UiTemplateError::InvalidNodeDefinition {
                detail: format!("component node {component} cannot declare slot fills; use children for direct descendants or template calls for slot assignment"),
            });
        }
        for child in &node.children {
            validate_node(document, child, containing_template)?;
        }
        return Ok(());
    }

    let template_id =
        node.template
            .as_deref()
            .ok_or_else(|| UiTemplateError::InvalidNodeDefinition {
                detail: "template node missing template id".to_string(),
            })?;
    let template =
        document
            .components
            .get(template_id)
            .ok_or_else(|| UiTemplateError::UnknownTemplate {
                template_id: template_id.to_string(),
            })?;

    if !node.children.is_empty() {
        return Err(UiTemplateError::InvalidNodeDefinition {
            detail: format!(
                "template node {template_id} cannot declare direct children; use named slots"
            ),
        });
    }

    for (slot_name, slot_template) in &template.slots {
        let provided = node.slots.get(slot_name).map(Vec::len).unwrap_or_default();
        if slot_template.required && provided == 0 {
            return Err(UiTemplateError::MissingRequiredSlot {
                template_id: template_id.to_string(),
                slot_name: slot_name.clone(),
            });
        }
        if !slot_template.multiple && provided > 1 {
            return Err(UiTemplateError::SlotDoesNotAcceptMultiple {
                template_id: template_id.to_string(),
                slot_name: slot_name.clone(),
            });
        }
    }

    for (slot_name, filled_nodes) in &node.slots {
        if !template.slots.contains_key(slot_name) {
            return Err(UiTemplateError::UnknownSlot {
                template_id: template_id.to_string(),
                slot_name: slot_name.clone(),
            });
        }
        for filled_node in filled_nodes {
            validate_node(document, filled_node, None)?;
        }
    }

    validate_node(document, &template.root, Some((template_id, template)))
}
