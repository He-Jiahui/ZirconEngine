use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiAssetSourceBuffer {
    text: Arc<String>,
    saved_text: Arc<String>,
    revision: u64,
}

/// Retains the actual buffer allocation so closing/reopening or undo cannot reuse a ticket.
#[derive(Clone, Debug)]
pub(crate) struct UiAssetSourceSnapshot {
    text: Arc<String>,
    revision: u64,
}

impl UiAssetSourceSnapshot {
    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        self.revision == other.revision && Arc::ptr_eq(&self.text, &other.text)
    }
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

    pub(crate) fn snapshot(&self) -> UiAssetSourceSnapshot {
        UiAssetSourceSnapshot {
            text: Arc::clone(&self.text),
            revision: self.revision,
        }
    }

    pub(crate) fn is_current_snapshot(&self, snapshot: &UiAssetSourceSnapshot) -> bool {
        self.revision == snapshot.revision && Arc::ptr_eq(&self.text, &snapshot.text)
    }

    pub fn mark_saved(&mut self) {
        self.saved_text = Arc::clone(&self.text);
    }

    pub fn is_dirty(&self) -> bool {
        self.text != self.saved_text
    }
}

#[cfg(test)]
#[path = "tests/source_buffer.rs"]
mod tests;

#[cfg(test)]
#[path = "source_buffer/tests/shared_text.rs"]
mod shared_text_tests;
