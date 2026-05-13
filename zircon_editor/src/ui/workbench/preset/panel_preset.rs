use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FyroxPanelComponentRole {
    AssetGrid,
    AssetList,
    CategorizedList,
    ContextMenu,
    FieldEditor,
    FilterBar,
    FolderTree,
    GizmoControls,
    GraphCanvas,
    InspectorSection,
    MetadataPane,
    PaneToolbar,
    PreviewPane,
    PropertyGrid,
    SearchField,
    SeverityChips,
    SourceEditor,
    StatusActionControls,
    Timeline,
    TreeView,
    VirtualList,
    ViewportHost,
    VisualDesigner,
}

impl FyroxPanelComponentRole {
    pub fn component_id(self) -> &'static str {
        match self {
            Self::AssetGrid => "AssetGrid",
            Self::AssetList => "AssetList",
            Self::CategorizedList => "CategorizedList",
            Self::ContextMenu => "ContextMenu",
            Self::FieldEditor => "FieldEditor",
            Self::FilterBar => "FilterBar",
            Self::FolderTree => "FolderTree",
            Self::GizmoControls => "GizmoControls",
            Self::GraphCanvas => "GraphCanvas",
            Self::InspectorSection => "InspectorSection",
            Self::MetadataPane => "MetadataPane",
            Self::PaneToolbar => "PaneToolbar",
            Self::PreviewPane => "PreviewPane",
            Self::PropertyGrid => "PropertyGrid",
            Self::SearchField => "SearchField",
            Self::SeverityChips => "SeverityChips",
            Self::SourceEditor => "SourceEditor",
            Self::StatusActionControls => "StatusActionControls",
            Self::Timeline => "Timeline",
            Self::TreeView => "TreeView",
            Self::VirtualList => "VirtualList",
            Self::ViewportHost => "ViewportHost",
            Self::VisualDesigner => "VisualDesigner",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FyroxPanelInteraction {
    AssetPreview,
    ContextMenu,
    DataRefresh,
    DetachAttach,
    MetadataEdit,
    PluginEnableDisable,
    PropertyEdit,
    SceneGizmo,
    SearchFilter,
    SelectionSync,
    SeverityFilter,
    SourceEdit,
    TimelineScrub,
    VirtualizedScroll,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FyroxPanelPreset {
    pub view_id: String,
    pub title: String,
    pub components: Vec<FyroxPanelComponentRole>,
    pub interactions: Vec<FyroxPanelInteraction>,
}

impl FyroxPanelPreset {
    pub fn new(view_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            view_id: view_id.into(),
            title: title.into(),
            components: Vec::new(),
            interactions: Vec::new(),
        }
    }

    pub fn with_components(
        mut self,
        components: impl IntoIterator<Item = FyroxPanelComponentRole>,
    ) -> Self {
        self.components = components.into_iter().collect();
        self
    }

    pub fn with_interactions(
        mut self,
        interactions: impl IntoIterator<Item = FyroxPanelInteraction>,
    ) -> Self {
        self.interactions = interactions.into_iter().collect();
        self
    }
}
