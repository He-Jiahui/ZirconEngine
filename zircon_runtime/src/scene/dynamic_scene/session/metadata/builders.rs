use super::model::RuntimeSessionMetadata;
use super::tags::normalize_metadata_tags;

impl RuntimeSessionMetadata {
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn with_updated_at_unix_millis(mut self, updated_at_unix_millis: u64) -> Self {
        self.updated_at_unix_millis = Some(updated_at_unix_millis);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self.normalize();
        self
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        normalize_metadata_tags(&mut self.tags);
    }
}
