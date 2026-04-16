use crate::{
    UiBindingRef, UiTemplateDocument, UiTemplateError, UiTemplateNode, UiTemplateValidator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct UiTemplateInstance {
    pub root: UiTemplateNode,
}

impl UiTemplateInstance {
    pub fn from_document(document: &UiTemplateDocument) -> Result<Self, UiTemplateError> {
        UiTemplateValidator::validate_document(document)?;
        let mut roots = expand_node(document, &document.root, None)?;
        let root =
            roots
                .drain(..)
                .next()
                .ok_or_else(|| UiTemplateError::InvalidNodeDefinition {
                    detail: "template document expansion produced no root nodes".to_string(),
                })?;
        Ok(Self { root })
    }

    pub fn binding_refs(&self) -> Vec<&UiBindingRef> {
        let mut bindings = Vec::new();
        collect_binding_refs(&self.root, &mut bindings);
        bindings
    }
}

fn collect_binding_refs<'a>(node: &'a UiTemplateNode, bindings: &mut Vec<&'a UiBindingRef>) {
    bindings.extend(node.bindings.iter());
    for child in &node.children {
        collect_binding_refs(child, bindings);
    }
}

fn expand_node(
    document: &UiTemplateDocument,
    node: &UiTemplateNode,
    slot_content: Option<&std::collections::BTreeMap<String, Vec<UiTemplateNode>>>,
) -> Result<Vec<UiTemplateNode>, UiTemplateError> {
    if let Some(slot_name) = &node.slot {
        let mut expanded = Vec::new();
        if let Some(provided) = slot_content.and_then(|slots| slots.get(slot_name)) {
            for filled_node in provided {
                expanded.extend(expand_node(document, filled_node, None)?);
            }
        }
        return Ok(expanded);
    }

    if let Some(template_id) = &node.template {
        let template = document.components.get(template_id).ok_or_else(|| {
            UiTemplateError::UnknownTemplate {
                template_id: template_id.clone(),
            }
        })?;
        return expand_node(document, &template.root, Some(&node.slots));
    }

    let mut expanded = node.clone();
    expanded.template = None;
    expanded.slot = None;
    expanded.slots.clear();
    expanded.children.clear();
    for child in &node.children {
        expanded
            .children
            .extend(expand_node(document, child, slot_content)?);
    }
    Ok(vec![expanded])
}
