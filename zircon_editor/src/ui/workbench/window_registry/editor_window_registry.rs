use std::collections::{BTreeMap, HashMap};

use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityWindowHostMode, ActivityWindowId, ActivityWindowLayout,
    FloatingWindowLayout, MainPageId, WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstance, ViewInstanceId};

use super::{
    DrawerBinding, DrawerDockPosition, DrawerViewInstance, DrawerWindowInstance, WindowInstance,
    WindowKind,
};

#[cfg(test)]
#[path = "editor_window_registry/tests/optimization_tests.rs"]
mod optimization_tests;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 权威布局的派生窗口索引；生产通过整轮sync重建，不应拿局部登记代替布局提交。
pub struct EditorWindowRegistry {
    active_window: Option<ActivityWindowId>,
    windows: BTreeMap<ActivityWindowId, WindowInstance>,
    drawer_views: BTreeMap<ViewInstanceId, DrawerViewInstance>,
    drawer_windows: BTreeMap<MainPageId, DrawerWindowInstance>,
}

impl EditorWindowRegistry {
    pub fn register_window(&mut self, window: WindowInstance) {
        if self.active_window.is_none() {
            self.active_window = Some(window.window_id.clone());
        }
        self.windows.insert(window.window_id.clone(), window);
    }

    // TODO: [CR-EDITOR-WORKBENCH-0008] 明确重复登记同一抽屉实例的限制；换owner或槽位时这里只覆盖实例索引，旧窗口列表与选择未清理，应使用重绑入口或拒绝迁移。
    /// 登记已存在且具抽屉能力的窗口归属；跨owner或槽迁移应使用bind_drawer保持双向索引一致。
    pub fn register_drawer_view(&mut self, drawer: DrawerViewInstance) -> Result<(), String> {
        let window = self
            .windows
            .get_mut(&drawer.owner_window)
            .ok_or_else(|| format!("missing drawer owner window {}", drawer.owner_window.0))?;
        if !window.drawer_capable() {
            return Err(format!(
                "window {} is not drawer-capable",
                drawer.owner_window.0
            ));
        }
        window
            .drawer_views
            .entry(drawer.dock_position)
            .or_default()
            .retain(|current| current != &drawer.instance_id);
        window
            .drawer_views
            .entry(drawer.dock_position)
            .or_default()
            .push(drawer.instance_id.clone());
        if window.selected_drawer.is_none() {
            window.selected_drawer = Some(drawer.instance_id.clone());
        }
        self.drawer_views.insert(drawer.instance_id.clone(), drawer);
        Ok(())
    }

    pub fn register_drawer_window(&mut self, window: DrawerWindowInstance) {
        self.drawer_windows.insert(window.window_id.clone(), window);
    }

    /// 先核验目标，再原地同步旧窗口、目标窗口和实例归属；失败不得部分解绑。
    pub fn bind_drawer(&mut self, binding: DrawerBinding) -> Result<(), String> {
        let DrawerBinding {
            window_id,
            drawer_view,
            dock_position,
        } = binding;
        let old_owner = self
            .drawer_views
            .get(&drawer_view)
            .ok_or_else(|| format!("missing drawer view {}", drawer_view.0))?
            .owner_window
            .clone();
        let target_window = self
            .windows
            .get(&window_id)
            .ok_or_else(|| format!("missing drawer owner window {}", window_id.0))?;
        if !target_window.drawer_capable() {
            return Err(format!("window {} is not drawer-capable", window_id.0));
        }

        if let Some(old_window) = self.windows.get_mut(&old_owner) {
            for views in old_window.drawer_views.values_mut() {
                views.retain(|view| view != &drawer_view);
            }
            if old_window.selected_drawer.as_ref() == Some(&drawer_view) {
                old_window.selected_drawer = None;
            }
        }

        let target_window = self
            .windows
            .get_mut(&window_id)
            .expect("drawer target window was validated before mutation");
        let target_views = target_window.drawer_views.entry(dock_position).or_default();
        target_views.retain(|view| view != &drawer_view);
        target_views.push(drawer_view.clone());
        if target_window.selected_drawer.is_none() {
            target_window.selected_drawer = Some(drawer_view.clone());
        }

        let drawer = self
            .drawer_views
            .get_mut(&drawer_view)
            .expect("drawer target was validated before mutation");
        drawer.owner_window = window_id;
        drawer.dock_position = dock_position;
        Ok(())
    }

    pub fn get_window(&self, window_id: &ActivityWindowId) -> Option<&WindowInstance> {
        self.windows.get(window_id)
    }

    pub fn get_drawer_view(&self, instance_id: &ViewInstanceId) -> Option<&DrawerViewInstance> {
        self.drawer_views.get(instance_id)
    }

    pub fn get_drawer_window(&self, window_id: &MainPageId) -> Option<&DrawerWindowInstance> {
        self.drawer_windows.get(window_id)
    }

    pub fn active_window(&self) -> Option<&WindowInstance> {
        self.active_window
            .as_ref()
            .and_then(|window_id| self.windows.get(window_id))
    }

    /// 派生索引中不存在的窗口使活动身份清空，调用方不能据此创建新布局窗口。
    pub fn activate_window(&mut self, window_id: ActivityWindowId) {
        self.active_window = self.windows.contains_key(&window_id).then_some(window_id);
    }

    pub fn selected_drawer_for_active_window(&self) -> Option<&DrawerViewInstance> {
        let window = self.active_window()?;
        let selected = window.selected_drawer.as_ref()?;
        self.drawer_views.get(selected)
    }

    /// 从已提交layout与实例表重建派生索引；折叠抽屉保留登记，只取消选中态。
    pub fn sync_from_layout(layout: &WorkbenchLayout, instances: &[ViewInstance]) -> Self {
        let mut registry = Self::default();
        let instances = instances_by_id(instances);
        let active_window = layout.active_activity_window_id();

        let activity_windows = layout.activity_windows();
        for (window_id, window) in activity_windows.iter() {
            let kind = if window.activity_drawers.is_empty() {
                WindowKind::Ordinary
            } else {
                WindowKind::DrawerCapable
            };
            registry.register_window(
                WindowInstance::new(
                    window_id.clone(),
                    window.descriptor_id.clone(),
                    kind,
                    window_id.0.clone(),
                    window.host_mode,
                )
                .with_menu_overflow_mode(window.menu_overflow_mode),
            );
            for drawer in window.activity_drawers.values() {
                sync_drawer_layout(&mut registry, window_id, drawer, &instances);
            }
            sync_window_drawer_selection(&mut registry, window_id, window);
        }

        for window in &layout.floating_windows {
            sync_detached_drawer_window(&mut registry, window, &instances);
        }

        if let Some(active_window) = active_window {
            registry.activate_window(active_window);
        }
        registry
    }
}

/// 仅识别明确的分离抽屉浮窗身份，普通文档浮窗不投影成抽屉窗口。
fn sync_detached_drawer_window(
    registry: &mut EditorWindowRegistry,
    window: &FloatingWindowLayout,
    instances: &HashMap<&str, &ViewInstance>,
) {
    if !window.window_id.0.starts_with("drawer-window:") {
        return;
    }
    let Some(focused) = window.focused_view.clone() else {
        return;
    };
    let descriptor_id = instances
        .get(focused.0.as_str())
        .map(|instance| instance.descriptor_id.clone())
        .unwrap_or_else(|| {
            ViewDescriptorId::new(
                focused
                    .0
                    .rsplit_once('#')
                    .map_or(focused.0.as_str(), |(descriptor_id, _)| descriptor_id),
            )
        });
    let title = instances
        .get(focused.0.as_str())
        .map(|instance| instance.title.clone())
        .unwrap_or_else(|| window.title.clone());
    let owner_window = ActivityWindowId::new(window.window_id.0.clone());
    registry.register_window(WindowInstance::new(
        owner_window.clone(),
        descriptor_id.clone(),
        WindowKind::DrawerWindow,
        window.title.clone(),
        ActivityWindowHostMode::NativeWindowHandle,
    ));
    let _ = registry.register_drawer_view(DrawerViewInstance::new(
        focused.clone(),
        descriptor_id,
        title,
        owner_window,
        DrawerDockPosition::Bottom,
    ));
    registry.register_drawer_window(DrawerWindowInstance::new(
        window.window_id.clone(),
        focused,
        window.title.clone(),
    ));
}

fn sync_window_drawer_selection(
    registry: &mut EditorWindowRegistry,
    window_id: &ActivityWindowId,
    window: &ActivityWindowLayout,
) {
    // Sync must preserve collapsed drawers: retained tabs stay registered, but only active_view is selected.
    let selected = window.activity_drawers.values().find_map(|drawer| {
        let active = drawer.active_view.as_ref()?;
        drawer
            .tab_stack
            .tabs
            .contains(active)
            .then(|| active.clone())
    });

    if let Some(window) = registry.windows.get_mut(window_id) {
        window.selected_drawer = selected;
    }
}

fn sync_drawer_layout(
    registry: &mut EditorWindowRegistry,
    window_id: &ActivityWindowId,
    drawer: &ActivityDrawerLayout,
    instances: &HashMap<&str, &ViewInstance>,
) {
    let position = DrawerDockPosition::from_slot(drawer.slot);
    for instance_id in &drawer.tab_stack.tabs {
        let descriptor_id = instances
            .get(instance_id.0.as_str())
            .map(|instance| instance.descriptor_id.clone())
            .unwrap_or_else(|| {
                ViewDescriptorId::new(
                    instance_id
                        .0
                        .rsplit_once('#')
                        .map_or(instance_id.0.as_str(), |(descriptor_id, _)| descriptor_id),
                )
            });
        let title = instances
            .get(instance_id.0.as_str())
            .map(|instance| instance.title.clone())
            .unwrap_or_else(|| instance_id.0.clone());
        let drawer = DrawerViewInstance::new(
            instance_id.clone(),
            descriptor_id,
            title,
            window_id.clone(),
            position,
        );
        let _ = registry.register_drawer_view(drawer);
    }
}

fn instances_by_id(instances: &[ViewInstance]) -> HashMap<&str, &ViewInstance> {
    instances
        .iter()
        .map(|instance| (instance.instance_id.0.as_str(), instance))
        .collect()
}

#[cfg(test)]
#[path = "tests/editor_window_registry_performance_tests.rs"]
mod performance_tests;
