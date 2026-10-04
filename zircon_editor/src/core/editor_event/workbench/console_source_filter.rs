use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 控制台来源选择；菜单事件与日志投影共享这些值，as_str 提供稳定的界面状态标识。
pub enum ConsoleSourceFilter {
    #[default]
    All,
    Editor,
    Runtime,
    Play,
    Plugin,
    Import,
    ScriptBuild,
}

impl ConsoleSourceFilter {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Editor => "editor",
            Self::Runtime => "runtime",
            Self::Play => "play",
            Self::Plugin => "plugin",
            Self::Import => "import",
            Self::ScriptBuild => "script_build",
        }
    }
}
