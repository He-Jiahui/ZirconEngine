use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HubPage {
    #[default]
    Projects,
    Editor,
    Assets,
    Builds,
    Plugins,
    Cloud,
    Team,
    Learn,
    Settings,
}

impl HubPage {
    pub fn id(self) -> &'static str {
        match self {
            Self::Projects => "projects",
            Self::Editor => "editor",
            Self::Assets => "assets",
            Self::Builds => "builds",
            Self::Plugins => "plugins",
            Self::Cloud => "cloud",
            Self::Team => "team",
            Self::Learn => "learn",
            Self::Settings => "settings",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "projects" => Some(Self::Projects),
            "editor" => Some(Self::Editor),
            "assets" => Some(Self::Assets),
            "builds" => Some(Self::Builds),
            "plugins" => Some(Self::Plugins),
            "cloud" => Some(Self::Cloud),
            "team" => Some(Self::Team),
            "learn" => Some(Self::Learn),
            "settings" => Some(Self::Settings),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HubPage;

    #[test]
    fn hub_page_parses_known_navigation_ids() {
        assert_eq!(HubPage::from_id("projects"), Some(HubPage::Projects));
        assert_eq!(HubPage::from_id("EDITOR"), Some(HubPage::Editor));
        assert_eq!(HubPage::from_id("settings"), Some(HubPage::Settings));
        assert_eq!(HubPage::from_id("missing"), None);
    }
}
