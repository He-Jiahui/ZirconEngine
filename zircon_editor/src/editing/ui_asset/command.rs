#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiAssetEditorCommand {
    EditSource { next_source: String },
}

impl UiAssetEditorCommand {
    pub fn edit_source(next_source: impl Into<String>) -> Self {
        Self::EditSource {
            next_source: next_source.into(),
        }
    }

    pub fn next_source(&self) -> &str {
        match self {
            Self::EditSource { next_source } => next_source,
        }
    }
}
