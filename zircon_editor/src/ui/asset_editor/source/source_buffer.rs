use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiAssetSourceBuffer {
    text: Arc<String>,
    saved_text: Arc<String>,
    revision: u64,
}

impl UiAssetSourceBuffer {
    pub fn new(text: impl Into<String>) -> Self {
        let text = Arc::new(text.into());
        Self {
            saved_text: Arc::clone(&text),
            text,
            revision: 0,
        }
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub fn replace(&mut self, text: impl Into<String>) {
        let text = Arc::new(text.into());
        if self.text == text {
            return;
        }
        self.text = text;
        self.revision = self.revision.wrapping_add(1);
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub fn mark_saved(&mut self) {
        self.saved_text = Arc::clone(&self.text);
    }

    pub fn is_dirty(&self) -> bool {
        self.text != self.saved_text
    }
}

#[cfg(test)]
mod tests {
    use super::UiAssetSourceBuffer;

    #[test]
    fn revision_changes_only_when_source_text_changes() {
        let mut buffer = UiAssetSourceBuffer::new("[nodes.root]");

        buffer.replace("[nodes.root]");
        assert_eq!(buffer.revision(), 0);

        buffer.replace("[nodes.root]\nkind = \"native\"");
        assert_eq!(buffer.revision(), 1);
    }
}

#[cfg(test)]
#[path = "source_buffer/shared_text_tests.rs"]
mod shared_text_tests;
