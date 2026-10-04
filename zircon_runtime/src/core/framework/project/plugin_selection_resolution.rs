use std::collections::HashSet;
use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;

use super::{ProjectPluginManifest, ProjectPluginSelection};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 对清单中单项插件选择的处理结果；跳过状态不会进入插件 ID 解析或注册查询。
pub enum PluginSelectionResolutionStatus {
    Resolved,
    InvalidId,
    Duplicate,
    Unsupported,
    SkippedDisabled,
    SkippedTargetMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSelectionResolution {
    pub selection: ProjectPluginSelection,
    pub status: PluginSelectionResolutionStatus,
}

impl PluginSelectionResolution {
    /// 只有必需项的无效 ID、重复 ID 或无提供者才算失败；禁用项和目标不匹配项属于跳过。
    pub fn is_required_failure(&self) -> bool {
        self.selection.required
            && matches!(
                self.status,
                PluginSelectionResolutionStatus::InvalidId
                    | PluginSelectionResolutionStatus::Duplicate
                    | PluginSelectionResolutionStatus::Unsupported
            )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequiredPluginSelectionResolutionError {
    failures: Vec<PluginSelectionResolution>,
}

impl RequiredPluginSelectionResolutionError {
    pub fn failures(&self) -> &[PluginSelectionResolution] {
        &self.failures
    }
}

impl fmt::Display for RequiredPluginSelectionResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut failures = self.failures.iter();
        let Some(first) = failures.next() else {
            return formatter.write_str("required plugin selection resolution failed");
        };
        write!(
            formatter,
            "required plugin selection `{}` is {:?}",
            first.selection.id, first.status
        )?;
        for failure in failures {
            write!(
                formatter,
                "; `{}` is {:?}",
                failure.selection.id, failure.status
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for RequiredPluginSelectionResolutionError {}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 每个选择各有一条 outcome；成功注册单独收集，因此两组数据不是按下标一一对应。
pub struct PluginSelectionResolutionReport<T> {
    outcomes: Vec<PluginSelectionResolution>,
    registrations: Vec<T>,
}

impl<T> PluginSelectionResolutionReport<T> {
    pub fn outcomes(&self) -> &[PluginSelectionResolution] {
        &self.outcomes
    }

    pub fn required_failures(&self) -> impl Iterator<Item = &PluginSelectionResolution> {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.is_required_failure())
    }

    pub fn into_registrations_if_required_resolved(
        self,
    ) -> Result<Self, RequiredPluginSelectionResolutionError> {
        self.into_registrations_if_required_resolved_where(|_| true)
    }

    /// 仅检查谓词选中的必需项；成功时原报告整体返回，不会按谓词裁剪 outcomes 或注册项。
    pub fn into_registrations_if_required_resolved_where(
        self,
        mut required_in_scope: impl FnMut(&ProjectPluginSelection) -> bool,
    ) -> Result<Self, RequiredPluginSelectionResolutionError> {
        let failures = self
            .outcomes
            .iter()
            .filter(|outcome| {
                outcome.is_required_failure() && required_in_scope(&outcome.selection)
            })
            .cloned()
            .collect::<Vec<_>>();
        if failures.is_empty() {
            Ok(self)
        } else {
            Err(RequiredPluginSelectionResolutionError { failures })
        }
    }
}

impl<T> Default for PluginSelectionResolutionReport<T> {
    fn default() -> Self {
        Self {
            outcomes: Vec::new(),
            registrations: Vec::new(),
        }
    }
}

impl<T> Deref for PluginSelectionResolutionReport<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.registrations
    }
}

impl<T> DerefMut for PluginSelectionResolutionReport<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.registrations
    }
}

impl<'a, T> IntoIterator for &'a PluginSelectionResolutionReport<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.registrations.iter()
    }
}

impl<T> IntoIterator for PluginSelectionResolutionReport<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.registrations.into_iter()
    }
}

/// 为清单每项记录一次结果；仅启用、目标匹配、ID 有效且未重复的项会调用 provider。
/// 成功注册与逐项结果分开收集，调用方应先完成必需项检查再消费注册列表。
pub fn resolve_plugin_selections<T>(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    mut provider: impl FnMut(&RuntimePluginId) -> Option<T>,
) -> PluginSelectionResolutionReport<T> {
    let mut seen = HashSet::with_capacity(manifest.selections.len());
    let mut outcomes = Vec::with_capacity(manifest.selections.len());
    let mut registrations = Vec::with_capacity(manifest.selections.len());

    for selection in &manifest.selections {
        let skipped = if !selection.enabled {
            Some(PluginSelectionResolutionStatus::SkippedDisabled)
        } else if !selection.supports_target(target_mode) {
            Some(PluginSelectionResolutionStatus::SkippedTargetMismatch)
        } else {
            None
        };
        if let Some(status) = skipped {
            outcomes.push(PluginSelectionResolution {
                selection: selection.clone(),
                status,
            });
            continue;
        }
        let Some(plugin_id) = RuntimePluginId::parse_key(&selection.id) else {
            outcomes.push(PluginSelectionResolution {
                selection: selection.clone(),
                status: PluginSelectionResolutionStatus::InvalidId,
            });
            continue;
        };
        if !seen.insert(plugin_id.clone()) {
            outcomes.push(PluginSelectionResolution {
                selection: selection.clone(),
                status: PluginSelectionResolutionStatus::Duplicate,
            });
            continue;
        }
        match provider(&plugin_id) {
            Some(registration) => {
                registrations.push(registration);
                outcomes.push(PluginSelectionResolution {
                    selection: selection.clone(),
                    status: PluginSelectionResolutionStatus::Resolved,
                });
            }
            None => outcomes.push(PluginSelectionResolution {
                selection: selection.clone(),
                status: PluginSelectionResolutionStatus::Unsupported,
            }),
        }
    }

    PluginSelectionResolutionReport {
        outcomes,
        registrations,
    }
}

#[cfg(test)]
#[path = "tests/plugin_selection_resolution.rs"]
mod tests;
