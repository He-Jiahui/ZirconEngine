use serde::{Deserialize, Serialize};

use crate::core::editor_operation::EditorOperationPath;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThumbnailProviderDescriptor {
    SourceImage,
    Icon(String),
    Placeholder {
        icon_name: String,
        palette: ThumbnailPlaceholderPalette,
    },
    Operation(EditorOperationPath),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThumbnailPlaceholderPalette {
    pub primary: [u8; 4],
    pub secondary: [u8; 4],
    pub accent: [u8; 4],
    pub banner: [u8; 4],
}

impl ThumbnailPlaceholderPalette {
    pub const fn neutral() -> Self {
        Self {
            primary: [42, 45, 55, 255],
            secondary: [24, 26, 34, 255],
            accent: [118, 128, 158, 255],
            banner: [16, 18, 24, 220],
        }
    }
}
