//! 汇集各业务消息命名空间，连接持久化的稳定字符串编号与运行时的类型化身份。
//! 解析按命名空间查表，未知编号交给消息层的归档兼容分支处理。

use crate::settings::HubLanguage;

pub use super::build::BuildMessageId;
pub use super::delivery::DeliveryMessageId;
pub use super::engine::EngineMessageId;
pub use super::learn::LearnMessageId;
pub use super::process::ProcessMessageId;
pub use super::project::ProjectMessageId;
pub use super::settings::SettingsMessageId;
pub use super::shell::ShellMessageId;

/// 供运行时、配置和语言投影共同使用的消息身份；字符串编号是归档格式的一部分。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HubMessageId {
    Shell(ShellMessageId),
    Project(ProjectMessageId),
    Engine(EngineMessageId),
    Build(BuildMessageId),
    Delivery(DeliveryMessageId),
    Process(ProcessMessageId),
    Settings(SettingsMessageId),
    Learn(LearnMessageId),
}

impl HubMessageId {
    /// 返回可持久化的语义编号；显示文本随语言变化时编号保持稳定。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shell(id) => id.as_str(),
            Self::Project(id) => id.as_str(),
            Self::Engine(id) => id.as_str(),
            Self::Build(id) => id.as_str(),
            Self::Delivery(id) => id.as_str(),
            Self::Process(id) => id.as_str(),
            Self::Settings(id) => id.as_str(),
            Self::Learn(id) => id.as_str(),
        }
    }

    /// 从配置中的精确编号恢复类型；未知编号返回空值，由消息反序列化保留原始内容。
    pub fn from_str_id(id: &str) -> Option<Self> {
        let (namespace, _) = id.split_once('.')?;
        match namespace {
            "shell" => ShellMessageId::ALL
                .iter()
                .copied()
                .find(|candidate| candidate.as_str() == id)
                .map(Self::Shell),
            "project" => ProjectMessageId::ALL
                .iter()
                .copied()
                .find(|candidate| candidate.as_str() == id)
                .map(Self::Project),
            "engine" => EngineMessageId::ALL
                .iter()
                .copied()
                .find(|candidate| candidate.as_str() == id)
                .map(Self::Engine),
            "build" => BuildMessageId::ALL
                .iter()
                .copied()
                .find(|candidate| candidate.as_str() == id)
                .map(Self::Build),
            "delivery" => DeliveryMessageId::ALL
                .iter()
                .copied()
                .find(|candidate| candidate.as_str() == id)
                .map(Self::Delivery),
            "process" => ProcessMessageId::ALL
                .iter()
                .copied()
                .find(|candidate| candidate.as_str() == id)
                .map(Self::Process),
            "settings" => SettingsMessageId::ALL
                .iter()
                .copied()
                .find(|candidate| candidate.as_str() == id)
                .map(Self::Settings),
            "learn" => LearnMessageId::ALL
                .iter()
                .copied()
                .find(|candidate| candidate.as_str() == id)
                .map(Self::Learn),
            _ => None,
        }
    }

    /// 供模板全集审查使用的参数数量契约；消息构造和加载当前不会据此拒绝参数。
    pub fn param_count(self) -> usize {
        match self {
            Self::Shell(id) => id.param_count(),
            Self::Project(id) => id.param_count(),
            Self::Engine(id) => id.param_count(),
            Self::Build(id) => id.param_count(),
            Self::Delivery(id) => id.param_count(),
            Self::Process(id) => id.param_count(),
            Self::Settings(id) => id.param_count(),
            Self::Learn(id) => id.param_count(),
        }
    }

    /// 仅由显示路径按当前语言选模板；参数的替换和原始文本兼容属于消息层。
    pub fn template(self, language: HubLanguage) -> &'static str {
        match self {
            Self::Shell(id) => id.template(language),
            Self::Project(id) => id.template(language),
            Self::Engine(id) => id.template(language),
            Self::Build(id) => id.template(language),
            Self::Delivery(id) => id.template(language),
            Self::Process(id) => id.template(language),
            Self::Settings(id) => id.template(language),
            Self::Learn(id) => id.template(language),
        }
    }

    /// 为枚举全集测试提供所有消息；单次编号解析应使用命名空间查表入口。
    pub fn all() -> Vec<Self> {
        let mut ids = Vec::new();
        ids.extend(ShellMessageId::ALL.iter().copied().map(Self::Shell));
        ids.extend(ProjectMessageId::ALL.iter().copied().map(Self::Project));
        ids.extend(EngineMessageId::ALL.iter().copied().map(Self::Engine));
        ids.extend(BuildMessageId::ALL.iter().copied().map(Self::Build));
        ids.extend(DeliveryMessageId::ALL.iter().copied().map(Self::Delivery));
        ids.extend(ProcessMessageId::ALL.iter().copied().map(Self::Process));
        ids.extend(SettingsMessageId::ALL.iter().copied().map(Self::Settings));
        ids.extend(LearnMessageId::ALL.iter().copied().map(Self::Learn));
        ids
    }
}

#[cfg(test)]
#[path = "tests/id.rs"]
mod tests;
