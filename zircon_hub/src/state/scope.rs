//! 统一界面和动作对“已选项目”“最近项目”和“源码引擎”的解释。
//! 这里依据注册表生成逻辑范围，磁盘存在性、清单有效性和构建产物可用性由执行路径校验。

use std::path::{Path, PathBuf};

use crate::engines::SourceEngineInstall;
use crate::projects::{metadata_for_path, project_paths_match, ProjectMetadataMap, RecentProject};

/// Canonical action/view scope derived once from the Hub snapshot inputs.
/// UI and runtime code should consult this model instead of independently
/// guessing whether a page action targets a selected project, a fallback project,
/// an active Source Engine, or the whole Hub installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HubScope {
    pub project: ProjectScope,
    pub source_engine: SourceEngineScope,
}

/// 保留显式选择与最近项目后备的区别；过期的显式选择不得悄悄变成另一个项目。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectScope {
    Selected(ProjectScopeProject),
    StaleSelection { requested_path: PathBuf },
    LatestRecent(ProjectScopeProject),
    None,
}

/// 动作和目录扫描需要的项目逻辑身份及绑定状态；不代表磁盘项目已验证。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectScopeProject {
    pub display_name: String,
    pub path: PathBuf,
    pub engine_id: Option<String>,
    pub engine_state: ProjectEngineScopeState,
}

/// 只反映项目是否绑定已注册引擎；就绪不保证检出目录或构建产物可执行。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectEngineScopeState {
    Ready,
    MissingBinding,
    Unavailable,
}

/// 选中项目优先约束引擎范围；没有有效显式项目时才允许活动引擎/首个引擎后备。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceEngineScope {
    ProjectBound(SourceEngineScopeEngine),
    ProjectUnbound {
        project_name: String,
    },
    ProjectEngineUnavailable {
        project_name: String,
        engine_id: String,
    },
    Active(SourceEngineScopeEngine),
    None,
}

/// 供范围消费端再次查注册表的轻量引擎身份，不复制执行配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceEngineScopeEngine {
    pub id: String,
    pub display_name: String,
}

impl HubScope {
    /// 从同一组快照输入生成范围，供界面与运行时共同使用；调用后仍须执行具体动作的有效性校验。
    pub fn resolve(
        selected_project_path: Option<&Path>,
        recent_projects: &[RecentProject],
        project_metadata: &ProjectMetadataMap,
        engines: &[SourceEngineInstall],
        active_engine_id: Option<&str>,
    ) -> Self {
        let project = resolve_project_scope(
            selected_project_path,
            recent_projects,
            project_metadata,
            engines,
        );
        // Source Engine scope is intentionally derived after project scope so a
        // selected project can prevent accidental active-engine fallback.
        let source_engine = resolve_source_engine_scope(&project, engines, active_engine_id);
        Self {
            project,
            source_engine,
        }
    }

    /// 用于只能作用于显式选择的目录扫描或绑定操作，不采用最近项目后备。
    pub fn selected_project(&self) -> Option<&ProjectScopeProject> {
        match &self.project {
            ProjectScope::Selected(project) => Some(project),
            ProjectScope::StaleSelection { .. }
            | ProjectScope::LatestRecent(_)
            | ProjectScope::None => None,
        }
    }

    /// 用于允许最近项目后备的动作；显式过期选择仍返回空值。
    pub fn selected_or_latest_project(&self) -> Option<&ProjectScopeProject> {
        match &self.project {
            ProjectScope::Selected(project) | ProjectScope::LatestRecent(project) => Some(project),
            ProjectScope::StaleSelection { .. } | ProjectScope::None => None,
        }
    }

    /// 供调用者区分“没有选择”与“选择已经失效”，避免静默改换目标。
    pub fn has_stale_selected_project(&self) -> bool {
        matches!(self.project, ProjectScope::StaleSelection { .. })
    }
}

impl ProjectScopeProject {
    /// 表示逻辑绑定允许尝试构建；实际源码、项目清单和工具链校验仍由构建动作负责。
    pub fn can_build(&self) -> bool {
        self.engine_state == ProjectEngineScopeState::Ready
    }
}

impl SourceEngineScope {
    /// 返回可查注册表的范围身份；选中项目缺少有效绑定时不给出活动引擎替代。
    pub fn engine_id(&self) -> Option<&str> {
        match self {
            Self::ProjectBound(engine) | Self::Active(engine) => Some(&engine.id),
            Self::ProjectUnbound { .. } | Self::ProjectEngineUnavailable { .. } | Self::None => {
                None
            }
        }
    }
}

fn resolve_project_scope(
    selected_project_path: Option<&Path>,
    recent_projects: &[RecentProject],
    project_metadata: &ProjectMetadataMap,
    engines: &[SourceEngineInstall],
) -> ProjectScope {
    if let Some(selected_path) = selected_project_path {
        if let Some(project) = recent_projects
            .iter()
            .find(|project| project_paths_match(&project.path, selected_path))
        {
            return ProjectScope::Selected(project_scope_project(
                project,
                project_metadata,
                engines,
            ));
        }
        return ProjectScope::StaleSelection {
            requested_path: selected_path.to_path_buf(),
        };
    }

    recent_projects
        .iter()
        .max_by_key(|project| project.last_opened_unix_ms)
        .map(|project| {
            ProjectScope::LatestRecent(project_scope_project(project, project_metadata, engines))
        })
        .unwrap_or(ProjectScope::None)
}

fn project_scope_project(
    project: &RecentProject,
    project_metadata: &ProjectMetadataMap,
    engines: &[SourceEngineInstall],
) -> ProjectScopeProject {
    let engine_id = metadata_for_path(project_metadata, &project.path)
        .and_then(|metadata| metadata.engine_id.clone());
    let engine_state = match engine_id.as_deref() {
        None => ProjectEngineScopeState::MissingBinding,
        Some(id) if engines.iter().any(|engine| engine.id == id) => ProjectEngineScopeState::Ready,
        Some(_) => ProjectEngineScopeState::Unavailable,
    };
    ProjectScopeProject {
        display_name: project_display_name(project),
        path: project.path.clone(),
        engine_id,
        engine_state,
    }
}

// 有明确选中项目时，缺失/不可用绑定必须继续显式呈现；活动引擎不能覆盖该约束。
fn resolve_source_engine_scope(
    project: &ProjectScope,
    engines: &[SourceEngineInstall],
    active_engine_id: Option<&str>,
) -> SourceEngineScope {
    match project {
        ProjectScope::Selected(project) => match project.engine_state {
            ProjectEngineScopeState::Ready => project
                .engine_id
                .as_deref()
                .and_then(|engine_id| engines.iter().find(|engine| engine.id == engine_id))
                .map(source_engine_scope_engine)
                .map(SourceEngineScope::ProjectBound)
                .unwrap_or_else(|| SourceEngineScope::ProjectEngineUnavailable {
                    project_name: project.display_name.clone(),
                    engine_id: project.engine_id.clone().unwrap_or_default(),
                }),
            ProjectEngineScopeState::MissingBinding => SourceEngineScope::ProjectUnbound {
                project_name: project.display_name.clone(),
            },
            ProjectEngineScopeState::Unavailable => SourceEngineScope::ProjectEngineUnavailable {
                project_name: project.display_name.clone(),
                engine_id: project.engine_id.clone().unwrap_or_default(),
            },
        },
        ProjectScope::StaleSelection { .. }
        | ProjectScope::LatestRecent(_)
        | ProjectScope::None => engines
            .iter()
            .find(|engine| active_engine_id == Some(engine.id.as_str()))
            .or_else(|| engines.first())
            .map(source_engine_scope_engine)
            .map(SourceEngineScope::Active)
            .unwrap_or(SourceEngineScope::None),
    }
}

fn source_engine_scope_engine(engine: &SourceEngineInstall) -> SourceEngineScopeEngine {
    SourceEngineScopeEngine {
        id: engine.id.clone(),
        display_name: engine.display_name.clone(),
    }
}

fn project_display_name(project: &RecentProject) -> String {
    if project.summary.name.trim().is_empty() {
        return project
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Zircon Project")
            .to_string();
    }
    project.summary.name.clone()
}

#[cfg(test)]
#[path = "tests/scope.rs"]
mod tests;
