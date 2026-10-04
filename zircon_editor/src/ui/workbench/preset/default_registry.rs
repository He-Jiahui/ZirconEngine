use std::collections::HashSet;

use serde_json::Value;

use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};
use crate::ui::workbench::window_registry::EditorWindowRegistry;

use super::default_layout::view_instance_id_for_window;
use super::{EditorFunctionalWindowKind, EditorUiDesignStack};

impl EditorUiDesignStack {
    pub fn default_view_instances(&self) -> Vec<ViewInstance> {
        let instance_capacity =
            self.window_model
                .windows
                .iter()
                .fold(0usize, |capacity, window| {
                    capacity
                        .saturating_add(window.primary_views.len())
                        .saturating_add(window.drawer_views.len())
                });
        let mut seen = HashSet::with_capacity(instance_capacity);
        let mut instances = Vec::with_capacity(instance_capacity);

        for window in &self.window_model.windows {
            for view in &window.primary_views {
                let instance = self.view_instance_for_window(window.kind, view, true);
                if admit_view_instance_id(&mut seen, &instance.instance_id) {
                    instances.push(instance);
                }
            }
            for view in &window.drawer_views {
                let instance = self.view_instance_for_window(window.kind, view, false);
                if admit_view_instance_id(&mut seen, &instance.instance_id) {
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

fn admit_view_instance_id(
    seen: &mut HashSet<ViewInstanceId>,
    instance_id: &ViewInstanceId,
) -> bool {
    if seen.contains(instance_id) {
        return false;
    }
    seen.insert(instance_id.clone());
    true
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
#[path = "default_registry/tests/optimization_batch_ix_editor634_tests.rs"]
mod optimization_batch_ix_editor634_tests;

#[cfg(test)]
#[path = "tests/default_registry.rs"]
mod tests;
