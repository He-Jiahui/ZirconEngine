use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetTypePresentation {
    display_name: String,
    badge: String,
    icon_name: String,
    color_token: String,
}

impl AssetTypePresentation {
    pub fn new(
        display_name: impl Into<String>,
        badge: impl Into<String>,
        icon_name: impl Into<String>,
        color_token: impl Into<String>,
    ) -> Self {
        Self {
            display_name: display_name.into(),
            badge: badge.into(),
            icon_name: icon_name.into(),
            color_token: color_token.into(),
        }
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn badge(&self) -> &str {
        &self.badge
    }

    pub fn icon_name(&self) -> &str {
        &self.icon_name
    }

    pub fn color_token(&self) -> &str {
        &self.color_token
    }

    pub(super) fn first_empty_field(&self) -> Option<&'static str> {
        if self.display_name.is_empty() {
            Some("presentation.display_name")
        } else if self.badge.is_empty() {
            Some("presentation.badge")
        } else if self.icon_name.is_empty() {
            Some("presentation.icon_name")
        } else if self.color_token.is_empty() {
            Some("presentation.color_token")
        } else {
            None
        }
    }
}
