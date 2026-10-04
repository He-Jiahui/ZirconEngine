//! 保存项目页的过滤、排序、布局和内部子页选择；这些编号跨配置与 Web 请求保持一致。
//! 外观布局选择不改变项目身份，过滤/排序由快照投影执行。

use serde::{Deserialize, Serialize};

/// 基于投影时路径可用性决定显示集合；不会删除记录或改变动作目标。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectFilterMode {
    #[default]
    All,
    Existing,
    Missing,
}

impl ProjectFilterMode {
    pub fn id(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Existing => "existing",
            Self::Missing => "missing",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All Projects",
            Self::Existing => "Existing",
            Self::Missing => "Missing",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Existing,
            Self::Existing => Self::Missing,
            Self::Missing => Self::All,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "all" | "all-projects" | "projects" => Some(Self::All),
            "existing" | "available" => Some(Self::Existing),
            "missing" | "missing-paths" => Some(Self::Missing),
            _ => None,
        }
    }
}

/// 项目列表的持久化排序偏好，快照负责实际排序。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectSortMode {
    #[default]
    LastModified,
    Name,
}

impl ProjectSortMode {
    pub fn id(self) -> &'static str {
        match self {
            Self::LastModified => "last-modified",
            Self::Name => "name",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::LastModified => "Last Modified",
            Self::Name => "Name",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::LastModified => Self::Name,
            Self::Name => Self::LastModified,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "last-modified" | "modified" | "recent" => Some(Self::LastModified),
            "name" | "title" => Some(Self::Name),
            _ => None,
        }
    }
}

/// 项目浏览器的展示形态偏好；同一项目身份可用于卡片和表格。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectViewMode {
    #[default]
    Grid,
    List,
}

impl ProjectViewMode {
    pub fn id(self) -> &'static str {
        match self {
            Self::Grid => "grid",
            Self::List => "list",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "grid" | "cards" => Some(Self::Grid),
            "list" | "table" => Some(Self::List),
            _ => None,
        }
    }
}

/// 项目主页面内部流程的位置，供创建对话框、列表和详情的状态恢复使用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectSubpage {
    #[default]
    Dashboard,
    NewProject,
    ProjectBrowser,
    ProjectDetail,
}

impl ProjectSubpage {
    pub fn id(self) -> &'static str {
        match self {
            Self::Dashboard => "dashboard",
            Self::NewProject => "new-project",
            Self::ProjectBrowser => "project-browser",
            Self::ProjectDetail => "project-detail",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "dashboard" | "projects" => Some(Self::Dashboard),
            "new-project" | "create-project" => Some(Self::NewProject),
            "project-browser" | "browser" | "project-list" | "all-projects" => {
                Some(Self::ProjectBrowser)
            }
            "project-detail" | "detail" => Some(Self::ProjectDetail),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "tests/project_view.rs"]
mod tests;
