use std::collections::BTreeSet;

use serde_json::Value;

use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance};
use crate::ui::workbench::window_registry::EditorWindowRegistry;

use super::default_layout::view_instance_id_for_window;
use super::{EditorFunctionalWindowKind, EditorUiDesignStack};

impl EditorUiDesignStack {
    pub fn default_view_instances(&self) -> Vec<ViewInstance> {
        let mut seen = BTreeSet::new();
        let mut instances = Vec::new();

        for window in &self.window_model.windows {
            for view in &window.primary_views {
                let instance = self.view_instance_for_window(window.kind, view, true);
                if seen.insert(instance.instance_id.clone()) {
                    instances.push(instance);
                }
            }
            for view in &window.drawer_views {
                let instance = self.view_instance_for_window(window.kind, view, false);
                if seen.insert(instance.instance_id.clone()) {
                    instances.push(instance);
                }
            }
        }

        instances
    }

    pub fn default_window_registry(&self) -> EditorWindowRegistry {
        let layout = self.default_workbench_layout();
        let instances = self.default_view_instances();
        EditorWindowRegistry::sync_from_layout(&layout, &instances)
    }

    fn view_instance_for_window(
        &self,
        window_kind: EditorFunctionalWindowKind,
        view: &str,
        primary_view: bool,
    ) -> ViewInstance {
        ViewInstance {
            instance_id: view_instance_id_for_window(window_kind, view),
            descriptor_id: ViewDescriptorId::new(view),
            title: title_from_view(view),
            serializable_payload: Value::Null,
            dirty: false,
            host: self.view_host_for_window(window_kind, view, primary_view),
        }
    }

    fn view_host_for_window(
        &self,
        window_kind: EditorFunctionalWindowKind,
        view: &str,
        primary_view: bool,
    ) -> ViewHost {
        if !primary_view {
            return ViewHost::Drawer(self.drawer_slot_for_view(view));
        }

        if window_kind == EditorFunctionalWindowKind::Workbench {
            ViewHost::Document(MainPageId::workbench(), vec![])
        } else {
            ViewHost::FloatingWindow(
                MainPageId::new(format!("window:{}", window_kind.slug())),
                vec![],
            )
        }
    }
}

fn title_from_view(view: &str) -> String {
    let view = view.strip_prefix("editor.").unwrap_or(view);
    view.split(['.', '_'])
        .filter(|part| !part.is_empty())
        .map(capitalize_ascii)
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize_ascii(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut title = first.to_ascii_uppercase().to_string();
    title.extend(chars);
    title
}

#[cfg(test)]
mod tests {
    use crate::ui::workbench::layout::{ActivityWindowHostMode, ActivityWindowId};
    use crate::ui::workbench::view::ViewInstanceId;
    use crate::ui::workbench::window_registry::{DrawerDockPosition, WindowKind};

    use super::*;

    #[test]
    fn default_view_instances_are_unique_per_functional_window() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
        let instances = stack.default_view_instances();
        let ids = instances
            .iter()
            .map(|instance| instance.instance_id.clone())
            .collect::<BTreeSet<_>>();

        assert_eq!(ids.len(), instances.len());
        assert!(ids.contains(&ViewInstanceId::new("editor.inspector#1")));
        assert!(ids.contains(&ViewInstanceId::new("editor.inspector#material_editor")));
        assert!(ids.contains(&ViewInstanceId::new("editor.asset_browser#material_editor")));
    }

    #[test]
    fn default_window_registry_syncs_preset_windows_drawers_and_titles() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
        let registry = stack.default_window_registry();

        let workbench = registry
            .get_window(&ActivityWindowId::workbench())
            .expect("workbench window");
        assert_eq!(workbench.kind, WindowKind::DrawerCapable);
        assert_eq!(
            workbench.selected_drawer,
            Some(ViewInstanceId::new("editor.hierarchy#1"))
        );

        let material = registry
            .get_window(&ActivityWindowId::new("window:material_editor"))
            .expect("material editor window");
        assert_eq!(material.kind, WindowKind::DrawerCapable);
        assert_eq!(
            material.host_mode,
            ActivityWindowHostMode::NativeWindowHandle
        );
        assert_eq!(
            material
                .drawer_views
                .get(&DrawerDockPosition::RightTop)
                .expect("right drawer"),
            &vec![ViewInstanceId::new("editor.inspector#material_editor")]
        );

        let inspector = registry
            .get_drawer_view(&ViewInstanceId::new("editor.inspector#material_editor"))
            .expect("material inspector drawer view");
        assert_eq!(inspector.title, "Inspector");
        assert_eq!(
            inspector.owner_window,
            ActivityWindowId::new("window:material_editor")
        );
    }
}
