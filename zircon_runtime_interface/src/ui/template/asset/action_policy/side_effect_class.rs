use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiActionSideEffectClass {
    LocalUi,
    EditorMutation,
    AssetIo,
    SceneMutation,
    ExternalProcess,
    Network,
}

impl UiActionSideEffectClass {
    pub fn infer(route: Option<&str>, action: Option<&str>) -> Self {
        let text = format!(
            "{} {}",
            route.unwrap_or_default().to_ascii_lowercase(),
            action.unwrap_or_default().to_ascii_lowercase()
        );
        if text.contains("network") || text.contains("http") || text.contains("socket") {
            Self::Network
        } else if text.contains("process") || text.contains("command") || text.contains("shell") {
            Self::ExternalProcess
        } else if text.contains("scene") || text.contains("entity") || text.contains("world") {
            Self::SceneMutation
        } else if text.contains("asset")
            || text.contains("file")
            || text.contains("save")
            || text.contains("load")
            || text.contains("import")
        {
            Self::AssetIo
        } else if text.contains("editor") || text.contains("inspector") || text.contains("undo") {
            Self::EditorMutation
        } else {
            Self::LocalUi
        }
    }
}
