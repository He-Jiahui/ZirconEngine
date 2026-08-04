use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::core::play::PlayInstanceId;

use super::EditorLogError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogChannel {
    Editor,
    Runtime,
    Play,
    Plugin,
    Import,
    ScriptBuild,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogSource {
    kind: LogChannel,
    detail: LogSourceDetail,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum LogSourceDetail {
    None,
    Play(PlayInstanceId),
    Plugin(Arc<str>),
}

impl LogSource {
    pub const fn editor() -> Self {
        Self {
            kind: LogChannel::Editor,
            detail: LogSourceDetail::None,
        }
    }

    pub const fn runtime() -> Self {
        Self {
            kind: LogChannel::Runtime,
            detail: LogSourceDetail::None,
        }
    }

    pub const fn play(instance: PlayInstanceId) -> Self {
        Self {
            kind: LogChannel::Play,
            detail: LogSourceDetail::Play(instance),
        }
    }

    pub fn plugin(id: impl Into<String>) -> Result<Self, EditorLogError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(EditorLogError::EmptyPluginSource);
        }
        Ok(Self {
            kind: LogChannel::Plugin,
            detail: LogSourceDetail::Plugin(Arc::from(id)),
        })
    }

    pub const fn import() -> Self {
        Self {
            kind: LogChannel::Import,
            detail: LogSourceDetail::None,
        }
    }

    pub const fn script_build() -> Self {
        Self {
            kind: LogChannel::ScriptBuild,
            detail: LogSourceDetail::None,
        }
    }

    pub const fn channel(&self) -> LogChannel {
        self.kind
    }

    pub(super) fn estimated_bytes(&self) -> usize {
        match &self.detail {
            LogSourceDetail::None => self.kind_name().len(),
            LogSourceDetail::Play(_) => "play".len() + std::mem::size_of::<u64>(),
            LogSourceDetail::Plugin(id) => "plugin".len() + id.len(),
        }
    }

    fn kind_name(&self) -> &'static str {
        match self.kind {
            LogChannel::Editor => "editor",
            LogChannel::Runtime => "runtime",
            LogChannel::Play => "play",
            LogChannel::Plugin => "plugin",
            LogChannel::Import => "import",
            LogChannel::ScriptBuild => "script_build",
        }
    }
}

impl Display for LogSource {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            LogSourceDetail::None => formatter.write_str(self.kind_name()),
            LogSourceDetail::Play(instance) => write!(formatter, "play:{}", instance.raw()),
            LogSourceDetail::Plugin(id) => write!(formatter, "plugin:{id}"),
        }
    }
}
