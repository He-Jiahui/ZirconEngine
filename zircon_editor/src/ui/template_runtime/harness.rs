use toml::Value;
use zircon_runtime::ui::surface::UiSurface;

#[cfg(test)]
use crate::core::editor_event::EditorEventJournal;
#[cfg(test)]
use crate::ui::slint_host::tab_drag::ResolvedWorkbenchTabDropRoute;

use super::{SlintUiHostModel, SlintUiHostProjection, SlintUiNodeProjection, SlintUiProjection};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorUiCompatibilitySnapshot {
    pub components: Vec<String>,
    pub control_ids: Vec<String>,
    pub binding_ids: Vec<String>,
    pub host_nodes: Vec<String>,
    pub surface_nodes: Vec<String>,
    pub slint_nodes: Vec<String>,
    pub route_bindings: Vec<String>,
    pub route_key_entries: Vec<String>,
    pub frame_entries: Vec<String>,
    pub attribute_entries: Vec<String>,
    pub style_token_entries: Vec<String>,
    pub text_entries: Vec<String>,
    pub icon_entries: Vec<String>,
    pub event_entries: Vec<String>,
    pub route_result_entries: Vec<String>,
}

#[derive(Default)]
pub struct EditorUiCompatibilityHarness;

impl EditorUiCompatibilityHarness {
    pub fn capture_projection_snapshot(
        projection: &SlintUiProjection,
    ) -> EditorUiCompatibilitySnapshot {
        let mut snapshot = EditorUiCompatibilitySnapshot {
            binding_ids: projection
                .bindings
                .iter()
                .map(|binding| binding.binding_id.clone())
                .collect(),
            ..EditorUiCompatibilitySnapshot::default()
        };
        collect_node_snapshot(&projection.root, &mut snapshot);
        snapshot
    }

    pub fn capture_host_model_snapshot(
        host_model: &SlintUiHostModel,
    ) -> EditorUiCompatibilitySnapshot {
        let mut snapshot = EditorUiCompatibilitySnapshot::default();
        for node in &host_model.nodes {
            snapshot.components.push(node.component.clone());
            if let Some(control_id) = &node.control_id {
                snapshot.control_ids.push(control_id.clone());
            }
            snapshot.host_nodes.push(format!(
                "{}|{}|{}",
                node.node_id,
                node.component,
                node.control_id.as_deref().unwrap_or_default()
            ));
            snapshot
                .frame_entries
                .push(format!("{}={}", node.node_id, render_frame(node.frame)));
            for binding in &node.bindings {
                snapshot.binding_ids.push(binding.binding_id.clone());
                if let Some(route_id) = binding.route_id {
                    snapshot
                        .route_bindings
                        .push(format!("{}@{}", binding.binding_id, route_id.0));
                }
            }
            for (key, value) in &node.attributes {
                snapshot.attribute_entries.push(format!(
                    "{}.{}={}",
                    node.node_id,
                    key,
                    render_toml_value(value)
                ));
            }
            for (key, value) in &node.style_tokens {
                snapshot
                    .style_token_entries
                    .push(format!("{}.{}={}", node.node_id, key, value));
            }
        }
        snapshot
    }

    pub fn capture_shared_surface_snapshot(surface: &UiSurface) -> EditorUiCompatibilitySnapshot {
        let mut snapshot = EditorUiCompatibilitySnapshot::default();
        for node in surface.tree.nodes.values() {
            if let Some(template) = &node.template_metadata {
                snapshot.components.push(template.component.clone());
                if let Some(control_id) = &template.control_id {
                    snapshot.control_ids.push(control_id.clone());
                }
                snapshot.surface_nodes.push(format!(
                    "{}|{}|{}",
                    node.node_path,
                    template.component,
                    template.control_id.as_deref().unwrap_or_default()
                ));
                snapshot.frame_entries.push(format!(
                    "{}={}",
                    node.node_path,
                    render_frame(node.layout_cache.frame)
                ));
                for binding in &template.bindings {
                    snapshot.binding_ids.push(binding.id.clone());
                }
                for (key, value) in &template.attributes {
                    snapshot.attribute_entries.push(format!(
                        "{}.{}={}",
                        node.node_path,
                        key,
                        render_toml_value(value)
                    ));
                }
                for (key, value) in &template.style_tokens {
                    snapshot
                        .style_token_entries
                        .push(format!("{}.{}={}", node.node_path, key, value));
                }
            }
        }
        snapshot
    }

    pub fn capture_slint_host_projection_snapshot(
        projection: &SlintUiHostProjection,
    ) -> EditorUiCompatibilitySnapshot {
        let mut snapshot = EditorUiCompatibilitySnapshot::default();
        for node in &projection.nodes {
            snapshot.components.push(node.component.clone());
            if let Some(control_id) = &node.control_id {
                snapshot.control_ids.push(control_id.clone());
            }
            snapshot.slint_nodes.push(format!(
                "{}|{}|{}",
                node.node_id,
                node.kind.as_str(),
                node.control_id.as_deref().unwrap_or_default()
            ));
            snapshot
                .frame_entries
                .push(format!("{}={}", node.node_id, render_frame(node.frame)));
            if let Some(text) = &node.text {
                snapshot
                    .text_entries
                    .push(format!("{}={}", node.node_id, text));
            }
            if let Some(icon) = &node.icon {
                snapshot
                    .icon_entries
                    .push(format!("{}={}", node.node_id, icon));
            }
            for route in &node.routes {
                snapshot.binding_ids.push(route.binding_id.clone());
                if let Some(route_id) = route.route_id {
                    snapshot
                        .route_bindings
                        .push(format!("{}@{}", route.binding_id, route_id.0));
                }
            }
            for (key, value) in &node.style_tokens {
                snapshot
                    .style_token_entries
                    .push(format!("{}.{}={}", node.node_id, key, value));
            }
        }
        snapshot
    }

    pub fn capture_floating_window_overlay_snapshot(
        floating_windows: &[crate::ui::slint_host::FloatingWindowData],
    ) -> EditorUiCompatibilitySnapshot {
        let mut snapshot = EditorUiCompatibilitySnapshot::default();
        for window in floating_windows {
            let window_id = window.window_id.to_string();
            let node_id = format!("floating-window/{window_id}");
            snapshot
                .components
                .push("FloatingWindowOverlay".to_string());
            snapshot.control_ids.push(window_id.clone());
            snapshot
                .host_nodes
                .push(format!("{node_id}|FloatingWindow|{window_id}"));
            snapshot.frame_entries.push(format!(
                "{node_id}={}",
                render_frame_values(
                    window.frame.x,
                    window.frame.y,
                    window.frame.width,
                    window.frame.height
                )
            ));
            snapshot
                .route_key_entries
                .push(format!("{node_id}.attach={}", window.target_group));
            snapshot
                .route_key_entries
                .push(format!("{node_id}.left={}", window.left_edge_target_group));
            snapshot.route_key_entries.push(format!(
                "{node_id}.right={}",
                window.right_edge_target_group
            ));
            snapshot
                .route_key_entries
                .push(format!("{node_id}.top={}", window.top_edge_target_group));
            snapshot.route_key_entries.push(format!(
                "{node_id}.bottom={}",
                window.bottom_edge_target_group
            ));
            snapshot
                .attribute_entries
                .push(format!("{node_id}.title={}", window.title));
            snapshot.attribute_entries.push(format!(
                "{node_id}.focus_target_id={}",
                window.focus_target_id
            ));
            snapshot.attribute_entries.push(format!(
                "{node_id}.active_pane.id={}",
                window.active_pane.id
            ));
            snapshot.attribute_entries.push(format!(
                "{node_id}.active_pane.kind={}",
                window.active_pane.kind
            ));
        }
        snapshot
    }

    #[cfg(test)]
    pub(crate) fn capture_event_journal_delta_snapshot(
        journal: &EditorEventJournal,
        baseline_records: usize,
    ) -> EditorUiCompatibilitySnapshot {
        let mut snapshot = EditorUiCompatibilitySnapshot::default();
        for record in journal.records().iter().skip(baseline_records) {
            snapshot.event_entries.push(format!(
                "{:?}|{:?}|effects={:?}|undo={:?}|error={}",
                record.source,
                record.event,
                record.effects,
                record.undo_policy,
                record.result.error.as_deref().unwrap_or_default()
            ));
        }
        snapshot
    }

    #[cfg(test)]
    pub(crate) fn capture_resolved_tab_drop_route_snapshot(
        route: &ResolvedWorkbenchTabDropRoute,
    ) -> EditorUiCompatibilitySnapshot {
        let mut snapshot = EditorUiCompatibilitySnapshot::default();
        snapshot
            .route_result_entries
            .push(format!("group={:?}", route.target_group));
        snapshot
            .route_result_entries
            .push(format!("label={}", route.target_label));
        match &route.target {
            crate::ui::slint_host::tab_drag::ResolvedWorkbenchTabDropTarget::Attach(drop) => {
                snapshot
                    .route_result_entries
                    .push("target=attach".to_string());
                snapshot
                    .route_result_entries
                    .push(format!("host={:?}", drop.host));
                snapshot
                    .route_result_entries
                    .push(format!("anchor={:?}", drop.anchor));
            }
            crate::ui::slint_host::tab_drag::ResolvedWorkbenchTabDropTarget::Split {
                workspace,
                path,
                axis,
                placement,
            } => {
                snapshot
                    .route_result_entries
                    .push("target=split".to_string());
                snapshot
                    .route_result_entries
                    .push(format!("workspace={workspace:?}"));
                snapshot
                    .route_result_entries
                    .push(format!("path={}", render_path(path)));
                snapshot.route_result_entries.push(format!("axis={axis:?}"));
                snapshot
                    .route_result_entries
                    .push(format!("placement={placement:?}"));
            }
        }
        snapshot
    }
}

fn collect_node_snapshot(
    node: &SlintUiNodeProjection,
    snapshot: &mut EditorUiCompatibilitySnapshot,
) {
    snapshot.components.push(node.component.clone());
    if let Some(control_id) = &node.control_id {
        snapshot.control_ids.push(control_id.clone());
    }
    for child in &node.children {
        collect_node_snapshot(child, snapshot);
    }
}

fn render_toml_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn render_frame(frame: zircon_runtime::ui::layout::UiFrame) -> String {
    render_frame_values(frame.x, frame.y, frame.width, frame.height)
}

fn render_frame_values(x: f32, y: f32, width: f32, height: f32) -> String {
    format!(
        "{},{},{},{}",
        render_number(x),
        render_number(y),
        render_number(width),
        render_number(height)
    )
}

fn render_number(value: f32) -> String {
    if (value.fract()).abs() <= f32::EPSILON {
        format!("{value:.0}")
    } else {
        let mut rendered = format!("{value:.3}");
        while rendered.contains('.') && rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.pop();
        }
        rendered
    }
}

#[cfg(test)]
fn render_path(path: &[usize]) -> String {
    if path.is_empty() {
        String::new()
    } else {
        path.iter()
            .map(|segment| segment.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}
