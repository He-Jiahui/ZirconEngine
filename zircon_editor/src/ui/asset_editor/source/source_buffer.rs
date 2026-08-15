#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiAssetSourceBuffer {
    text: String,
    saved_text: String,
    revision: u64,
}

impl UiAssetSourceBuffer {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            saved_text: text.clone(),
            text,
            revision: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn replace(&mut self, text: impl Into<String>) {
        let text = text.into();
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
        self.saved_text = self.text.clone();
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
