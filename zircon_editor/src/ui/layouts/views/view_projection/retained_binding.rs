use zircon_runtime_interface::ui::{
    component::UiValue, event_ui::UiNodeId, tree::UiTemplateNodeMetadata,
};

use super::super::ViewTemplateNodeData;
use super::{icon_button_hides_label, string_attribute, value_to_display_text};

pub(super) struct ViewTemplateTextOverrideSemantics {
    hides_label: bool,
    input_kind: Option<ViewTemplateInputKind>,
    input_placeholder: String,
    authored_text: String,
    authored_value_text: String,
}

#[derive(Clone, Copy)]
enum ViewTemplateInputKind {
    Text,
    Number,
}

impl ViewTemplateTextOverrideSemantics {
    pub(super) fn from_metadata(
        metadata: &UiTemplateNodeMetadata,
        component_role: &str,
        authored_requested_text: &str,
    ) -> Self {
        let input_kind = match component_role {
            "input-field" => Some(ViewTemplateInputKind::Text),
            "number-field" => Some(ViewTemplateInputKind::Number),
            _ => None,
        };
        let authored_value_text = metadata
            .attributes
            .get("value_text")
            .or_else(|| metadata.attributes.get("value"))
            .map(value_to_display_text)
            .unwrap_or_default();
        let input_placeholder = input_kind
            .is_some()
            .then(|| string_attribute(metadata, "placeholder").unwrap_or_default())
            .unwrap_or_default();
        let mut semantics = Self {
            hides_label: icon_button_hides_label(metadata),
            input_kind,
            input_placeholder,
            authored_text: String::new(),
            authored_value_text,
        };
        semantics.authored_text = semantics.authored_display_text(authored_requested_text);
        semantics
    }

    fn authored_display_text(&self, authored_requested_text: &str) -> String {
        if self.hides_label {
            return String::new();
        }
        if self.input_kind.is_some() {
            return if self.authored_value_text.is_empty() {
                self.input_placeholder.clone()
            } else {
                self.authored_value_text.clone()
            };
        }
        authored_requested_text.to_string()
    }

    pub(super) fn apply(
        &self,
        node: &mut ViewTemplateNodeData,
        requested_text: Option<&str>,
        current_value_number: Option<f32>,
    ) {
        let (display_text, value_text) = self.projected_text(requested_text);
        node.text = display_text.into();
        node.value_text = value_text.into();
        if matches!(self.input_kind, Some(ViewTemplateInputKind::Number)) {
            node.value_number = current_value_number.unwrap_or_default();
        }
    }

    pub(super) fn projected_text(&self, requested_text: Option<&str>) -> (String, String) {
        let Some(requested_text) = requested_text else {
            return (self.authored_text.clone(), self.authored_value_text.clone());
        };
        if self.input_kind.is_some() {
            let display_text = if requested_text.is_empty() {
                self.input_placeholder.clone()
            } else {
                requested_text.to_string()
            };
            return (display_text, requested_text.to_string());
        }
        (
            if self.hides_label {
                String::new()
            } else {
                requested_text.to_string()
            },
            String::new(),
        )
    }
}

pub(super) struct ViewTemplateTextBinding {
    pub(super) node_id: UiNodeId,
    pub(super) property: String,
    pub(super) authored_value: UiValue,
    numeric_property: Option<(String, UiValue)>,
}

impl ViewTemplateTextBinding {
    pub(super) fn requested_mutations(
        &self,
        requested_text: Option<&str>,
    ) -> Vec<(String, UiValue)> {
        let text_value = requested_text
            .map(|text| match &self.authored_value {
                UiValue::Int(_) => text
                    .parse::<i64>()
                    .map(UiValue::Int)
                    .unwrap_or_else(|_| UiValue::String(text.to_string())),
                UiValue::Float(_) => text
                    .parse::<f64>()
                    .map(UiValue::Float)
                    .unwrap_or_else(|_| UiValue::String(text.to_string())),
                _ => UiValue::String(text.to_string()),
            })
            .unwrap_or_else(|| self.authored_value.clone());
        let mut mutations = vec![(self.property.clone(), text_value)];
        let Some((numeric_property, authored_numeric_value)) = self.numeric_property.as_ref()
        else {
            return mutations;
        };
        let numeric_value = match requested_text {
            None => Some(authored_numeric_value.clone()),
            Some(text) => match authored_numeric_value {
                UiValue::Int(_) => text.parse::<i64>().ok().map(UiValue::Int),
                UiValue::Float(_) => text.parse::<f64>().ok().map(UiValue::Float),
                _ => None,
            },
        };
        if let Some(numeric_value) = numeric_value {
            mutations.push((numeric_property.clone(), numeric_value));
        }
        mutations
    }
}

pub(super) fn text_binding_for_metadata(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    component_role: &str,
) -> ViewTemplateTextBinding {
    let candidate_properties: &[&str] = if component_role == "number-field" {
        &["value_text", "value"]
    } else if component_role == "input-field" {
        &["value", "value_text"]
    } else {
        &["label", "text", "placeholder"]
    };
    let (property, authored_value) = candidate_properties
        .iter()
        .find_map(|property| {
            metadata
                .attributes
                .get(*property)
                .map(|value| ((*property).to_string(), UiValue::from_toml(value)))
        })
        .unwrap_or_else(|| {
            let property = if matches!(component_role, "input-field" | "number-field") {
                "value"
            } else {
                "text"
            };
            (property.to_string(), UiValue::String(String::new()))
        });
    let numeric_property = (component_role == "number-field")
        .then(|| {
            metadata
                .attributes
                .get("value")
                .map(|value| ("value".to_string(), UiValue::from_toml(value)))
        })
        .flatten()
        .filter(|(numeric_property, _)| numeric_property != &property);
    ViewTemplateTextBinding {
        node_id,
        property,
        authored_value,
        numeric_property,
    }
}
