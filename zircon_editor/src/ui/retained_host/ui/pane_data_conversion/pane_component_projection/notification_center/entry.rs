#[derive(Clone, Debug)]
pub(super) struct NotificationProjectionEntry {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) message: String,
    pub(super) tone: String,
    pub(super) unread: bool,
    pub(super) disabled: bool,
}

impl NotificationProjectionEntry {
    pub(super) fn new(id: String) -> Self {
        Self {
            title: id.clone(),
            id,
            message: String::new(),
            tone: "info".to_string(),
            unread: false,
            disabled: false,
        }
    }

    pub(super) fn matches_id(&self, id: &str) -> bool {
        !id.is_empty() && (self.id == id || self.title == id)
    }
}
