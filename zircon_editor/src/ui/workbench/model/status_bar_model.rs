#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusBarModel {
    pub primary_text: String,
    pub secondary_text: Option<String>,
    pub viewport_label: String,
}
