use std::fmt;

#[derive(Debug)]
pub enum EditorError {
    Layout(String),
    Registry(String),
    Project(String),
    UiAsset(String),
}

impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Layout(error)
            | Self::Registry(error)
            | Self::Project(error)
            | Self::UiAsset(error) => f.write_str(error),
        }
    }
}

impl std::error::Error for EditorError {}
