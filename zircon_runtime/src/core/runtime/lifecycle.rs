//! Service and module lifecycle vocabulary.

use serde::{Deserialize, Serialize};

use super::contexts::ModuleContext;
use crate::core::framework::error::CoreResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupMode {
    Immediate,
    Lazy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InitLevel {
    Kernel,
    Servers,
    Scene,
    Editor,
    Post,
}

impl InitLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Kernel => "Kernel",
            Self::Servers => "Servers",
            Self::Scene => "Scene",
            Self::Editor => "Editor",
            Self::Post => "Post",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    Registered,
    Initializing,
    Running,
    Stopping,
    Unloaded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceKind {
    Driver,
    Manager,
    Plugin,
}

impl ServiceKind {
    pub fn from_registry_segment(value: &str) -> Option<Self> {
        Self::from_registry_segment_bytes(value.as_bytes())
    }

    pub(crate) fn from_registry_segment_bytes(value: &[u8]) -> Option<Self> {
        match value {
            b"Driver" => Some(Self::Driver),
            b"Manager" => Some(Self::Manager),
            b"Plugin" => Some(Self::Plugin),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Driver => "Driver",
            Self::Manager => "Manager",
            Self::Plugin => "Plugin",
        }
    }
}

pub trait ModuleLifecycle: Send + Sync {
    fn build(&self, _context: &ModuleContext) -> CoreResult<()> {
        Ok(())
    }

    fn ready(&self, _context: &ModuleContext) -> CoreResult<bool> {
        Ok(true)
    }

    fn finish(&self, _context: &ModuleContext) -> CoreResult<()> {
        Ok(())
    }

    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct NoopModuleLifecycle;

impl ModuleLifecycle for NoopModuleLifecycle {}
