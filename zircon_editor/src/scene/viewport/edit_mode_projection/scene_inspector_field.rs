use super::SceneInspectorFieldValue;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneInspectorField {
    pub(crate) component: String,
    pub(crate) label: String,
    pub(crate) property_path: Option<String>,
    pub(crate) value: SceneInspectorFieldValue,
    pub(crate) editable: bool,
}
