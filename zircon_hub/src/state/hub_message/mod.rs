//! 向状态层集中暴露消息载体和各业务身份，避免动作端依赖具体语言模板文件。
//! 模板属于业务族，编号解析与归档兼容分别由身份层和消息层拥有。

mod build;
mod delivery;
mod engine;
mod id;
mod learn;
mod message;
mod process;
mod project;
mod settings;
mod shell;

pub use id::{
    BuildMessageId, DeliveryMessageId, EngineMessageId, HubMessageId, LearnMessageId,
    ProcessMessageId, ProjectMessageId, SettingsMessageId, ShellMessageId,
};
pub use message::HubMessage;
