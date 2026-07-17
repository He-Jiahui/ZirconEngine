#[derive(Clone, Debug)]
pub(super) struct CommandProjectionEntry {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) description: String,
    pub(super) disabled: bool,
    pub(super) filter_matched: bool,
}

impl CommandProjectionEntry {
    pub(super) fn new(id: String) -> Self {
        Self {
            label: id.clone(),
            id,
            description: String::new(),
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
        contains_ascii_case_insensitive(&self.id, query)
            || contains_ascii_case_insensitive(&self.label, query)
    }
}

fn contains_ascii_case_insensitive(value: &str, expected: &str) -> bool {
    expected.is_empty()
        || value
            .as_bytes()
            .windows(expected.len())
            .any(|window| window.eq_ignore_ascii_case(expected.as_bytes()))
}
