#[derive(Clone, Debug)]
pub(super) struct CommandProjectionEntry {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) disabled: bool,
    pub(super) filter_matched: bool,
}

impl CommandProjectionEntry {
    pub(super) fn new(id: String) -> Self {
        Self {
            label: id.clone(),
            id,
            disabled: false,
            filter_matched: false,
        }
    }

    pub(super) fn with_filter_matched(mut self) -> Self {
        self.filter_matched = true;
        self
    }

    pub(super) fn matches_query(&self, query: Option<&str>) -> bool {
        let Some(query) = query.map(str::trim).filter(|query| !query.is_empty()) else {
            return false;
        };
        let query = query.to_ascii_lowercase();
        self.id.to_ascii_lowercase().contains(query.as_str())
            || self.label.to_ascii_lowercase().contains(query.as_str())
    }
}
