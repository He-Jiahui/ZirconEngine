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

#[cfg(test)]
mod tests {
    use super::{ProjectSortMode, ProjectViewMode};

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
}
