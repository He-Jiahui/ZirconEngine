#[derive(Default)]
pub(in super::super) struct PendingOptionManifest {
    pub(in super::super) key: Option<String>,
    pub(in super::super) display_name: Option<String>,
    pub(in super::super) value_type: Option<String>,
    pub(in super::super) default_value: Option<String>,
    pub(in super::super) enum_values: Vec<String>,
    pub(in super::super) required_capability: Option<String>,
}
