//! Workbench layout model and mutation logic for the editor shell.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::autolayout::{PaneConstraintOverride, ShellRegionId};
use crate::project::ProjectEditorWorkspace;
use crate::view::{ViewHost, ViewInstanceId, ViewRegistry};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MainPageId(pub(crate) String);

impl MainPageId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn workbench() -> Self {
        Self::new("workbench")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ActivityDrawerSlot {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
    BottomLeft,
    BottomRight,
}

impl ActivityDrawerSlot {
    pub const ALL: [Self; 6] = [
        Self::LeftTop,
        Self::LeftBottom,
        Self::RightTop,
        Self::RightBottom,
        Self::BottomLeft,
        Self::BottomRight,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityDrawerMode {
    Pinned,
    AutoHide,
    Collapsed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitPlacement {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceTarget {
    MainPage(MainPageId),
    FloatingWindow(MainPageId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TabStackLayout {
    pub tabs: Vec<ViewInstanceId>,
    pub active_tab: Option<ViewInstanceId>,
}

impl TabStackLayout {
    fn insert(&mut self, instance_id: ViewInstanceId, anchor: Option<&TabInsertionAnchor>) {
        self.tabs.retain(|current| current != &instance_id);

        if let Some(anchor) = anchor {
            if let Some(anchor_index) = self.tabs.iter().position(|current| current == &anchor.target_id)
            {
                let insert_index = match anchor.side {
                    TabInsertionSide::Before => anchor_index,
                    TabInsertionSide::After => anchor_index + 1,
                };
                self.tabs.insert(insert_index.min(self.tabs.len()), instance_id.clone());
                self.active_tab = Some(instance_id);
                return;
            }
        }

        self.tabs.push(instance_id.clone());
        self.active_tab = Some(instance_id);
    }

    fn remove(&mut self, instance_id: &ViewInstanceId) -> bool {
        let before = self.tabs.len();
        self.tabs.retain(|current| current != instance_id);
        if self.active_tab.as_ref() == Some(instance_id) {
            self.active_tab = self.tabs.last().cloned();
        }
        before != self.tabs.len()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DocumentNode {
    SplitNode {
        axis: SplitAxis,
        ratio: f32,
        first: Box<DocumentNode>,
        second: Box<DocumentNode>,
    },
    Tabs(TabStackLayout),
}

impl Default for DocumentNode {
    fn default() -> Self {
        Self::Tabs(TabStackLayout::default())
    }
}

impl DocumentNode {
    fn node_at_path_mut(&mut self, path: &[usize]) -> Option<&mut DocumentNode> {
        if path.is_empty() {
            return Some(self);
        }

        match self {
            Self::Tabs(_) if path.len() == 1 && path[0] == 0 => Some(self),
            Self::SplitNode { first, second, .. } => match path[0] {
                0 => first.node_at_path_mut(&path[1..]),
                1 => second.node_at_path_mut(&path[1..]),
                _ => None,
            },
            Self::Tabs(_) => None,
        }
    }

    fn remove_instance(&mut self, instance_id: &ViewInstanceId) -> bool {
        match self {
            Self::Tabs(stack) => stack.remove(instance_id),
            Self::SplitNode { first, second, .. } => {
                first.remove_instance(instance_id) || second.remove_instance(instance_id)
            }
        }
    }

    fn contains(&self, instance_id: &ViewInstanceId) -> bool {
        match self {
            Self::Tabs(stack) => stack.tabs.contains(instance_id),
            Self::SplitNode { first, second, .. } => {
                first.contains(instance_id) || second.contains(instance_id)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivityDrawerLayout {
    pub slot: ActivityDrawerSlot,
    pub tab_stack: TabStackLayout,
    pub active_view: Option<ViewInstanceId>,
    pub mode: ActivityDrawerMode,
    pub extent: f32,
    pub visible: bool,
}

impl ActivityDrawerLayout {
    pub fn new(slot: ActivityDrawerSlot) -> Self {
        Self {
            slot,
            tab_stack: TabStackLayout::default(),
            active_view: None,
            mode: ActivityDrawerMode::Pinned,
            extent: if matches!(
                slot,
                ActivityDrawerSlot::BottomLeft | ActivityDrawerSlot::BottomRight
            ) {
                200.0
            } else {
                260.0
            },
            visible: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MainHostPageLayout {
    WorkbenchPage {
        id: MainPageId,
        title: String,
        document_workspace: DocumentNode,
    },
    ExclusiveActivityWindowPage {
        id: MainPageId,
        title: String,
        window_instance: ViewInstanceId,
    },
}

impl MainHostPageLayout {
    pub fn id(&self) -> &MainPageId {
        match self {
            Self::WorkbenchPage { id, .. } | Self::ExclusiveActivityWindowPage { id, .. } => id,
        }
    }

    fn document_workspace_mut(&mut self) -> Option<&mut DocumentNode> {
        match self {
            Self::WorkbenchPage {
                document_workspace, ..
            } => Some(document_workspace),
            Self::ExclusiveActivityWindowPage { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FloatingWindowLayout {
    pub window_id: MainPageId,
    pub title: String,
    pub workspace: DocumentNode,
    pub focused_view: Option<ViewInstanceId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkbenchLayout {
    pub active_main_page: MainPageId,
    pub main_pages: Vec<MainHostPageLayout>,
    pub drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    pub floating_windows: Vec<FloatingWindowLayout>,
    #[serde(default)]
    pub region_overrides: BTreeMap<ShellRegionId, PaneConstraintOverride>,
    #[serde(default)]
    pub view_overrides: BTreeMap<ViewInstanceId, PaneConstraintOverride>,
}

impl Default for WorkbenchLayout {
    fn default() -> Self {
        Self {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                document_workspace: DocumentNode::default(),
            }],
            drawers: ActivityDrawerSlot::ALL
                .into_iter()
                .map(|slot| (slot, ActivityDrawerLayout::new(slot)))
                .collect(),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutDiff {
    pub changed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutNormalizationReport {
    pub placeholders: Vec<ViewInstanceId>,
    pub removed_missing_active_tabs: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DragPayload {
    pub instance_id: ViewInstanceId,
    pub kind: crate::view::ViewKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HitTarget {
    Drawer(ActivityDrawerSlot),
    Document(MainPageId, Vec<usize>),
    DocumentEdge {
        page_id: MainPageId,
        path: Vec<usize>,
        edge: DockEdge,
    },
    FloatingWindow(MainPageId, Vec<usize>),
    FloatingWindowEdge {
        window_id: MainPageId,
        path: Vec<usize>,
        edge: DockEdge,
    },
    ExclusivePage(MainPageId),
    NewFloatingWindow,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DropTarget {
    Host(ViewHost),
    Split {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
        axis: SplitAxis,
        placement: SplitPlacement,
    },
    NewFloatingWindow,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RestorePolicy {
    ProjectThenGlobal,
    PresetThenProjectThenGlobal { preset: Option<WorkbenchLayout> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabInsertionSide {
    Before,
    After,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabInsertionAnchor {
    pub target_id: ViewInstanceId,
    pub side: TabInsertionSide,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LayoutCommand {
    OpenView {
        instance_id: ViewInstanceId,
        target: ViewHost,
    },
    CloseView {
        instance_id: ViewInstanceId,
    },
    FocusView {
        instance_id: ViewInstanceId,
    },
    MoveView {
        instance_id: ViewInstanceId,
        target: ViewHost,
    },
    AttachView {
        instance_id: ViewInstanceId,
        target: ViewHost,
        anchor: Option<TabInsertionAnchor>,
    },
    DetachViewToWindow {
        instance_id: ViewInstanceId,
        new_window: MainPageId,
    },
    CreateSplit {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
        axis: SplitAxis,
        placement: SplitPlacement,
        new_instance: ViewInstanceId,
    },
    ResizeSplit {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
        ratio: f32,
    },
    SetDrawerMode {
        slot: ActivityDrawerSlot,
        mode: ActivityDrawerMode,
    },
    SetDrawerExtent {
        slot: ActivityDrawerSlot,
        extent: f32,
    },
    ActivateDrawerTab {
        slot: ActivityDrawerSlot,
        instance_id: ViewInstanceId,
    },
    ActivateMainPage {
        page_id: MainPageId,
    },
    SavePreset {
        name: String,
    },
    LoadPreset {
        name: String,
    },
    ResetToDefault,
}

#[derive(Clone, Debug, Default)]
pub struct LayoutManager;

impl LayoutManager {
    pub fn default_layout(&self) -> WorkbenchLayout {
        crate::host::manager::builtin_hybrid_layout()
    }

    pub fn load_global_default(&self, config: Option<WorkbenchLayout>) -> Option<WorkbenchLayout> {
        config
    }

    pub fn load_project_workspace(
        &self,
        workspace: Option<ProjectEditorWorkspace>,
    ) -> Option<ProjectEditorWorkspace> {
        workspace
    }

    pub fn save_global_default(&self, layout: &WorkbenchLayout) -> WorkbenchLayout {
        layout.clone()
    }

    pub fn save_project_workspace(
        &self,
        workspace: &ProjectEditorWorkspace,
    ) -> ProjectEditorWorkspace {
        workspace.clone()
    }

    pub fn restore_workspace(
        &self,
        policy: RestorePolicy,
        project_workspace: Option<ProjectEditorWorkspace>,
        global_default: Option<WorkbenchLayout>,
    ) -> Result<WorkbenchLayout, String> {
        Ok(match policy {
            RestorePolicy::ProjectThenGlobal => project_workspace
                .map(|workspace| workspace.workbench)
                .or(global_default)
                .unwrap_or_default(),
            RestorePolicy::PresetThenProjectThenGlobal { preset } => preset
                .or_else(|| project_workspace.map(|workspace| workspace.workbench))
                .or(global_default)
                .unwrap_or_default(),
        })
    }

    pub fn resolve_drop(&self, payload: DragPayload, target: HitTarget) -> DropTarget {
        match target {
            HitTarget::Drawer(slot) => DropTarget::Host(ViewHost::Drawer(slot)),
            HitTarget::Document(page_id, path) => {
                DropTarget::Host(ViewHost::Document(page_id, path))
            }
            HitTarget::DocumentEdge {
                page_id,
                path,
                edge,
            } => DropTarget::Split {
                workspace: WorkspaceTarget::MainPage(page_id),
                path,
                axis: edge_axis(edge),
                placement: edge_placement(edge),
            },
            HitTarget::FloatingWindow(window_id, path) => {
                DropTarget::Host(ViewHost::FloatingWindow(window_id, path))
            }
            HitTarget::FloatingWindowEdge {
                window_id,
                path,
                edge,
            } => DropTarget::Split {
                workspace: WorkspaceTarget::FloatingWindow(window_id),
                path,
                axis: edge_axis(edge),
                placement: edge_placement(edge),
            },
            HitTarget::ExclusivePage(page_id) => {
                let _ = payload;
                DropTarget::Host(ViewHost::ExclusivePage(page_id))
            }
            HitTarget::NewFloatingWindow => DropTarget::NewFloatingWindow,
        }
    }

    pub fn normalize(
        &self,
        layout: &mut WorkbenchLayout,
        _registry: &ViewRegistry,
    ) -> LayoutNormalizationReport {
        for slot in ActivityDrawerSlot::ALL {
            layout
                .drawers
                .entry(slot)
                .or_insert_with(|| ActivityDrawerLayout::new(slot));
        }

        let mut removed_missing_active_tabs = 0;
        for drawer in layout.drawers.values_mut() {
            if let Some(active) = drawer.tab_stack.active_tab.clone() {
                if !drawer.tab_stack.tabs.contains(&active) {
                    drawer.tab_stack.active_tab = drawer.tab_stack.tabs.first().cloned();
                    removed_missing_active_tabs += 1;
                }
            }
            if let Some(active) = drawer.active_view.clone() {
                if !drawer.tab_stack.tabs.contains(&active) {
                    drawer.active_view = drawer.tab_stack.active_tab.clone();
                    removed_missing_active_tabs += 1;
                }
            }
        }

        if !layout
            .main_pages
            .iter()
            .any(|page| page.id() == &layout.active_main_page)
        {
            layout.active_main_page = layout
                .main_pages
                .first()
                .map(|page| page.id().clone())
                .unwrap_or_else(MainPageId::workbench);
        }

        LayoutNormalizationReport {
            placeholders: Vec::new(),
            removed_missing_active_tabs,
        }
    }

    pub fn apply(
        &self,
        layout: &mut WorkbenchLayout,
        cmd: LayoutCommand,
    ) -> Result<LayoutDiff, String> {
        match cmd {
            LayoutCommand::OpenView {
                instance_id,
                target,
            }
            | LayoutCommand::MoveView {
                instance_id,
                target,
            } => {
                self.detach_instance(layout, &instance_id);
                self.attach_instance(layout, instance_id, target, None)?;
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::AttachView {
                instance_id,
                target,
                anchor,
            } => {
                self.detach_instance(layout, &instance_id);
                self.attach_instance(layout, instance_id, target, anchor)?;
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::CloseView { instance_id } => Ok(LayoutDiff {
                changed: self.detach_instance(layout, &instance_id),
            }),
            LayoutCommand::FocusView { instance_id } => Ok(LayoutDiff {
                changed: self.focus_instance(layout, &instance_id),
            }),
            LayoutCommand::DetachViewToWindow {
                instance_id,
                new_window,
            } => {
                self.detach_instance(layout, &instance_id);
                layout.floating_windows.push(FloatingWindowLayout {
                    window_id: new_window.clone(),
                    title: format!("Window {}", new_window.0),
                    workspace: DocumentNode::Tabs(TabStackLayout {
                        tabs: vec![instance_id.clone()],
                        active_tab: Some(instance_id.clone()),
                    }),
                    focused_view: Some(instance_id),
                });
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::CreateSplit {
                workspace,
                path,
                axis,
                placement,
                new_instance,
            } => {
                self.detach_instance(layout, &new_instance);
                let node = self
                    .workspace_node_mut(layout, &workspace, &path)
                    .ok_or_else(|| format!("missing workspace path for {:?}", workspace))?;
                let previous = node.clone();
                let inserted = DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![new_instance.clone()],
                    active_tab: Some(new_instance),
                });
                let (first, second) = match placement {
                    SplitPlacement::Before => (inserted, previous),
                    SplitPlacement::After => (previous, inserted),
                };
                *node = DocumentNode::SplitNode {
                    axis,
                    ratio: 0.5,
                    first: Box::new(first),
                    second: Box::new(second),
                };
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::ResizeSplit {
                workspace,
                path,
                ratio,
            } => {
                let node = self
                    .workspace_node_mut(layout, &workspace, &path)
                    .ok_or_else(|| format!("missing split path for {:?}", workspace))?;
                let DocumentNode::SplitNode {
                    ratio: current_ratio,
                    ..
                } = node
                else {
                    return Err("target path is not a split node".to_string());
                };
                *current_ratio = ratio.clamp(0.1, 0.9);
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::SetDrawerMode { slot, mode } => {
                let drawer = layout
                    .drawers
                    .get_mut(&slot)
                    .ok_or_else(|| format!("missing drawer {:?}", slot))?;
                drawer.mode = mode;
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::SetDrawerExtent { slot, extent } => {
                let drawer = layout
                    .drawers
                    .get_mut(&slot)
                    .ok_or_else(|| format!("missing drawer {:?}", slot))?;
                drawer.extent = extent.max(120.0);
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::ActivateDrawerTab { slot, instance_id } => {
                let drawer = layout
                    .drawers
                    .get_mut(&slot)
                    .ok_or_else(|| format!("missing drawer {:?}", slot))?;
                if drawer.tab_stack.tabs.contains(&instance_id) {
                    drawer.tab_stack.active_tab = Some(instance_id.clone());
                    drawer.active_view = Some(instance_id);
                    Ok(LayoutDiff { changed: true })
                } else {
                    Err("drawer does not contain target tab".to_string())
                }
            }
            LayoutCommand::ActivateMainPage { page_id } => {
                layout.active_main_page = page_id;
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::SavePreset { .. } | LayoutCommand::LoadPreset { .. } => {
                Ok(LayoutDiff { changed: false })
            }
            LayoutCommand::ResetToDefault => {
                *layout = self.default_layout();
                Ok(LayoutDiff { changed: true })
            }
        }
    }

    fn attach_instance(
        &self,
        layout: &mut WorkbenchLayout,
        instance_id: ViewInstanceId,
        target: ViewHost,
        anchor: Option<TabInsertionAnchor>,
    ) -> Result<(), String> {
        match target {
            ViewHost::Drawer(slot) => {
                let drawer = layout
                    .drawers
                    .get_mut(&slot)
                    .ok_or_else(|| format!("missing drawer {:?}", slot))?;
                drawer.tab_stack.insert(instance_id.clone(), anchor.as_ref());
                drawer.active_view = Some(instance_id);
            }
            ViewHost::Document(page_id, path) => {
                let node = self
                    .document_node_mut(layout, &page_id, &path)
                    .ok_or_else(|| format!("missing document node on page {}", page_id.0))?;
                match node {
                    DocumentNode::Tabs(stack) => stack.insert(instance_id, anchor.as_ref()),
                    DocumentNode::SplitNode { .. } => {
                        return Err("cannot attach directly to split node".to_string())
                    }
                }
            }
            ViewHost::FloatingWindow(window_id, path) => {
                let window = layout
                    .floating_windows
                    .iter_mut()
                    .find(|window| window.window_id == window_id)
                    .ok_or_else(|| format!("missing floating window {}", window_id.0))?;
                let node = window
                    .workspace
                    .node_at_path_mut(&path)
                    .ok_or_else(|| format!("missing floating window node {}", window_id.0))?;
                match node {
                    DocumentNode::Tabs(stack) => {
                        stack.insert(instance_id.clone(), anchor.as_ref());
                        window.focused_view = Some(instance_id);
                    }
                    DocumentNode::SplitNode { .. } => {
                        return Err("cannot attach directly to split node".to_string())
                    }
                }
            }
            ViewHost::ExclusivePage(page_id) => {
                layout
                    .main_pages
                    .push(MainHostPageLayout::ExclusiveActivityWindowPage {
                        id: page_id.clone(),
                        title: page_id.0.clone(),
                        window_instance: instance_id,
                    });
                layout.active_main_page = page_id;
            }
        }

        Ok(())
    }

    fn detach_instance(&self, layout: &mut WorkbenchLayout, instance_id: &ViewInstanceId) -> bool {
        let mut changed = false;

        for drawer in layout.drawers.values_mut() {
            changed |= drawer.tab_stack.remove(instance_id);
            if drawer.active_view.as_ref() == Some(instance_id) {
                drawer.active_view = drawer.tab_stack.active_tab.clone();
            }
        }

        for page in &mut layout.main_pages {
            if let Some(workspace) = page.document_workspace_mut() {
                changed |= workspace.remove_instance(instance_id);
            }
        }

        for window in &mut layout.floating_windows {
            changed |= window.workspace.remove_instance(instance_id);
            if window.focused_view.as_ref() == Some(instance_id) {
                window.focused_view = None;
            }
        }

        layout.main_pages.retain(|page| match page {
            MainHostPageLayout::WorkbenchPage { .. } => true,
            MainHostPageLayout::ExclusiveActivityWindowPage {
                window_instance, ..
            } => window_instance != instance_id,
        });
        layout
            .floating_windows
            .retain(|window| match &window.workspace {
                DocumentNode::Tabs(stack) => !stack.tabs.is_empty(),
                DocumentNode::SplitNode { .. } => true,
            });

        changed
    }

    fn focus_instance(&self, layout: &mut WorkbenchLayout, instance_id: &ViewInstanceId) -> bool {
        for drawer in layout.drawers.values_mut() {
            if drawer.tab_stack.tabs.contains(instance_id) {
                drawer.tab_stack.active_tab = Some(instance_id.clone());
                drawer.active_view = Some(instance_id.clone());
                return true;
            }
        }

        for page in &mut layout.main_pages {
            if let Some(workspace) = page.document_workspace_mut() {
                if Self::focus_in_document_node(workspace, instance_id) {
                    layout.active_main_page = page.id().clone();
                    return true;
                }
            } else if let MainHostPageLayout::ExclusiveActivityWindowPage {
                id,
                window_instance,
                ..
            } = page
            {
                if window_instance == instance_id {
                    layout.active_main_page = id.clone();
                    return true;
                }
            }
        }

        for window in &mut layout.floating_windows {
            if window.workspace.contains(instance_id) {
                window.focused_view = Some(instance_id.clone());
                return true;
            }
        }

        false
    }

    fn focus_in_document_node(node: &mut DocumentNode, instance_id: &ViewInstanceId) -> bool {
        match node {
            DocumentNode::Tabs(stack) => {
                if stack.tabs.contains(instance_id) {
                    stack.active_tab = Some(instance_id.clone());
                    true
                } else {
                    false
                }
            }
            DocumentNode::SplitNode { first, second, .. } => {
                Self::focus_in_document_node(first, instance_id)
                    || Self::focus_in_document_node(second, instance_id)
            }
        }
    }

    fn document_node_mut<'a>(
        &self,
        layout: &'a mut WorkbenchLayout,
        page_id: &MainPageId,
        path: &[usize],
    ) -> Option<&'a mut DocumentNode> {
        let page = layout
            .main_pages
            .iter_mut()
            .find(|page| page.id() == page_id)?;
        let workspace = page.document_workspace_mut()?;
        workspace.node_at_path_mut(path)
    }

    fn workspace_node_mut<'a>(
        &self,
        layout: &'a mut WorkbenchLayout,
        workspace: &WorkspaceTarget,
        path: &[usize],
    ) -> Option<&'a mut DocumentNode> {
        match workspace {
            WorkspaceTarget::MainPage(page_id) => self.document_node_mut(layout, page_id, path),
            WorkspaceTarget::FloatingWindow(window_id) => layout
                .floating_windows
                .iter_mut()
                .find(|window| &window.window_id == window_id)
                .and_then(|window| window.workspace.node_at_path_mut(path)),
        }
    }
}

fn edge_axis(edge: DockEdge) -> SplitAxis {
    match edge {
        DockEdge::Left | DockEdge::Right => SplitAxis::Horizontal,
        DockEdge::Top | DockEdge::Bottom => SplitAxis::Vertical,
    }
}

fn edge_placement(edge: DockEdge) -> SplitPlacement {
    match edge {
        DockEdge::Left | DockEdge::Top => SplitPlacement::Before,
        DockEdge::Right | DockEdge::Bottom => SplitPlacement::After,
    }
}
