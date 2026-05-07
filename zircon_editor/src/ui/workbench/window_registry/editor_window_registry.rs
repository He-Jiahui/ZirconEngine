use std::collections::BTreeMap;

use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityWindowHostMode, ActivityWindowId, ActivityWindowLayout,
    FloatingWindowLayout, MainPageId, WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstance, ViewInstanceId};

use super::{
    DrawerBinding, DrawerDockPosition, DrawerViewInstance, DrawerWindowInstance, WindowInstance,
    WindowKind,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

    pub fn bind_drawer(&mut self, binding: DrawerBinding) -> Result<(), String> {
        let drawer = self
            .drawer_views
            .get_mut(&binding.drawer_view)
            .ok_or_else(|| format!("missing drawer view {}", binding.drawer_view.0))?;
        let old_owner = drawer.owner_window.clone();
        if let Some(old_window) = self.windows.get_mut(&old_owner) {
            for views in old_window.drawer_views.values_mut() {
                views.retain(|view| view != &binding.drawer_view);
            }
            if old_window.selected_drawer.as_ref() == Some(&binding.drawer_view) {
                old_window.selected_drawer = None;
            }
        }
        drawer.owner_window = binding.window_id.clone();
        drawer.dock_position = binding.dock_position;
        let rebound = drawer.clone();
        self.register_drawer_view(rebound)
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

    pub fn activate_window(&mut self, window_id: ActivityWindowId) {
        self.active_window = self.windows.contains_key(&window_id).then_some(window_id);
    }

    pub fn selected_drawer_for_active_window(&self) -> Option<&DrawerViewInstance> {
        let window = self.active_window()?;
        let selected = window.selected_drawer.as_ref()?;
        self.drawer_views.get(selected)
    }

    pub fn sync_from_layout(layout: &WorkbenchLayout, instances: &[ViewInstance]) -> Self {
        let mut registry = Self::default();
        let instances = instances_by_id(instances);
        let active_window = layout.active_activity_window_id();

        for (window_id, window) in layout.activity_windows() {
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
                sync_drawer_layout(&mut registry, &window_id, drawer, &instances);
            }
            sync_window_drawer_selection(&mut registry, &window_id, &window);
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

fn sync_detached_drawer_window(
    registry: &mut EditorWindowRegistry,
    window: &FloatingWindowLayout,
    instances: &BTreeMap<ViewInstanceId, ViewInstance>,
) {
    if !window.window_id.0.starts_with("drawer-window:") {
        return;
    }
    let Some(focused) = window.focused_view.clone() else {
        return;
    };
    let descriptor_id = instances
        .get(&focused)
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
        .get(&focused)
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
    instances: &BTreeMap<ViewInstanceId, ViewInstance>,
) {
    let position = DrawerDockPosition::from_slot(drawer.slot);
    for instance_id in &drawer.tab_stack.tabs {
        let descriptor_id = instances
            .get(instance_id)
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
            .get(instance_id)
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

fn instances_by_id(instances: &[ViewInstance]) -> BTreeMap<ViewInstanceId, ViewInstance> {
    instances
        .iter()
        .cloned()
        .map(|instance| (instance.instance_id.clone(), instance))
        .collect()
}
