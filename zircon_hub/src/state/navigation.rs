//! 定义配置、动作请求和界面导航共同使用的页面身份；语言投影只改变页面名称。

use serde::{Deserialize, Serialize};

/// 持久化和跨端请求的页面身份；新增页面需同步动作解析、语言投影和 Web 页面分派。
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
    /// 输出导航协议中的稳定编号，不能使用本地化标题代替。
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

    /// 供动作入口识别页面；接受大小写/边缘空白差异，未知目标留给调用者报错。
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
#[path = "tests/navigation.rs"]
mod tests;
