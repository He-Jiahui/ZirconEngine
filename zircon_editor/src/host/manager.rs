//! Editor manager service and window host bookkeeping.

mod startup;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use zircon_core::CoreHandle;
use zircon_manager::{AssetManager, ConfigManager, ManagerResolver};
use zircon_scene::{DefaultLevelManager, LevelMetadata, Scene, DEFAULT_LEVEL_MANAGER_NAME};

use crate::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, DocumentNode, LayoutCommand,
    LayoutManager, MainHostPageLayout, MainPageId, RestorePolicy, TabStackLayout, WorkbenchLayout,
};
use crate::project::{
    list_layout_preset_assets, load_layout_preset_asset, project_root_path,
    save_layout_preset_asset, EditorProjectDocument, ProjectEditorWorkspace,
};
use crate::view::{
    DockPolicy, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance,
    ViewInstanceId, ViewKind, ViewRegistry,
};
use crate::default_constraints_for_content;
use crate::ViewContentKind;

const DEFAULT_LAYOUT_KEY: &str = "editor.workbench.default_layout";
const PRESET_LAYOUTS_KEY: &str = "editor.workbench.presets";
type NativeWindowHandle = u64;

#[derive(Debug)]
pub enum EditorError {
    Layout(String),
    Registry(String),
    Project(String),
}

impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Layout(error) | Self::Registry(error) | Self::Project(error) => {
                f.write_str(error)
            }
        }
    }
}

impl std::error::Error for EditorError {}

#[derive(Clone, Debug, Default)]
pub struct WindowHostManager {
    windows: BTreeMap<MainPageId, Option<NativeWindowHandle>>,
}

impl WindowHostManager {
    pub fn open_native_window(&mut self, window_id: MainPageId, handle: Option<NativeWindowHandle>) {
        self.windows.insert(window_id, handle);
    }

    pub fn close_native_window(&mut self, window_id: &MainPageId) {
        self.windows.remove(window_id);
    }

    pub fn sync_window_bounds(&mut self, _window_id: &MainPageId, _bounds: [f32; 4]) {}

    pub fn reattach_window(&mut self, window_id: &MainPageId, _drop_target: &ViewHost) {
        self.close_native_window(window_id);
    }
}

#[derive(Clone, Debug)]
pub struct EditorSessionState {
    pub layout: WorkbenchLayout,
    pub open_view_instances: BTreeMap<ViewInstanceId, ViewInstance>,
    pub active_center_tab: Option<ViewInstanceId>,
    pub active_drawers: Vec<ActivityDrawerSlot>,
}

impl Default for EditorSessionState {
    fn default() -> Self {
        Self {
            layout: WorkbenchLayout::default(),
            open_view_instances: BTreeMap::new(),
            active_center_tab: None,
            active_drawers: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct EditorManager {
    core: CoreHandle,
    view_registry: Mutex<ViewRegistry>,
    layout_manager: LayoutManager,
    window_host_manager: Mutex<WindowHostManager>,
    session: Mutex<EditorSessionState>,
}

impl EditorManager {
    pub fn new(core: CoreHandle) -> Self {
        let manager = Self {
            core,
            view_registry: Mutex::new(ViewRegistry::default()),
            layout_manager: LayoutManager,
            window_host_manager: Mutex::new(WindowHostManager::default()),
            session: Mutex::new(EditorSessionState::default()),
        };
        manager
            .register_builtin_views()
            .expect("builtin editor views");
        manager
            .bootstrap_default_layout()
            .expect("default workbench");
        manager
    }

    pub fn register_builtin_views(&self) -> Result<(), EditorError> {
        let mut registry = self.view_registry.lock().unwrap();
        for descriptor in builtin_view_descriptors() {
            if registry.descriptor(&descriptor.descriptor_id).is_none() {
                registry
                    .register_view(descriptor)
                    .map_err(EditorError::Registry)?;
            }
        }
        Ok(())
    }

    pub fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorProjectDocument, EditorError> {
        let root =
            project_root_path(&path).map_err(|error| EditorError::Project(error.to_string()))?;
        self.asset_manager()?
            .open_project(root.to_string_lossy().as_ref())
            .map_err(|error| EditorError::Project(error.to_string()))?;
        EditorProjectDocument::load_from_path(&path)
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub fn save_project(
        &self,
        path: impl AsRef<Path>,
        world: &zircon_scene::Scene,
    ) -> Result<(), EditorError> {
        let workspace = self.project_workspace();
        let root =
            project_root_path(&path).map_err(|error| EditorError::Project(error.to_string()))?;
        EditorProjectDocument::save_to_path(&path, world, Some(&workspace))
            .map_err(|error| EditorError::Project(error.to_string()))?;
        self.asset_manager()?
            .open_project(root.to_string_lossy().as_ref())
            .map(|_| ())
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub fn create_runtime_level(
        &self,
        scene: Scene,
    ) -> Result<zircon_scene::LevelSystem, EditorError> {
        let manager = self
            .core
            .resolve_manager::<DefaultLevelManager>(DEFAULT_LEVEL_MANAGER_NAME)
            .map_err(|error| EditorError::Project(error.to_string()))?;
        Ok(manager.create_level(scene, LevelMetadata::default()))
    }

    pub fn current_layout(&self) -> WorkbenchLayout {
        self.session.lock().unwrap().layout.clone()
    }

    pub fn current_view_instances(&self) -> Vec<ViewInstance> {
        self.session
            .lock()
            .unwrap()
            .open_view_instances
            .values()
            .cloned()
            .collect()
    }

    pub fn descriptors(&self) -> Vec<ViewDescriptor> {
        self.view_registry.lock().unwrap().list_descriptors()
    }

    pub fn apply_layout_command(&self, cmd: LayoutCommand) -> Result<bool, EditorError> {
        match cmd {
            LayoutCommand::SavePreset { name } => {
                self.save_preset(&name)?;
                return Ok(false);
            }
            LayoutCommand::LoadPreset { name } => {
                return self.load_preset(&name);
            }
            LayoutCommand::ResetToDefault => {
                let mut session = self.session.lock().unwrap();
                let mut registry = self.view_registry.lock().unwrap();
                ensure_builtin_shell_instances(&mut registry, &mut session)?;
                session.layout = self.layout_manager.default_layout();
                self.recompute_session_metadata(&mut session);
                return Ok(true);
            }
            _ => {}
        }

        let mut session = self.session.lock().unwrap();
        let diff = self
            .layout_manager
            .apply(&mut session.layout, cmd)
            .map_err(EditorError::Layout)?;
        self.recompute_session_metadata(&mut session);
        Ok(diff.changed)
    }

    pub fn open_view(
        &self,
        descriptor_id: ViewDescriptorId,
        target_host: Option<ViewHost>,
    ) -> Result<ViewInstanceId, EditorError> {
        let mut registry = self.view_registry.lock().unwrap();
        let instance = registry
            .open_descriptor(descriptor_id)
            .map_err(EditorError::Registry)?;
        drop(registry);

        let target = target_host.unwrap_or_else(|| instance.host.clone());
        self.attach_instance(instance, target)
    }

    pub fn close_view(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        if self.non_closeable_instance(instance_id) {
            return Ok(false);
        }
        let changed = self.apply_layout_command(LayoutCommand::CloseView {
            instance_id: instance_id.clone(),
        })?;
        if changed {
            self.session
                .lock()
                .unwrap()
                .open_view_instances
                .remove(instance_id);
            self.view_registry
                .lock()
                .unwrap()
                .remove_instance(instance_id);
        }
        Ok(changed)
    }

    pub fn focus_view(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.apply_layout_command(LayoutCommand::FocusView {
            instance_id: instance_id.clone(),
        })
    }

    pub fn detach_view_to_window(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        let window_id = MainPageId::new(format!("window:{}", instance_id.0));
        let changed = self.apply_layout_command(LayoutCommand::DetachViewToWindow {
            instance_id: instance_id.clone(),
            new_window: window_id.clone(),
        })?;
        if changed {
            self.window_host_manager
                .lock()
                .unwrap()
                .open_native_window(window_id, None);
        }
        Ok(changed)
    }

    pub fn attach_view_to_target(
        &self,
        instance_id: &ViewInstanceId,
        drop_target: ViewHost,
    ) -> Result<bool, EditorError> {
        self.apply_layout_command(LayoutCommand::AttachView {
            instance_id: instance_id.clone(),
            target: drop_target,
            anchor: None,
        })
    }

    pub fn restore_workspace(&self, policy: RestorePolicy) -> Result<WorkbenchLayout, EditorError> {
        let global = self.load_global_default_layout();
        let workspace = self.project_workspace();
        let restored = self
            .layout_manager
            .restore_workspace(policy, Some(workspace), global)
            .map_err(EditorError::Layout)?;
        let mut session = self.session.lock().unwrap();
        session.layout = restored.clone();
        self.recompute_session_metadata(&mut session);
        Ok(restored)
    }

    pub fn save_global_default_layout(&self) -> Result<(), EditorError> {
        let layout = self.current_layout();
        let config = self.config_manager()?;
        config
            .set_value(
                DEFAULT_LAYOUT_KEY,
                serde_json::to_value(layout)
                    .map_err(|error| EditorError::Project(error.to_string()))?,
            )
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub fn apply_project_workspace(
        &self,
        workspace: Option<ProjectEditorWorkspace>,
    ) -> Result<(), EditorError> {
        if workspace.is_none() {
            return self.bootstrap_default_layout();
        }

        let mut session = self.session.lock().unwrap();
        let mut registry = self.view_registry.lock().unwrap();
        registry.clear_instances();

        let workspace = workspace.expect("checked above");
        session.layout = workspace.workbench;
        session.open_view_instances.clear();
        for instance in workspace.open_view_instances {
            let restored = registry
                .restore_instance(instance)
                .map_err(EditorError::Registry)?;
            session
                .open_view_instances
                .insert(restored.instance_id.clone(), restored);
        }
        session.active_center_tab = workspace.active_center_tab;
        session.active_drawers = workspace.active_drawers;
        self.layout_manager
            .normalize(&mut session.layout, &registry);
        self.recompute_session_metadata(&mut session);
        Ok(())
    }

    pub fn project_workspace(&self) -> ProjectEditorWorkspace {
        let session = self.session.lock().unwrap();
        ProjectEditorWorkspace {
            layout_version: 1,
            workbench: session.layout.clone(),
            open_view_instances: session.open_view_instances.values().cloned().collect(),
            active_center_tab: session.active_center_tab.clone(),
            active_drawers: session.active_drawers.clone(),
        }
    }

    fn attach_instance(
        &self,
        instance: ViewInstance,
        target: ViewHost,
    ) -> Result<ViewInstanceId, EditorError> {
        {
            let mut session = self.session.lock().unwrap();
            session
                .open_view_instances
                .insert(instance.instance_id.clone(), instance.clone());
        }
        self.apply_layout_command(LayoutCommand::AttachView {
            instance_id: instance.instance_id.clone(),
            target,
            anchor: None,
        })?;
        Ok(instance.instance_id)
    }

    fn bootstrap_default_layout(&self) -> Result<(), EditorError> {
        let mut registry = self.view_registry.lock().unwrap();
        registry.clear_instances();
        let mut session = EditorSessionState::default();
        ensure_builtin_shell_instances(&mut registry, &mut session)?;
        session.layout = self.layout_manager.default_layout();
        self.layout_manager
            .normalize(&mut session.layout, &registry);
        *self.session.lock().unwrap() = session;

        if let Some(layout) = self.load_global_default_layout() {
            let mut session = self.session.lock().unwrap();
            session.layout = layout;
            repair_builtin_shell_layout(&mut session.layout);
            self.layout_manager
                .normalize(&mut session.layout, &registry);
            self.recompute_session_metadata(&mut session);
        } else {
            let mut session = self.session.lock().unwrap();
            self.recompute_session_metadata(&mut session);
        }
        Ok(())
    }

    fn recompute_session_metadata(&self, session: &mut EditorSessionState) {
        let placements = collect_instance_hosts(&session.layout);
        session
            .open_view_instances
            .retain(|instance_id, _| placements.contains_key(instance_id));
        for (instance_id, host) in placements {
            if let Some(instance) = session.open_view_instances.get_mut(&instance_id) {
                instance.host = host;
            }
        }

        session.active_drawers = session
            .layout
            .drawers
            .iter()
            .filter_map(|(slot, drawer)| drawer.visible.then_some(*slot))
            .collect();
        session.active_center_tab = session
            .layout
            .main_pages
            .iter()
            .find(|page| page.id() == &session.layout.active_main_page)
            .and_then(|page| match page {
                MainHostPageLayout::WorkbenchPage {
                    document_workspace, ..
                } => active_tab_from_document(document_workspace),
                MainHostPageLayout::ExclusiveActivityWindowPage {
                    window_instance, ..
                } => Some(window_instance.clone()),
            });
    }

    fn config_manager(&self) -> Result<std::sync::Arc<dyn ConfigManager>, EditorError> {
        ManagerResolver::new(self.core.clone())
            .config()
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    fn asset_manager(&self) -> Result<std::sync::Arc<dyn AssetManager>, EditorError> {
        ManagerResolver::new(self.core.clone())
            .asset()
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    fn load_global_default_layout(&self) -> Option<WorkbenchLayout> {
        let config = self.config_manager().ok()?;
        let value = config.get_value(DEFAULT_LAYOUT_KEY)?;
        serde_json::from_value(value).ok()
    }

    fn save_preset(&self, name: &str) -> Result<(), EditorError> {
        if let Some(project_root) = self.current_project_root()? {
            save_layout_preset_asset(&project_root, name, &self.current_layout())
                .map_err(|error| EditorError::Project(error.to_string()))?;
            return Ok(());
        }
        let mut presets = self.load_presets()?;
        presets.insert(name.to_string(), self.current_layout());
        self.config_manager()?
            .set_value(
                PRESET_LAYOUTS_KEY,
                serde_json::to_value(presets)
                    .map_err(|error| EditorError::Project(error.to_string()))?,
            )
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    fn load_preset(&self, name: &str) -> Result<bool, EditorError> {
        if let Some(project_root) = self.current_project_root()? {
            if let Some(layout) = load_layout_preset_asset(&project_root, name)
                .map_err(|error| EditorError::Project(error.to_string()))?
            {
                let mut session = self.session.lock().unwrap();
                session.layout = layout;
                self.recompute_session_metadata(&mut session);
                return Ok(true);
            }
        }
        let presets = self.load_presets()?;
        let layout = presets
            .get(name)
            .cloned()
            .ok_or_else(|| EditorError::Layout(format!("missing preset {name}")))?;
        let mut session = self.session.lock().unwrap();
        session.layout = layout;
        self.recompute_session_metadata(&mut session);
        Ok(true)
    }

    fn load_presets(&self) -> Result<BTreeMap<String, WorkbenchLayout>, EditorError> {
        let Some(value) = self.config_manager()?.get_value(PRESET_LAYOUTS_KEY) else {
            return Ok(BTreeMap::new());
        };
        serde_json::from_value(value).map_err(|error| EditorError::Project(error.to_string()))
    }

    pub fn preset_names(&self) -> Result<Vec<String>, EditorError> {
        let mut names = Vec::new();
        if let Some(project_root) = self.current_project_root()? {
            names.extend(
                list_layout_preset_assets(&project_root)
                    .map_err(|error| EditorError::Project(error.to_string()))?,
            );
        }
        names.extend(self.load_presets()?.into_keys());
        names.sort();
        names.dedup();
        Ok(names)
    }

    fn non_closeable_instance(&self, instance_id: &ViewInstanceId) -> bool {
        self.session
            .lock()
            .unwrap()
            .open_view_instances
            .get(instance_id)
            .is_some_and(|instance| {
                matches!(
                    instance.descriptor_id.0.as_str(),
                    "editor.scene" | "editor.game"
                )
            })
    }

    fn current_project_root(&self) -> Result<Option<PathBuf>, EditorError> {
        let Some(project) = self.asset_manager()?.current_project() else {
            return Ok(None);
        };
        Ok(Some(PathBuf::from(project.root_path)))
    }
}

fn active_tab_from_document(node: &crate::layout::DocumentNode) -> Option<ViewInstanceId> {
    match node {
        crate::layout::DocumentNode::Tabs(stack) => stack.active_tab.clone(),
        crate::layout::DocumentNode::SplitNode { first, second, .. } => {
            active_tab_from_document(first).or_else(|| active_tab_from_document(second))
        }
    }
}

fn collect_instance_hosts(layout: &WorkbenchLayout) -> BTreeMap<ViewInstanceId, ViewHost> {
    let mut placements = BTreeMap::new();

    for (slot, drawer) in &layout.drawers {
        for instance_id in &drawer.tab_stack.tabs {
            placements.insert(instance_id.clone(), ViewHost::Drawer(*slot));
        }
    }

    for page in &layout.main_pages {
        match page {
            MainHostPageLayout::WorkbenchPage {
                id,
                document_workspace,
                ..
            } => collect_document_hosts(document_workspace, &mut placements, |path| {
                ViewHost::Document(id.clone(), path)
            }),
            MainHostPageLayout::ExclusiveActivityWindowPage {
                id,
                window_instance,
                ..
            } => {
                placements.insert(window_instance.clone(), ViewHost::ExclusivePage(id.clone()));
            }
        }
    }

    for window in &layout.floating_windows {
        collect_document_hosts(&window.workspace, &mut placements, |path| {
            ViewHost::FloatingWindow(window.window_id.clone(), path)
        });
    }

    placements
}

fn collect_document_hosts(
    node: &crate::layout::DocumentNode,
    placements: &mut BTreeMap<ViewInstanceId, ViewHost>,
    make_host: impl Fn(Vec<usize>) -> ViewHost + Copy,
) {
    fn visit(
        node: &crate::layout::DocumentNode,
        path: &mut Vec<usize>,
        placements: &mut BTreeMap<ViewInstanceId, ViewHost>,
        make_host: impl Fn(Vec<usize>) -> ViewHost + Copy,
    ) {
        match node {
            crate::layout::DocumentNode::Tabs(stack) => {
                let host = make_host(path.clone());
                for instance_id in &stack.tabs {
                    placements.insert(instance_id.clone(), host.clone());
                }
            }
            crate::layout::DocumentNode::SplitNode { first, second, .. } => {
                path.push(0);
                visit(first, path, placements, make_host);
                path.pop();
                path.push(1);
                visit(second, path, placements, make_host);
                path.pop();
            }
        }
    }

    let mut path = Vec::new();
    visit(node, &mut path, placements, make_host);
}

fn repair_builtin_shell_layout(layout: &mut WorkbenchLayout) {
    let baseline = builtin_hybrid_layout();
    let mut present: BTreeSet<ViewInstanceId> = collect_instance_hosts(layout).into_keys().collect();

    for (slot, baseline_drawer) in &baseline.drawers {
        let target_drawer = layout
            .drawers
            .entry(*slot)
            .or_insert_with(|| ActivityDrawerLayout::new(*slot));

        for instance_id in &baseline_drawer.tab_stack.tabs {
            if present.insert(instance_id.clone()) {
                target_drawer.tab_stack.tabs.push(instance_id.clone());
            }
        }

        if target_drawer
            .tab_stack
            .active_tab
            .as_ref()
            .is_none_or(|active| !target_drawer.tab_stack.tabs.contains(active))
        {
            target_drawer.tab_stack.active_tab = baseline_drawer
                .tab_stack
                .active_tab
                .clone()
                .filter(|active| target_drawer.tab_stack.tabs.contains(active))
                .or_else(|| target_drawer.tab_stack.tabs.first().cloned());
        }

        if target_drawer
            .active_view
            .as_ref()
            .is_none_or(|active| !target_drawer.tab_stack.tabs.contains(active))
        {
            target_drawer.active_view = target_drawer.tab_stack.active_tab.clone();
        }
    }

    let Some(baseline_stack) = baseline_main_page_tabs(&baseline) else {
        return;
    };
    let stack = first_tab_stack_mut(ensure_workbench_document_root(layout));
    for instance_id in baseline_stack.tabs {
        if present.insert(instance_id.clone()) {
            stack.tabs.push(instance_id);
        }
    }
    if stack
        .active_tab
        .as_ref()
        .is_none_or(|active| !stack.tabs.contains(active))
    {
        stack.active_tab = baseline_stack
            .active_tab
            .filter(|active| stack.tabs.contains(active))
            .or_else(|| stack.tabs.first().cloned());
    }
}

fn baseline_main_page_tabs(layout: &WorkbenchLayout) -> Option<TabStackLayout> {
    layout.main_pages.iter().find_map(|page| match page {
        MainHostPageLayout::WorkbenchPage {
            document_workspace, ..
        } => first_tab_stack(document_workspace).cloned(),
        MainHostPageLayout::ExclusiveActivityWindowPage { .. } => None,
    })
}

fn first_tab_stack(node: &DocumentNode) -> Option<&TabStackLayout> {
    match node {
        DocumentNode::Tabs(stack) => Some(stack),
        DocumentNode::SplitNode { first, second, .. } => {
            first_tab_stack(first).or_else(|| first_tab_stack(second))
        }
    }
}

fn first_tab_stack_mut(node: &mut DocumentNode) -> &mut TabStackLayout {
    match node {
        DocumentNode::Tabs(stack) => stack,
        DocumentNode::SplitNode { first, second, .. } => {
            if let Some(stack) = first_tab_stack(first) {
                let _ = stack;
                first_tab_stack_mut(first)
            } else {
                first_tab_stack_mut(second)
            }
        }
    }
}

fn ensure_workbench_document_root(layout: &mut WorkbenchLayout) -> &mut DocumentNode {
    if let Some(index) = layout
        .main_pages
        .iter()
        .position(|page| matches!(page, MainHostPageLayout::WorkbenchPage { .. }))
    {
        match &mut layout.main_pages[index] {
            MainHostPageLayout::WorkbenchPage {
                document_workspace, ..
            } => document_workspace,
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => unreachable!(),
        }
    } else {
        layout.main_pages.insert(
            0,
            MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                document_workspace: DocumentNode::default(),
            },
        );
        match &mut layout.main_pages[0] {
            MainHostPageLayout::WorkbenchPage {
                document_workspace, ..
            } => document_workspace,
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => unreachable!(),
        }
    }
}

fn builtin_view_descriptors() -> Vec<ViewDescriptor> {
    vec![
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.project"),
            ViewKind::ActivityView,
            "Project",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Project))
        .with_icon_key("project"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.hierarchy"),
            ViewKind::ActivityView,
            "Hierarchy",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Hierarchy))
        .with_icon_key("hierarchy"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.inspector"),
            ViewKind::ActivityView,
            "Inspector",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::RightTop)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Inspector))
        .with_icon_key("inspector"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.scene"),
            ViewKind::ActivityView,
            "Scene",
        )
        .with_dock_policy(DockPolicy::DrawerOrDocument)
        .with_preferred_host(PreferredHost::DocumentCenter)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Scene))
        .with_icon_key("scene"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.game"),
            ViewKind::ActivityView,
            "Game",
        )
        .with_dock_policy(DockPolicy::DrawerOrDocument)
        .with_preferred_host(PreferredHost::DocumentCenter)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Game))
        .with_icon_key("game"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.assets"),
            ViewKind::ActivityView,
            "Assets",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Assets))
        .with_icon_key("assets"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.console"),
            ViewKind::ActivityView,
            "Console",
        )
        .with_preferred_drawer_slot(ActivityDrawerSlot::BottomLeft)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::Console))
        .with_icon_key("console"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.prefab"),
            ViewKind::ActivityWindow,
            "Prefab Editor",
        )
        .with_multi_instance(true)
        .with_preferred_host(PreferredHost::DocumentCenter)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::PrefabEditor))
        .with_icon_key("prefab"),
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.asset_browser"),
            ViewKind::ActivityWindow,
            "Asset Browser",
        )
        .with_preferred_host(PreferredHost::ExclusiveMainPage)
        .with_default_constraints(default_constraints_for_content(ViewContentKind::AssetBrowser))
        .with_icon_key("asset-browser"),
        startup::welcome_view_descriptor(),
    ]
}

fn ensure_builtin_shell_instances(
    registry: &mut ViewRegistry,
    session: &mut EditorSessionState,
) -> Result<(), EditorError> {
    for instance in builtin_shell_view_instances() {
        let restored = if let Some(existing) = registry.instance(&instance.instance_id).cloned() {
            existing
        } else {
            registry
                .restore_instance(instance.clone())
                .map_err(EditorError::Registry)?
        };
        session
            .open_view_instances
            .insert(restored.instance_id.clone(), restored);
    }
    Ok(())
}

fn builtin_shell_view_instances() -> Vec<ViewInstance> {
    vec![
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.project#1"),
            descriptor_id: ViewDescriptorId::new("editor.project"),
            title: "Project".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.assets#1"),
            descriptor_id: ViewDescriptorId::new("editor.assets"),
            title: "Assets".to_string(),
            serializable_payload: serde_json::json!({ "root": "crate://" }),
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.hierarchy#1"),
            descriptor_id: ViewDescriptorId::new("editor.hierarchy"),
            title: "Hierarchy".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.inspector#1"),
            descriptor_id: ViewDescriptorId::new("editor.inspector"),
            title: "Inspector".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.console#1"),
            descriptor_id: ViewDescriptorId::new("editor.console"),
            title: "Console".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::BottomLeft),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.game#1"),
            descriptor_id: ViewDescriptorId::new("editor.game"),
            title: "Game".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Document(MainPageId::workbench(), vec![]),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.scene#1"),
            descriptor_id: ViewDescriptorId::new("editor.scene"),
            title: "Scene".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Document(MainPageId::workbench(), vec![]),
        },
    ]
}

pub(crate) fn builtin_hybrid_layout() -> WorkbenchLayout {
    WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![MainHostPageLayout::WorkbenchPage {
            id: MainPageId::workbench(),
            title: "Workbench".to_string(),
            document_workspace: DocumentNode::Tabs(TabStackLayout {
                tabs: vec![
                    ViewInstanceId::new("editor.scene#1"),
                    ViewInstanceId::new("editor.game#1"),
                ],
                active_tab: Some(ViewInstanceId::new("editor.scene#1")),
            }),
        }],
        drawers: BTreeMap::from([
            (
                ActivityDrawerSlot::LeftTop,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::LeftTop,
                    tab_stack: TabStackLayout {
                        tabs: vec![
                            ViewInstanceId::new("editor.project#1"),
                            ViewInstanceId::new("editor.assets#1"),
                            ViewInstanceId::new("editor.hierarchy#1"),
                        ],
                        active_tab: Some(ViewInstanceId::new("editor.project#1")),
                    },
                    active_view: Some(ViewInstanceId::new("editor.project#1")),
                    mode: ActivityDrawerMode::Pinned,
                    extent: 312.0,
                    visible: true,
                },
            ),
            (
                ActivityDrawerSlot::LeftBottom,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::LeftBottom,
                    tab_stack: TabStackLayout::default(),
                    active_view: None,
                    mode: ActivityDrawerMode::Collapsed,
                    extent: 288.0,
                    visible: true,
                },
            ),
            (
                ActivityDrawerSlot::RightTop,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::RightTop,
                    tab_stack: TabStackLayout {
                        tabs: vec![ViewInstanceId::new("editor.inspector#1")],
                        active_tab: Some(ViewInstanceId::new("editor.inspector#1")),
                    },
                    active_view: Some(ViewInstanceId::new("editor.inspector#1")),
                    mode: ActivityDrawerMode::Pinned,
                    extent: 308.0,
                    visible: true,
                },
            ),
            (
                ActivityDrawerSlot::RightBottom,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::RightBottom,
                    tab_stack: TabStackLayout::default(),
                    active_view: None,
                    mode: ActivityDrawerMode::Collapsed,
                    extent: 288.0,
                    visible: true,
                },
            ),
            (
                ActivityDrawerSlot::BottomLeft,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::BottomLeft,
                    tab_stack: TabStackLayout {
                        tabs: vec![ViewInstanceId::new("editor.console#1")],
                        active_tab: Some(ViewInstanceId::new("editor.console#1")),
                    },
                    active_view: Some(ViewInstanceId::new("editor.console#1")),
                    mode: ActivityDrawerMode::Pinned,
                    extent: 164.0,
                    visible: true,
                },
            ),
            (
                ActivityDrawerSlot::BottomRight,
                ActivityDrawerLayout {
                    slot: ActivityDrawerSlot::BottomRight,
                    tab_stack: TabStackLayout::default(),
                    active_view: None,
                    mode: ActivityDrawerMode::Collapsed,
                    extent: 224.0,
                    visible: true,
                },
            ),
        ]),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    }
}
