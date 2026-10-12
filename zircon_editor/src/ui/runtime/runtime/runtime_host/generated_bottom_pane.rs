use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::layouts::windows::workbench_host_window::PaneBodyPresentation;
use crate::ui::template_runtime::{
    RetainedUiHostAdapter, RetainedUiHostProjection, RetainedUiProjection,
};
use crate::ui::v2_design_tokens::prepare_editor_v2_document;
use zircon_runtime::ui::{
    dispatch::UiInputManager,
    surface::UiSurface,
    tree::UiRuntimeTreeLayoutExt,
    v2::{UiV2CompiledDocument, UiV2SourceFileReceipt, UiV2SurfaceBuilder},
};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    dispatch::{UiComponentEventReport, UiInputDispatchResult},
    event_ui::{UiNodeId, UiTreeId},
    layout::UiSize,
    surface::UiSurfaceFrame,
    template::UiActionRef,
    window::{UiWindowEvent, UiWindowEventMetadata, UiWindowInputPumpEvent},
};

use super::{v2_template_file_cache, EditorUiHostRuntime, EditorUiHostRuntimeError};

#[path = "generated_bottom_pane/input.rs"]
mod input;

const BINDING_PREFIX: &str = "WorkbenchGeneratedBottom/";
const INTERNAL_ACTION_PREFIX: &str = "generated-bottom-surface/";
const DRAWER_CONTROL: &str = "WorkbenchGeneratedBottomDrawer";

pub(in crate::ui::template_runtime::runtime) struct GeneratedBottomPane {
    pub(in crate::ui::template_runtime::runtime) document_id: String,
    original_compiled: Arc<UiV2CompiledDocument>,
    source_receipts: Vec<UiV2SourceFileReceipt>,
    projection: RetainedUiProjection,
    surface: UiSurface,
    input_manager: UiInputManager,
    slots: BTreeMap<String, GeneratedBottomBinding>,
    size: UiSize,
    visible: bool,
    epoch: u128,
    published: Option<Arc<UiSurfaceFrame>>,
    pointer_source: Option<input::GeneratedPointerSource>,
    last_input_result: Option<Arc<UiInputDispatchResult>>,
}

struct GeneratedBottomBinding {
    binding_id: String,
    control_id: String,
    event_kind: UiEventKind,
    node_id: UiNodeId,
    incarnation: u64,
    canonical: EditorUiBinding,
}

struct GeneratedBottomPaneInput {
    bindings: Vec<EditorUiBinding>,
    result: Arc<UiInputDispatchResult>,
}

pub(crate) struct GeneratedBottomPaneProjection {
    pub(crate) projection: RetainedUiHostProjection,
    pub(crate) accepts_text_input: bool,
    pub(crate) input_result: Option<Arc<UiInputDispatchResult>>,
}

pub(crate) struct GeneratedBottomPaneDispatch<T> {
    pub(crate) dispatched: Vec<T>,
    pub(crate) result: Arc<UiInputDispatchResult>,
}

type Feedback = BTreeMap<String, BTreeMap<String, toml::Value>>;

fn unavailable(pane_id: &str, reason: impl Into<String>) -> EditorUiHostRuntimeError {
    zircon_runtime_interface::ui::v2::UiV2AssetError::InvalidDocument {
        asset_id: format!("generated-bottom.{pane_id}"),
        detail: reason.into(),
    }
    .into()
}

impl GeneratedBottomPane {
    fn new(
        runtime: &EditorUiHostRuntime,
        pane_id: &str,
        body: &PaneBodyPresentation,
    ) -> Result<Self, EditorUiHostRuntimeError> {
        let document_id = body.document_id.as_str();
        if document_id
            != crate::ui::template_runtime::builtin::PANE_GENERATED_BOTTOM_BODY_DOCUMENT_ID
        {
            return Err(unavailable(
                pane_id,
                "pane source is outside the Generated native body owner",
            ));
        }
        // The default startup subset remains unchanged. Only the actual visible pane loads this closure.
        let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let physical = zircon_runtime::asset::runtime_asset_path_with_dev_asset_root(
            "/assets/ui/editor/host/generated_bottom_body.zui",
            &asset_root,
        );
        let theme = zircon_runtime::asset::runtime_asset_path_with_dev_asset_root(
            "/assets/ui/theme/editor_workbench_strict.zui",
            &asset_root,
        );
        let outcome = v2_template_file_cache()
            .lock()
            .expect("v2 template file cache mutex should not be poisoned")
            .load_store([physical, theme])?;
        if outcome.root_asset_id != document_id || !outcome.unresolved_imports.is_empty() {
            return Err(unavailable(
                pane_id,
                "Generated body source identity or import closure is invalid",
            ));
        }
        let original_compiled = Arc::clone(&outcome.compiled);
        let mut projection = super::super::projection::project_v2_document(
            document_id,
            original_compiled.as_ref(),
            &runtime.template_adapter,
        )?;
        super::super::pane_payload_projection::inject_pane_projection_attributes(
            &mut projection.root,
            body,
        );
        let mut compiled = original_compiled.as_ref().clone();
        let mut authored = BTreeMap::new();
        let mut controls = BTreeSet::new();
        for node in &mut compiled.arena.nodes {
            for reference in &mut node.events {
                if !reference.id.starts_with(BINDING_PREFIX) {
                    continue;
                }
                let canonical = runtime.template_adapter.resolve_binding(reference)?;
                let Some(control) = node.control_id.as_deref() else {
                    return Err(unavailable(pane_id, "authored action has no control"));
                };
                if !control.starts_with("WorkbenchGeneratedBottom")
                    || !matches!(
                        reference.event,
                        UiEventKind::Click | UiEventKind::Change | UiEventKind::Submit
                    )
                    || reference.action.is_some()
                    || canonical.path().event_kind != reference.event
                    || format!(
                        "{}/{}",
                        canonical.path().view_id,
                        canonical.path().control_id
                    ) != reference.id
                    || !matches!(canonical.payload(), EditorUiBindingPayload::MenuAction { action_id }
                        if action_id.starts_with("workbench.generated_bottom."))
                {
                    return Err(unavailable(
                        pane_id,
                        format!("unexpected authored binding {}", reference.id),
                    ));
                }
                let action = format!("{INTERNAL_ACTION_PREFIX}{}", reference.id);
                if authored
                    .insert(
                        action.clone(),
                        (
                            reference.id.clone(),
                            control.to_string(),
                            reference.event.clone(),
                            canonical,
                        ),
                    )
                    .is_some()
                {
                    return Err(unavailable(pane_id, "duplicate authored action identity"));
                }
                controls.insert(control.to_string());
                // Replace the authored route only in this holder instance with its producer action.
                reference.route = None;
                reference.action = Some(UiActionRef {
                    action: Some(action),
                    ..UiActionRef::default()
                });
            }
        }
        if authored.len() != 46 || controls.len() != 45 {
            return Err(unavailable(
                pane_id,
                "incomplete Generated authored action/control contract",
            ));
        }
        let prepared = prepare_editor_v2_document(outcome.root_document.as_ref());
        let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document_with_theme(
            UiTreeId::new(format!("generated-bottom.{pane_id}")),
            &prepared,
            &compiled,
            &runtime.active_theme,
        )?;
        for control in [DRAWER_CONTROL, "WorkbenchGeneratedBottomPanelHost"] {
            if surface.unique_control_node_id(control).is_none() {
                return Err(unavailable(
                    pane_id,
                    format!("missing actual wrapper control {control}"),
                ));
            }
        }
        let mut slots = BTreeMap::new();
        for (action, (binding_id, control_id, event_kind, canonical)) in authored {
            let mut nodes = surface.tree.nodes.iter().filter_map(|(id, node)| {
                (node
                    .template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some(control_id.as_str()))
                .then_some(*id)
            });
            let Some(node_id) = nodes.next() else {
                return Err(unavailable(
                    pane_id,
                    format!("missing action control {control_id}"),
                ));
            };
            if nodes.next().is_some() {
                return Err(unavailable(
                    pane_id,
                    format!("ambiguous action control {control_id}"),
                ));
            }
            let incarnation = surface
                .tree
                .node_incarnation(node_id)
                .ok_or_else(|| unavailable(pane_id, "missing action node incarnation"))?;
            slots.insert(
                action,
                GeneratedBottomBinding {
                    binding_id,
                    control_id,
                    event_kind,
                    node_id,
                    incarnation,
                    canonical,
                },
            );
        }
        Ok(Self {
            document_id: document_id.to_string(),
            original_compiled,
            source_receipts: outcome.source_receipts,
            projection,
            surface,
            input_manager: UiInputManager::default(),
            slots,
            size: UiSize::new(0.0, 0.0),
            visible: false,
            epoch: 0,
            published: None,
            pointer_source: None,
            last_input_result: None,
        })
    }

    fn accepts_node_input(&self, id: UiNodeId) -> bool {
        let mut cursor = Some(id);
        while let Some(id) = cursor {
            let Some(node) = self.surface.tree.node(id) else {
                return false;
            };
            if !node.state_flags.enabled
                || !node.is_render_visible()
                || node
                    .template_metadata
                    .as_ref()
                    .is_some_and(|metadata| metadata.widget.disabled)
            {
                return false;
            }
            cursor = node.parent;
        }
        true
    }

    fn current_slot(&self, action: &str) -> Option<&GeneratedBottomBinding> {
        let slot = self.slots.get(action)?;
        if !self.visible
            || self.surface.tree.node_incarnation(slot.node_id) != Some(slot.incarnation)
        {
            return None;
        }
        let node = self.surface.tree.node(slot.node_id)?;
        (self.accepts_node_input(slot.node_id)
            && node
                .template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(slot.control_id.as_str()))
        .then_some(slot)
    }

    fn binding_for_report(&self, report: &UiComponentEventReport) -> Option<EditorUiBinding> {
        if !report.delivered {
            return None;
        }
        // Ordinary component reports do not supply a binding association.
        let invocation = report.template_action.as_ref()?;
        if !invocation.is_action() || !invocation.payload.is_empty() {
            return None;
        }
        let slot = self.current_slot(invocation.target_id())?;
        let producer_matches = matches!(
            (&slot.event_kind, &report.event),
            (
                UiEventKind::Click | UiEventKind::Submit,
                zircon_runtime_interface::ui::component::UiComponentEvent::Commit { .. }
            ) | (
                UiEventKind::Change,
                zircon_runtime_interface::ui::component::UiComponentEvent::ValueChanged { .. }
            )
        );
        (slot.node_id == report.target && producer_matches).then(|| slot.canonical.clone())
    }

    fn apply_feedback(&mut self, updates: &Feedback) -> Result<(), EditorUiHostRuntimeError> {
        super::dynamic_control_state::apply_template_control_attributes_to_surface(
            &self.document_id,
            &mut self.surface,
            updates,
        )?;
        // Preserve the original mode owner's selected/checked subtree behavior.
        // This traversal borrows only this holder's tree; it adds no parallel value store.
        for (control, attributes) in updates {
            let Some(toml::Value::Boolean(selected)) = attributes.get("selected") else {
                continue;
            };
            if attributes.get("checked") != Some(&toml::Value::Boolean(*selected)) {
                continue;
            }
            let Some(root) = self.surface.unique_control_node_id(control) else {
                continue;
            };
            let mut descendants = self
                .surface
                .tree
                .node(root)
                .map(|node| node.children.clone())
                .unwrap_or_default();
            while let Some(id) = descendants.pop() {
                if let Some(node) = self.surface.tree.node(id) {
                    descendants.extend(node.children.iter().copied());
                }
                for property in ["selected", "checked"] {
                    let report = self.surface.mutate_property(
                        zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
                            id,
                            property,
                            zircon_runtime_interface::ui::component::UiValue::Bool(*selected),
                        ),
                    )?;
                    if let Some(message) = report.message {
                        return Err(unavailable(&self.document_id, message));
                    }
                }
            }
        }
        self.surface.rebuild_dirty(self.size)?;
        Ok(())
    }

    fn accepts_text_input(&self) -> bool {
        self.visible && self.surface.focus.focused
            .and_then(|id| self.surface.tree.node(id))
            .is_some_and(|node| node.state_flags.enabled && node.is_render_visible()
                && node.template_metadata.as_ref().is_some_and(|metadata|
                    !metadata.widget.disabled && metadata.widget.resolved_behavior(&metadata.component)
                        == zircon_runtime_interface::ui::widget::UiWidgetBehavior::TextInput))
    }

    fn action_token(&self, pane_id: &str, binding_id: &str) -> String {
        format!(
            "template-v2/{pane_id}/generated-bottom/e{}/{binding_id}",
            self.epoch
        )
    }

    fn set_wrapper_visible(&mut self, visible: bool) -> Result<(), EditorUiHostRuntimeError> {
        let changes = BTreeMap::from([(
            DRAWER_CONTROL.to_string(),
            BTreeMap::from([(
                "visibility".to_string(),
                toml::Value::String(if visible { "visible" } else { "collapsed" }.to_string()),
            )]),
        )]);
        super::dynamic_control_state::apply_template_control_attributes_to_surface(
            &self.document_id,
            &mut self.surface,
            &changes,
        )
    }

    fn show(&mut self, size: UiSize) -> Result<(), EditorUiHostRuntimeError> {
        if !self.visible {
            self.set_wrapper_visible(true)?;
        }
        if self.size != size {
            for index in 0..self.surface.tree.roots.len() {
                let id = self.surface.tree.roots[index];
                self.surface.tree.mark_layout_dirty(id);
            }
        }
        self.size = size;
        self.surface.rebuild_dirty(size)?;
        self.visible = true;
        Ok(())
    }

    fn hide(&mut self, metadata: UiWindowEventMetadata) -> Result<(), EditorUiHostRuntimeError> {
        self.surface.clear_focus();
        self.surface.dispatch_window_input_pump_event(
            &mut self.input_manager,
            UiWindowInputPumpEvent::Window(UiWindowEvent::window_focused(metadata, false)),
        )?;
        self.set_wrapper_visible(false)?;
        self.surface.rebuild_dirty(self.size)?;
        self.visible = false;
        self.published = None;
        self.pointer_source = None;
        self.last_input_result = None;
        Ok(())
    }
}

impl EditorUiHostRuntime {
    pub(crate) fn project_generated_bottom_pane(
        &self,
        pane_id: &str,
        body: &PaneBodyPresentation,
        size: UiSize,
    ) -> Result<GeneratedBottomPaneProjection, EditorUiHostRuntimeError> {
        if ![size.width, size.height]
            .iter()
            .all(|value| value.is_finite() && *value > 0.0)
        {
            return Err(unavailable(
                pane_id,
                "pane content has no positive finite geometry",
            ));
        }
        let mut registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let needs_new = registry
            .generated_bottom_panes
            .get(pane_id)
            .is_none_or(|holder| holder.document_id != body.document_id);
        if needs_new {
            // Constructor failure leaves the old owner and its leases unchanged.
            let holder = GeneratedBottomPane::new(self, pane_id, body)?;
            registry.remove_pane(pane_id);
            registry
                .generated_bottom_panes
                .insert(pane_id.to_string(), holder);
        }
        let holder = registry
            .generated_bottom_panes
            .get_mut(pane_id)
            .expect("prepared pane holder");
        super::super::pane_payload_projection::inject_pane_projection_attributes(
            &mut holder.projection.root,
            body,
        );
        holder.show(size)?;
        let frame = holder.surface.surface_frame();
        let changed_source = holder
            .published
            .as_ref()
            .is_none_or(|old| !Arc::ptr_eq(old, &frame));
        if changed_source {
            registry.advance_generated_bottom_epoch(pane_id);
        }
        let epoch = registry.generated_bottom_epoch(pane_id);
        let holder = registry
            .generated_bottom_panes
            .get_mut(pane_id)
            .expect("prepared pane holder");
        holder.epoch = epoch;
        holder.published = Some(Arc::clone(&frame));
        let mut model = self.build_host_model_with_surface(&holder.projection, &holder.surface)?;
        for node in &mut model.nodes {
            for binding in &mut node.bindings {
                let Some(slot) = holder
                    .slots
                    .get(&format!("{INTERNAL_ACTION_PREFIX}{}", binding.binding_id))
                else {
                    continue;
                };
                binding.action_id = holder.action_token(pane_id, &slot.binding_id);
            }
        }
        let mut host = RetainedUiHostAdapter::build_projection(&model);
        for node in &mut host.nodes {
            node.source_surface_frame = Some(Arc::clone(&frame));
        }
        host.source_surface_frame = Some(frame);
        Ok(GeneratedBottomPaneProjection {
            projection: host,
            accepts_text_input: holder.accepts_text_input(),
            input_result: holder.last_input_result.clone(),
        })
    }

    pub(crate) fn owns_generated_bottom_pane(&self, pane_id: &str) -> bool {
        self.template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .generated_bottom_panes
            .contains_key(pane_id)
    }

    pub(crate) fn generated_bottom_pane_visible(&self, pane_id: &str) -> bool {
        self.template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .generated_bottom_panes
            .get(pane_id)
            .is_some_and(|holder| holder.visible)
    }

    /// The callback is restricted to the canonical Editor preview dispatcher and pure feedback.
    /// It must not query this runtime, publish a projection, or mutate Registry control state.
    pub(crate) fn dispatch_generated_bottom_action_token<T>(
        &self,
        token: &str,
        control_id: &str,
        edited_value: Option<&str>,
        dispatch: impl FnOnce(&EditorUiBinding) -> Result<(T, Feedback), String>,
    ) -> Option<Result<T, EditorUiHostRuntimeError>> {
        let mut registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let (pane_id, action) =
            registry
                .generated_bottom_panes
                .iter()
                .find_map(|(pane_id, holder)| {
                    holder.slots.keys().find_map(|action| {
                        let slot = holder.current_slot(action)?;
                        let expected = holder.action_token(pane_id, &slot.binding_id);
                        (token == expected
                            && slot.control_id == control_id
                            && if edited_value.is_some() {
                                matches!(slot.event_kind, UiEventKind::Change | UiEventKind::Submit)
                            } else {
                                slot.event_kind == UiEventKind::Click
                            }
                            && holder.published.as_ref().is_some_and(|published| {
                                Arc::ptr_eq(published, &holder.surface.surface_frame())
                            }))
                        .then(|| (pane_id.clone(), action.clone()))
                    })
                })?;
        Some((|| {
            let holder = registry
                .generated_bottom_panes
                .get_mut(&pane_id)
                .expect("leased Generated pane");
            let canonical = holder
                .current_slot(&action)
                .expect("verified action slot")
                .canonical
                .clone();
            if let Some(value) = edited_value {
                let attributes = BTreeMap::from([(
                    control_id.to_string(),
                    BTreeMap::from([("value".to_string(), toml::Value::String(value.to_string()))]),
                )]);
                holder.apply_feedback(&attributes)?;
            }
            let (result, updates) =
                dispatch(&canonical).map_err(|error| unavailable(&pane_id, error))?;
            holder.apply_feedback(&updates)?;
            Ok(result)
        })())
    }

    pub(crate) fn generated_bottom_pane_ids(&self) -> Vec<String> {
        self.template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .generated_bottom_panes
            .keys()
            .cloned()
            .collect()
    }

    pub(crate) fn deactivate_generated_bottom_pane_input(
        &self,
        pane_id: &str,
        metadata: UiWindowEventMetadata,
    ) -> Result<bool, EditorUiHostRuntimeError> {
        let mut registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let Some(holder) = registry.generated_bottom_panes.get_mut(pane_id) else {
            return Ok(false);
        };
        if !holder.visible {
            return Ok(false);
        }
        holder.hide(metadata)?;
        registry.advance_generated_bottom_epoch(pane_id);
        let epoch = registry.generated_bottom_epoch(pane_id);
        registry
            .generated_bottom_panes
            .get_mut(pane_id)
            .expect("hidden holder remains loaded")
            .epoch = epoch;
        Ok(true)
    }

    pub(crate) fn generated_bottom_pane_input_result(
        &self,
        pane_id: &str,
    ) -> Option<Arc<UiInputDispatchResult>> {
        self.template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .generated_bottom_panes
            .get(pane_id)?
            .last_input_result
            .clone()
    }

    #[cfg(test)]
    pub(crate) fn generated_bottom_owner_state_for_test(
        &self,
        pane_id: &str,
    ) -> Option<(bool, u128, Arc<UiSurfaceFrame>)> {
        let registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let holder = registry.generated_bottom_panes.get(pane_id)?;
        Some((holder.visible, holder.epoch, holder.surface.surface_frame()))
    }

    #[cfg(test)]
    pub(crate) fn generated_bottom_control_properties_for_test(
        &self,
        pane_id: &str,
        control: &str,
    ) -> Option<BTreeMap<String, zircon_runtime_interface::ui::component::UiValue>> {
        let registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let holder = registry.generated_bottom_panes.get(pane_id)?;
        let id = holder.surface.unique_control_node_id(control)?;
        let snapshot = zircon_runtime::ui::surface::reflector_snapshot(&holder.surface, None);
        Some(
            snapshot
                .nodes
                .get(&id)?
                .properties
                .iter()
                .map(|(name, property)| (name.clone(), property.resolved_value.clone()))
                .collect(),
        )
    }

    #[cfg(test)]
    pub(crate) fn generated_bottom_subtree_selection_for_test(
        &self,
        pane_id: &str,
        control: &str,
    ) -> Option<Vec<(UiNodeId, Option<bool>, bool)>> {
        let registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let holder = registry.generated_bottom_panes.get(pane_id)?;
        let root = holder.surface.unique_control_node_id(control)?;
        let mut pending = vec![root];
        let mut values = Vec::new();
        while let Some(id) = pending.pop() {
            let node = holder.surface.tree.node(id)?;
            pending.extend(node.children.iter().copied());
            let selected = node
                .template_metadata
                .as_ref()
                .and_then(|metadata| metadata.attributes.get("selected"))
                .and_then(toml::Value::as_bool);
            values.push((id, selected, node.state_flags.checked));
        }
        Some(values)
    }

    #[cfg(test)]
    pub(crate) fn generated_bottom_bindings_for_test(
        &self,
        pane_id: &str,
    ) -> Vec<(String, String, UiEventKind, EditorUiBinding)> {
        let registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let holder = registry
            .generated_bottom_panes
            .get(pane_id)
            .expect("actual loaded Generated owner");
        // Verify the original compiled instance association rather than a separate registry inventory.
        let mut authored = Vec::new();
        for node in &holder.original_compiled.arena.nodes {
            for reference in &node.events {
                if !reference.id.starts_with(BINDING_PREFIX) {
                    continue;
                }
                assert!(
                    reference.action.is_none(),
                    "original source metadata remains authored"
                );
                let action = format!("{INTERNAL_ACTION_PREFIX}{}", reference.id);
                let slot = holder
                    .slots
                    .get(&action)
                    .expect("same original binding has an instance action");
                assert_eq!(node.control_id.as_deref(), Some(slot.control_id.as_str()));
                assert_eq!(reference.event, slot.event_kind);
                let actual = holder
                    .surface
                    .tree
                    .node(slot.node_id)
                    .expect("same Surface node");
                let metadata = actual
                    .template_metadata
                    .as_ref()
                    .expect("actual template metadata");
                assert!(metadata
                    .bindings
                    .iter()
                    .any(|event| event.id == reference.id
                        && event
                            .action
                            .as_ref()
                            .and_then(|action| action.action.as_deref())
                            == Some(action.as_str())));
                authored.push((
                    slot.binding_id.clone(),
                    slot.control_id.clone(),
                    slot.event_kind.clone(),
                    slot.canonical.clone(),
                ));
            }
        }
        authored
    }

    #[cfg(test)]
    pub(crate) fn generated_bottom_pane_source_receipts_for_test(
        &self,
        pane_id: &str,
    ) -> Option<Vec<UiV2SourceFileReceipt>> {
        let registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let holder = registry.generated_bottom_panes.get(pane_id)?;
        (holder.visible
            && holder
                .published
                .as_ref()
                .is_some_and(|frame| Arc::ptr_eq(frame, &holder.surface.surface_frame())))
        .then(|| holder.source_receipts.clone())
    }
}
