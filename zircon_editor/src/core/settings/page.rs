//! Descriptor for one plugin-contributed settings page.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsPageDescriptor {
    id: String,
    display_name: String,
    category_path: String,
}

impl SettingsPageDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        category_path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            category_path: category_path.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn category_path(&self) -> &str {
        &self.category_path
    }

    pub(crate) fn is_valid_category_path(&self) -> bool {
        !self.category_path.is_empty() && !self.category_path.split('/').any(str::is_empty)
    }
}
