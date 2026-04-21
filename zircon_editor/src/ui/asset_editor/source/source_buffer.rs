#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiAssetSourceBuffer {
    text: String,
    saved_text: String,
}

impl UiAssetSourceBuffer {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            saved_text: text.clone(),
            text,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn replace(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn mark_saved(&mut self) {
        self.saved_text = self.text.clone();
    }

    pub fn is_dirty(&self) -> bool {
        self.text != self.saved_text
    }
}
