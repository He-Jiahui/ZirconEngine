use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};

use super::ScheduleError;

/// Dense identifier for a lowercase ASCII `<plugin>.<set>` name such as `physics.main`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SystemSetId(u32);

impl SystemSetId {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// 将命名系统集合映射为日程局部 ID；SystemRef::Set 与系统元数据必须使用同一注册表的 ID。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystemSetRegistry {
    names: Vec<String>,
    ids_by_name: HashMap<String, SystemSetId>,
}

impl SystemSetRegistry {
    pub fn intern<'a>(
        &mut self,
        name: impl Into<Cow<'a, str>>,
    ) -> Result<SystemSetId, ScheduleError> {
        let name = name.into();
        validate_system_set_name(name.as_ref())?;
        if let Some(id) = self.ids_by_name.get(name.as_ref()).copied() {
            return Ok(id);
        }

        let name = name.into_owned();
        let id = SystemSetId::from_raw(self.names.len() as u32);
        self.names.push(name.clone());
        self.ids_by_name.insert(name, id);
        Ok(id)
    }

    pub fn name(&self, id: SystemSetId) -> Option<&str> {
        self.names.get(id.index()).map(String::as_str)
    }
}

fn validate_system_set_name(name: &str) -> Result<(), ScheduleError> {
    if name.trim().is_empty() || name.trim() != name {
        return Err(ScheduleError::EmptySystemSetName);
    }
    let mut segments = name.split('.');
    let (Some(plugin), Some(set), None) = (segments.next(), segments.next(), segments.next())
    else {
        return Err(ScheduleError::InvalidSystemSetName(name.to_string()));
    };
    if !is_lowercase_system_set_token(plugin) || !is_lowercase_system_set_token(set) {
        return Err(ScheduleError::InvalidSystemSetName(name.to_string()));
    }
    Ok(())
}

fn is_lowercase_system_set_token(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
#[path = "tests/system_set.rs"]
mod tests;
