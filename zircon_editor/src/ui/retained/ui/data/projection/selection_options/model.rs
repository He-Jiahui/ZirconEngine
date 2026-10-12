use crate::ui::retained_host as host_contract;

pub(in super::super) struct ProjectedSelectionOptions {
    pub(in super::super) selection_state: String,
    pub(in super::super) search_query: String,
    pub(in super::super) selected: bool,
    pub(in super::super) tree_depth: i32,
    pub(in super::super) tree_indent_px: f32,
    pub(in super::super) options_text: String,
    pub(in super::super) options: Vec<String>,
    pub(in super::super) structured_options: Vec<host_contract::TemplatePaneOptionData>,
}
