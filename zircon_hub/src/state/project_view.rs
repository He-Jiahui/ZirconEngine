#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
mod tests {
    use super::{ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode};

    #[test]
    fn project_filter_mode_cycles_through_supported_modes() {
        assert_eq!(ProjectFilterMode::All.next(), ProjectFilterMode::Existing);
        assert_eq!(
            ProjectFilterMode::Existing.next(),
            ProjectFilterMode::Missing
        );
        assert_eq!(ProjectFilterMode::Missing.next(), ProjectFilterMode::All);
        assert_eq!(
            ProjectFilterMode::from_id("available"),
            Some(ProjectFilterMode::Existing)
        );
    }

    #[test]
    fn project_sort_mode_cycles_between_supported_modes() {
        assert_eq!(ProjectSortMode::LastModified.next(), ProjectSortMode::Name);
        assert_eq!(ProjectSortMode::Name.next(), ProjectSortMode::LastModified);
    }

    #[test]
    fn project_view_mode_parses_ui_ids() {
        assert_eq!(
            ProjectViewMode::from_id("grid"),
            Some(ProjectViewMode::Grid)
        );
        assert_eq!(
            ProjectViewMode::from_id("TABLE"),
            Some(ProjectViewMode::List)
        );
        assert_eq!(ProjectViewMode::from_id("unknown"), None);
    }

    #[test]
    fn project_subpage_parses_internal_page_ids() {
        assert_eq!(
            ProjectSubpage::from_id("new-project"),
            Some(ProjectSubpage::NewProject)
        );
        assert_eq!(ProjectSubpage::ProjectBrowser.id(), "project-browser");
        assert_eq!(ProjectSubpage::from_id("missing"), None);
    }
}
