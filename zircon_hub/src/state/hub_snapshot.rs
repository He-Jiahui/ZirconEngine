//! 将运行时数据汇集为只读投影输入；项目可用性另用缓存保存，避免每次界面投影重复访问文件系统。
//! 逻辑选择、缓存中的路径存在性和真正执行动作的校验是三个不同时间点的事实。

use crate::assets::AssetCatalogEntry;
use crate::engines::SourceEngineInstall;
use crate::learn::LearnCatalogEntry;
use crate::plugins::PluginCatalogEntry;
use crate::projects::{ProjectMetadataMap, RecentProject};
use crate::settings::HubSettings;
use crate::team::TeamOverview;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{
    HubActionRecord, HubPage, HubScope, ProjectFilterMode, ProjectSortMode, ProjectSubpage,
    ProjectViewMode, TaskStatus,
};

/// 一次显式刷新形成的路径存在性缓存；普通投影只同步路径集合并保留已有探测结果。
/// 它是显示时的快照，执行创建、构建或删除动作仍须重新校验实际文件系统。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ProjectAvailabilitySnapshot {
    by_path: HashMap<PathBuf, bool>,
}

impl ProjectAvailabilitySnapshot {
    pub(crate) fn capture(projects: &[RecentProject]) -> Self {
        Self::capture_with_selected(projects, None)
    }

    /// 用于建立初始投影；额外纳入已选但不在最近列表中的路径，以支持过期选择详情。
    pub(crate) fn capture_with_selected(
        projects: &[RecentProject],
        selected_path: Option<&Path>,
    ) -> Self {
        Self::capture_with_selected_and_probe(projects, selected_path, Path::exists)
    }

    pub(crate) fn synchronize(&mut self, projects: &[RecentProject]) -> bool {
        self.synchronize_with_selected(projects, None)
    }

    /// 在普通投影前同步记录集合，仅探测新增路径；已知路径的磁盘变化须由刷新入口更新。
    pub(crate) fn synchronize_with_selected(
        &mut self,
        projects: &[RecentProject],
        selected_path: Option<&Path>,
    ) -> bool {
        self.synchronize_with_selected_and_probe(projects, selected_path, Path::exists)
    }

    pub(crate) fn refresh(&mut self, projects: &[RecentProject]) -> bool {
        self.refresh_with_selected(projects, None)
    }

    /// 由显式刷新或焦点恢复路径调用，重新探测所有相关路径；返回值表示缓存内容是否变化。
    pub(crate) fn refresh_with_selected(
        &mut self,
        projects: &[RecentProject],
        selected_path: Option<&Path>,
    ) -> bool {
        self.refresh_with_selected_and_probe(projects, selected_path, Path::exists)
    }

    /// 供过滤和详情显示读取缓存；未纳入本缓存的路径按不可用处理。
    pub(crate) fn path_exists(&self, path: &Path) -> bool {
        self.by_path.get(path).copied().unwrap_or(false)
    }

    fn capture_with_probe(
        projects: &[RecentProject],
        mut probe: impl FnMut(&Path) -> bool,
    ) -> Self {
        Self::capture_with_selected_and_probe(projects, None, &mut probe)
    }

    fn capture_with_selected_and_probe(
        projects: &[RecentProject],
        selected_path: Option<&Path>,
        mut probe: impl FnMut(&Path) -> bool,
    ) -> Self {
        let mut snapshot = Self::default();
        snapshot.synchronize_with_selected_and_probe(projects, selected_path, &mut probe);
        snapshot
    }

    fn synchronize_with_probe(
        &mut self,
        projects: &[RecentProject],
        mut probe: impl FnMut(&Path) -> bool,
    ) -> bool {
        self.synchronize_with_selected_and_probe(projects, None, &mut probe)
    }

    fn synchronize_with_selected_and_probe(
        &mut self,
        projects: &[RecentProject],
        selected_path: Option<&Path>,
        mut probe: impl FnMut(&Path) -> bool,
    ) -> bool {
        let selected_outside_recents =
            selected_path.is_some_and(|path| !projects.iter().any(|project| project.path == path));
        let expected_len = projects.len() + usize::from(selected_outside_recents);
        // 路径集合未变时保留探测事实；单纯重绘不应成为隐式磁盘刷新。
        if self.by_path.len() == expected_len
            && projects
                .iter()
                .all(|project| self.by_path.contains_key(&project.path))
            && selected_path.is_none_or(|path| self.by_path.contains_key(path))
        {
            return false;
        }

        let mut synchronized = HashMap::with_capacity(expected_len);
        for project in projects {
            let exists = self
                .by_path
                .remove(&project.path)
                .unwrap_or_else(|| probe(&project.path));
            synchronized.insert(project.path.clone(), exists);
        }
        if let Some(path) = selected_path {
            if !synchronized.contains_key(path) {
                let exists = self.by_path.remove(path).unwrap_or_else(|| probe(path));
                synchronized.insert(path.to_path_buf(), exists);
            }
        }
        self.by_path = synchronized;
        true
    }

    fn refresh_with_probe(
        &mut self,
        projects: &[RecentProject],
        mut probe: impl FnMut(&Path) -> bool,
    ) -> bool {
        self.refresh_with_selected_and_probe(projects, None, &mut probe)
    }

    fn refresh_with_selected_and_probe(
        &mut self,
        projects: &[RecentProject],
        selected_path: Option<&Path>,
        mut probe: impl FnMut(&Path) -> bool,
    ) -> bool {
        let refreshed = Self::capture_with_selected_and_probe(projects, selected_path, &mut probe);
        if *self == refreshed {
            return false;
        }
        *self = refreshed;
        true
    }
}

/// 供各视图使用的运行时数据切片；它不持有可变会话、文件句柄或正在执行的工作者。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HubSnapshot {
    pub selected_page: HubPage,
    pub project_filter: ProjectFilterMode,
    pub project_sort: ProjectSortMode,
    pub project_view_mode: ProjectViewMode,
    pub project_subpage: ProjectSubpage,
    pub search_query: String,
    pub selected_project_path: Option<PathBuf>,
    pub new_project_name: String,
    pub selected_template_id: String,
    pub new_project_location: PathBuf,
    pub new_project_engine_id: Option<String>,
    pub pending_delete_project_path: Option<PathBuf>,
    pub task_status: TaskStatus,
    pub queued_background_actions: usize,
    pub recent_projects: Vec<RecentProject>,
    pub project_metadata: ProjectMetadataMap,
    pub assets: Vec<AssetCatalogEntry>,
    pub learn_resources: Vec<LearnCatalogEntry>,
    pub plugins: Vec<PluginCatalogEntry>,
    pub team: TeamOverview,
    pub action_history: Vec<HubActionRecord>,
    pub engines: Vec<SourceEngineInstall>,
    pub active_engine_id: Option<String>,
    pub settings: HubSettings,
    pub settings_draft: HubSettings,
}

impl HubSnapshot {
    /// 按当前选择和注册表解析动作/目录范围；它不执行磁盘有效性或工具链检查。
    pub fn scope(&self) -> HubScope {
        HubScope::resolve(
            self.selected_project_path.as_deref(),
            &self.recent_projects,
            &self.project_metadata,
            &self.engines,
            self.active_engine_id.as_deref(),
        )
    }

    /// 为独立快照消费者提供一次性磁盘探测后的项目列表；持续界面投影应复用会话可用性缓存。
    pub fn filtered_recent_projects(&self) -> Vec<RecentProject> {
        let availability = ProjectAvailabilitySnapshot::capture(&self.recent_projects);
        self.filtered_recent_projects_with_availability(&availability)
    }

    /// 过滤、搜索和排序共享同一可用性快照，避免一次界面投影中读取不一致的磁盘结果。
    pub(crate) fn filtered_recent_projects_with_availability(
        &self,
        availability: &ProjectAvailabilitySnapshot,
    ) -> Vec<RecentProject> {
        let query = self.search_query.trim().to_ascii_lowercase();
        let mut projects: Vec<_> = self
            .recent_projects
            .iter()
            .filter(|project| self.project_filter.includes(project, availability))
            .filter(|project| query.is_empty() || project_matches_query(project, &query))
            .cloned()
            .collect();

        // TODO: [CR-HUBSTATE-0005] 确认“最后修改”应按最近打开时间还是磁盘修改时间排序；当前仅使用打开记录，需核对项目页文案与排序契约。
        match self.project_sort {
            ProjectSortMode::LastModified => projects
                .sort_by(|left, right| right.last_opened_unix_ms.cmp(&left.last_opened_unix_ms)),
            ProjectSortMode::Name => {
                projects.sort_by_key(|project| project_display_name(project).to_ascii_lowercase());
            }
        }

        projects
    }
}

impl ProjectFilterMode {
    fn includes(self, project: &RecentProject, availability: &ProjectAvailabilitySnapshot) -> bool {
        match self {
            Self::All => true,
            Self::Existing => availability.path_exists(&project.path),
            Self::Missing => !availability.path_exists(&project.path),
        }
    }
}

fn project_matches_query(project: &RecentProject, query: &str) -> bool {
    project.summary.name.to_ascii_lowercase().contains(query)
        || project
            .path
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains(query)
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
#[path = "tests/hub_snapshot.rs"]
mod tests;
