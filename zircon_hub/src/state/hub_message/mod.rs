//! 向状态层集中暴露消息载体和各业务身份，避免动作端依赖具体语言模板文件。
//! 模板属于业务族，编号解析与归档兼容分别由身份层和消息层拥有。

#[path = "build.rs"]
mod build;
#[path = "delivery.rs"]
mod delivery;
#[path = "engine.rs"]
mod engine;
#[path = "id.rs"]
mod id;
#[path = "learn.rs"]
mod learn;
#[path = "message.rs"]
mod message;
#[path = "process.rs"]
mod process;
#[path = "project.rs"]
mod project;
#[path = "settings.rs"]
mod settings;
#[path = "shell.rs"]
mod shell;

pub use id::{
    BuildMessageId, DeliveryMessageId, EngineMessageId, HubMessageId, LearnMessageId,
    ProcessMessageId, ProjectMessageId, SettingsMessageId, ShellMessageId,
};
pub use message::HubMessage;
