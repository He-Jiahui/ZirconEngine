#[derive(Clone, Debug)]
pub(super) struct CommandProjectionEntry {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) disabled: bool,
}

impl CommandProjectionEntry {
    pub(super) fn new(id: String) -> Self {
        Self {
            label: id.clone(),
            id,
            disabled: false,
        }
    }

    pub(super) fn matches_query(&self, query: Option<&str>) -> bool {
        let Some(query) = query else {
            return false;
        };
        self.id.to_ascii_lowercase().contains(query)
            || self.label.to_ascii_lowercase().contains(query)
    }
}
