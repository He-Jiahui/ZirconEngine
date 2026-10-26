use std::{
    path::PathBuf,
    sync::{Arc, Weak},
};

use zircon_runtime::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::template::UiCompiledDocument;
use zircon_runtime::ui::v2::{UiV2CompiledDocument, UiV2SurfaceBuilder};
use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2SourceFileReceipt};
use zircon_runtime_interface::ui::v2::UiV2AssetDocument;
use zircon_runtime_interface::ui::{
    dispatch::UiInputEvent,
    surface::UiSurfaceFrame,
    window::{UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata, UiWindowInputPumpEvent},
};
use zircon_runtime_interface::ui::{event_ui::UiTreeId, layout::UiSize};

use crate::ui::asset_editor::session::UiAssetEditorSessionError;
use crate::ui::template::EditorTemplateRuntimeService;
use crate::ui::v2_design_tokens::prepare_editor_v2_document;

#[path = "visibility/mod.rs"]
pub(crate) mod visibility;

#[derive(Debug, PartialEq)]
pub struct UiAssetPreviewHost {
    preview_size: UiSize,
    surface: UiSurface,
    builtin_reference: Option<String>,
    source_receipts: Arc<[UiV2SourceFileReceipt]>,
    input_feedback: String,
    last_input_result: Option<Arc<zircon_runtime_interface::ui::dispatch::UiInputDispatchResult>>,
    pointer_source: Option<PreviewPointerSource>,
    visibility_state: Option<visibility::PreviewVisibilityState>,
}

impl UiAssetPreviewHost {
    pub fn new(
        preview_size: UiSize,
        asset_id: &str,
        compiled: &UiCompiledDocument,
    ) -> Result<Self, UiAssetEditorSessionError> {
        let template_service = EditorTemplateRuntimeService;
        let mut surface = template_service.build_surface_from_compiled_document(
            UiTreeId::new(format!("ui_asset.preview.{asset_id}")),
            compiled,
        )?;
        surface.compute_layout(preview_size)?;
        Ok(Self {
            preview_size,
            surface,
            builtin_reference: None,
            source_receipts: Arc::from([]),
            input_feedback: String::new(),
            last_input_result: None,
            pointer_source: None,
            visibility_state: None,
        })
    }

    pub fn new_v2(
        preview_size: UiSize,
        document: &UiV2AssetDocument,
        compiled: &UiV2CompiledDocument,
    ) -> Result<Self, UiAssetEditorSessionError> {
        let prepared_document = prepare_editor_v2_document(document);
        let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
            UiTreeId::new(format!("ui_asset.preview.v2.{}", document.asset.id)),
            &prepared_document,
            compiled,
        )?;
        surface.compute_layout(preview_size)?;
        Ok(Self {
            preview_size,
            surface,
            builtin_reference: None,
            source_receipts: Arc::from([]),
            input_feedback: String::new(),
            last_input_result: None,
            pointer_source: None,
            visibility_state: None,
        })
    }

    pub(crate) fn load_builtin(
        preview_size: UiSize,
        reference: &str,
        descriptor: &crate::ui::template::EditorComponentDescriptor,
    ) -> Result<Self, UiAssetEditorSessionError> {
        let unavailable = |reason: String| UiAssetEditorSessionError::BuiltinPreview { reason };
        let (resource, component) = reference
            .rsplit_once('#')
            .ok_or_else(|| unavailable(format!("invalid component reference {reference}")))?;
        let relative = resource
            .strip_prefix("res://")
            .ok_or_else(|| unavailable(format!("invalid builtin resource {resource}")))?;
        let relative = zircon_runtime::asset::AssetRelativePath::parse(relative)
            .map_err(|error| unavailable(format!("invalid builtin resource path: {error}")))?;
        let path = zircon_runtime::asset::runtime_asset_path_with_dev_asset_root(
            &relative,
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        );
        let outcome =
            crate::ui::asset_editor::node_projection::node_projection_v2_store_file_cache()
                .lock()
                .map_err(|_| unavailable("UI asset source cache is poisoned".to_string()))?
                .load_store([path])?;
        if outcome.root_document.asset.id != resource {
            return Err(unavailable(format!(
                "source identity does not match {resource}"
            )));
        }
        if !outcome.unresolved_imports.is_empty() {
            return Err(unavailable(format!(
                "unresolved imports for {reference}: {:?}",
                outcome.unresolved_imports
            )));
        }
        if outcome.root_document.asset.kind
            != zircon_runtime_interface::ui::v2::UiV2AssetKind::Component
        {
            return Err(UiAssetEditorSessionError::UnexpectedV2Kind {
                expected: zircon_runtime_interface::ui::v2::UiV2AssetKind::Component,
                actual: outcome.root_document.asset.kind,
            });
        }
        let mut document = outcome.root_document.as_ref().clone();
        let root = document
            .components
            .get(component)
            .ok_or_else(|| unavailable(format!("source does not declare {component}")))?
            .root
            .clone();
        if descriptor.document_id != resource || descriptor.component_id != component {
            return Err(unavailable(format!(
                "catalog identity does not match {reference}"
            )));
        }
        let node = document.nodes.get_mut(&root).ok_or_else(|| {
            unavailable(format!("component {component} has no declared root {root}"))
        })?;
        // Palette defaults initialize this component instance; the loaded source stays unchanged.
        for prop in &descriptor.props {
            if let Some(value) = builtin_prop_default(&prop.default) {
                node.props.insert(prop.name.clone(), value);
            }
        }
        document.root = Some(zircon_runtime_interface::ui::v2::UiV2Root { node: root });
        let compiled =
            UiV2DocumentCompiler::compile_with_prototype_store(&document, outcome.store.as_ref())?;
        let mut preview = Self::new_v2(preview_size, &document, &compiled)?;
        preview.builtin_reference = Some(reference.to_string());
        preview.source_receipts = outcome.source_receipts.into();
        Ok(preview)
    }

    pub(crate) fn builtin_reference(&self) -> Option<&str> {
        self.builtin_reference.as_deref()
    }

    pub(crate) fn source_receipts(&self) -> &[UiV2SourceFileReceipt] {
        self.source_receipts.as_ref()
    }

    pub(crate) fn accepts_text_input(&self) -> bool {
        self.surface
            .focus
            .focused
            .and_then(|id| self.surface.tree.node(id))
            .is_some_and(|node| {
                node.state_flags.enabled
                    && node.is_render_visible()
                    && node.template_metadata.as_ref().is_some_and(|metadata| {
                        !metadata.widget.disabled
                            && metadata.widget.resolved_behavior(&metadata.component)
                                == zircon_runtime_interface::ui::widget::UiWidgetBehavior::TextInput
                    })
            })
    }

    pub fn last_input_result(
        &self,
    ) -> Option<&zircon_runtime_interface::ui::dispatch::UiInputDispatchResult> {
        self.last_input_result.as_deref()
    }

    pub(crate) fn published_input_result(
        &self,
    ) -> Option<Arc<zircon_runtime_interface::ui::dispatch::UiInputDispatchResult>> {
        self.last_input_result.clone()
    }

    pub(crate) fn input_feedback(&self) -> &str {
        &self.input_feedback
    }

    pub(crate) fn dispatch_input(
        &mut self,
        source_frame: &Arc<UiSurfaceFrame>,
        event: UiWindowInputPumpEvent,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let current_frame = self.surface.surface_frame();
        let captured_continuation = match &event {
            UiWindowInputPumpEvent::Input(input) => {
                self.accepts_captured_pointer_source(source_frame, input)
            }
            UiWindowInputPumpEvent::Window(_) => false,
        };
        if !Arc::ptr_eq(source_frame, &current_frame) && !captured_continuation {
            return Err(UiAssetEditorSessionError::RetiredPreviewInput);
        }
        // Input retires the property gesture; its old baseline must not overwrite input effects.
        self.visibility_state = None;
        let mut manager = zircon_runtime::ui::dispatch::UiInputManager::default();
        let result = match event {
            UiWindowInputPumpEvent::Input(input) => {
                if self.surface.window_state.closed || self.surface.window_state.destroyed {
                    return Err(UiAssetEditorSessionError::RetiredPreviewInput);
                }
                if matches!(&input, UiInputEvent::Pointer(pointer) if pointer.event.kind == zircon_runtime_interface::ui::surface::UiPointerEventKind::Down)
                    && self.surface.window_state.focused == Some(false)
                {
                    let metadata = match &input {
                        UiInputEvent::Pointer(pointer) => &pointer.metadata,
                        _ => unreachable!(),
                    };
                    let window_metadata =
                        zircon_runtime_interface::ui::window::UiWindowEventMetadata::for_window(
                            metadata.window_id.clone().unwrap_or_default(),
                            metadata.timestamp,
                            metadata.sequence,
                        )
                        .synthetic(metadata.synthetic);
                    self.surface.dispatch_window_input_pump_event(
                        &mut manager,
                        UiWindowInputPumpEvent::Window(UiWindowEvent::window_focused(
                            window_metadata,
                            true,
                        )),
                    )?;
                }
                self.surface.dispatch_input_event(
                    &UiPointerDispatcher::default(),
                    &UiNavigationDispatcher::default(),
                    input,
                )?
            }
            UiWindowInputPumpEvent::Window(window) => {
                if matches!(
                    &window.kind,
                    UiWindowEventKind::Focused { focused: false }
                        | UiWindowEventKind::Closed
                        | UiWindowEventKind::Destroyed
                ) {
                    // Keep text focus, composition and input-method ownership in the existing Surface.
                    self.surface.clear_focus();
                }
                if matches!(
                    &window.kind,
                    UiWindowEventKind::Focused { focused: false }
                        | UiWindowEventKind::Closed
                        | UiWindowEventKind::Destroyed
                ) {
                    self.pointer_source = None;
                }
                self.surface.dispatch_window_input_pump_event(
                    &mut manager,
                    UiWindowInputPumpEvent::Window(window),
                )?
            }
        };
        self.surface.rebuild_dirty(self.preview_size)?;
        self.input_feedback = if !result.rejected_effects.is_empty() {
            "Preview input rejected"
        } else if result.widget_events.is_empty() && result.component_events.is_empty() {
            "Preview input received"
        } else {
            "Preview updated"
        }
        .to_string();
        self.update_pointer_source(source_frame, &result.event);
        self.last_input_result = Some(Arc::new(result));
        Ok(true)
    }

    pub(crate) fn deactivate_input(
        &mut self,
        metadata: UiWindowEventMetadata,
    ) -> Result<bool, UiAssetEditorSessionError> {
        // Lifecycle input uses the current owner frame, even after its native pane disappears.
        let current = self.surface.surface_frame();
        self.dispatch_input(
            &current,
            UiWindowInputPumpEvent::Window(UiWindowEvent::window_focused(metadata, false)),
        )
    }

    fn accepts_captured_pointer_source(
        &self,
        source: &Arc<UiSurfaceFrame>,
        event: &UiInputEvent,
    ) -> bool {
        use zircon_runtime_interface::ui::surface::UiPointerEventKind;
        let UiInputEvent::Pointer(pointer) = event else {
            return false;
        };
        if !matches!(
            pointer.event.kind,
            UiPointerEventKind::Move | UiPointerEventKind::Up | UiPointerEventKind::Cancel
        ) {
            return false;
        }
        let Some(proof) = &self.pointer_source else {
            return false;
        };
        let current = self.surface.surface_frame();
        let known_frame = [proof.initial.upgrade(), proof.latest.upgrade()]
            .into_iter()
            .flatten()
            .any(|frame| Arc::ptr_eq(&frame, source));
        known_frame
            && source.tree_id == current.tree_id
            && proof.metadata.pointer_id == pointer.metadata.pointer_id
            && proof.metadata.window_id == pointer.metadata.window_id
            && proof.builtin_reference == self.builtin_reference
            && self.surface.tree.roots.contains(&proof.root)
            && self.surface.tree.node_incarnation(proof.root) == Some(proof.root_incarnation)
            && self.surface.tree.node_incarnation(proof.node) == Some(proof.incarnation)
            && (current.focus_state.captured == Some(proof.node)
                || current.focus_state.pressed == Some(proof.node))
            && self
                .surface
                .tree
                .node(proof.node)
                .is_some_and(|node| node.state_flags.enabled && node.is_render_visible())
    }

    fn update_pointer_source(&mut self, source: &Arc<UiSurfaceFrame>, event: &UiInputEvent) {
        use zircon_runtime_interface::ui::surface::UiPointerEventKind;
        let UiInputEvent::Pointer(pointer) = event else {
            return;
        };
        match pointer.event.kind {
            UiPointerEventKind::Down => {
                let current = self.surface.surface_frame();
                self.pointer_source = current
                    .focus_state
                    .captured
                    .or(current.focus_state.pressed)
                    .and_then(|node| {
                        let root = *self.surface.tree.roots.first()?;
                        Some(PreviewPointerSource {
                            initial: Arc::downgrade(source),
                            latest: Arc::downgrade(source),
                            node,
                            incarnation: self.surface.tree.node_incarnation(node)?,
                            root,
                            root_incarnation: self.surface.tree.node_incarnation(root)?,
                            builtin_reference: self.builtin_reference.clone(),
                            metadata: pointer.metadata.clone(),
                        })
                    });
            }
            UiPointerEventKind::Move => {
                if let Some(proof) = &mut self.pointer_source {
                    if proof.metadata.pointer_id == pointer.metadata.pointer_id
                        && proof.metadata.window_id == pointer.metadata.window_id
                    {
                        proof.latest = Arc::downgrade(source);
                    }
                }
            }
            UiPointerEventKind::Up | UiPointerEventKind::Cancel => {
                let current = self.surface.surface_frame();
                let retained = self.pointer_source.as_ref().is_some_and(|proof| {
                    current.focus_state.captured == Some(proof.node)
                        || current.focus_state.pressed == Some(proof.node)
                });
                if retained {
                    if let Some(proof) = &mut self.pointer_source {
                        proof.latest = Arc::downgrade(source);
                    }
                } else {
                    self.pointer_source = None;
                }
            }
            _ => {}
        }
    }

    pub fn rebuild(
        &mut self,
        asset_id: &str,
        compiled: &UiCompiledDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        *self = Self::new(self.preview_size, asset_id, compiled)?;
        Ok(())
    }

    pub fn rebuild_with_size(
        &mut self,
        preview_size: UiSize,
        _asset_id: &str,
        _compiled: &UiCompiledDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        if self.preview_size == preview_size {
            return Ok(());
        }

        self.resize(preview_size)
    }

    pub(crate) fn resize(&mut self, preview_size: UiSize) -> Result<(), UiAssetEditorSessionError> {
        if self.preview_size == preview_size {
            return Ok(());
        }
        self.visibility_state = None;
        self.preview_size = preview_size;
        for root_index in 0..self.surface.tree.roots.len() {
            let root_id = self.surface.tree.roots[root_index];
            if let Some(root) = self.surface.tree.node_mut(root_id) {
                root.dirty.layout = true;
                root.dirty.hit_test = true;
                root.dirty.render = true;
            }
        }
        self.surface.rebuild_dirty(preview_size)?;
        Ok(())
    }

    pub fn surface(&self) -> &UiSurface {
        &self.surface
    }

    #[cfg(test)]
    pub(crate) fn surface_mut(&mut self) -> &mut UiSurface {
        &mut self.surface
    }

    pub fn preview_size(&self) -> UiSize {
        self.preview_size
    }
}

fn builtin_prop_default(default: &crate::ui::template::EditorPropDefault) -> Option<toml::Value> {
    use crate::ui::template::{EditorPropDefault, EditorPropLiteral};
    Some(match default {
        EditorPropDefault::None => return None,
        EditorPropDefault::Token(token) => toml::Value::String(token.clone()),
        EditorPropDefault::Literal(literal) => match literal {
            EditorPropLiteral::Text(value) => toml::Value::String(value.clone()),
            EditorPropLiteral::Boolean(value) => toml::Value::Boolean(*value),
            EditorPropLiteral::Integer(value) => toml::Value::Integer(*value),
            EditorPropLiteral::Float(value) => toml::Value::Float(*value),
            EditorPropLiteral::TextList(values) => {
                toml::Value::Array(values.iter().cloned().map(toml::Value::String).collect())
            }
        },
    })
}

#[derive(Debug)]
struct PreviewPointerSource {
    initial: Weak<UiSurfaceFrame>,
    latest: Weak<UiSurfaceFrame>,
    node: zircon_runtime_interface::ui::event_ui::UiNodeId,
    incarnation: u64,
    root: zircon_runtime_interface::ui::event_ui::UiNodeId,
    root_incarnation: u64,
    builtin_reference: Option<String>,
    metadata: zircon_runtime_interface::ui::dispatch::UiInputEventMetadata,
}

impl PartialEq for PreviewPointerSource {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.initial, &other.initial)
            && Weak::ptr_eq(&self.latest, &other.latest)
            && self.node == other.node
            && self.incarnation == other.incarnation
            && self.root == other.root
            && self.root_incarnation == other.root_incarnation
            && self.builtin_reference == other.builtin_reference
            && self.metadata == other.metadata
    }
}

impl Clone for UiAssetPreviewHost {
    fn clone(&self) -> Self {
        Self {
            preview_size: self.preview_size,
            surface: self.surface.clone(),
            builtin_reference: self.builtin_reference.clone(),
            source_receipts: Arc::clone(&self.source_receipts),
            input_feedback: self.input_feedback.clone(),
            last_input_result: self.last_input_result.clone(),
            // Clones inherit no permission to admit an old frame's captured continuation.
            pointer_source: None,
            // A clone can share a cached frame, but never an active property gesture lease.
            visibility_state: None,
        }
    }
}
