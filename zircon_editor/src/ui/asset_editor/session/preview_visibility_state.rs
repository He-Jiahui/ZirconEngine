use std::sync::Arc;

use zircon_runtime::core::{framework::animation::AnimationChannelAsset, math::Real};
use zircon_runtime_interface::ui::{
    surface::UiSurfaceFrame,
    v2::{UiV2AssetDocument, UiV2AssetKind},
};

use super::{
    preview_host::visibility::{
        UiPreviewVisibilityError as Error, UiPreviewVisibilityGesture,
        UiPreviewVisibilityOutcome as Outcome, UiPreviewVisibilityResetError,
        UiPreviewVisibilitySample, UiPreviewVisibilitySource, UiPreviewVisibilityTarget,
    },
    ui_asset_editor_session::UiAssetEditorSession,
    v2_authoring::build_v2_preview_host,
};

impl UiAssetEditorSession {
    pub(crate) fn begin_preview_visibility(
        &mut self,
        source_node_id: &str,
    ) -> Result<UiPreviewVisibilityGesture, Error> {
        let source = self.visibility_source()?;
        let document = self.visibility_document()?;
        let root = document.root_node_id().ok_or(Error::UnsupportedDocument)?;
        let parent = document.nodes.get(root).ok_or(Error::UnsupportedDocument)?;
        let node = document
            .nodes
            .get(source_node_id)
            .ok_or(Error::UnsupportedTarget)?;
        if source_node_id == root
            || node.component != "Label"
            || !node.children.is_empty()
            || !parent
                .children
                .iter()
                .any(|child| child.node == source_node_id)
            || !node
                .props
                .get("text")
                .and_then(toml::Value::as_str)
                .is_some_and(|s| !s.is_empty())
        {
            return Err(Error::UnsupportedTarget);
        }
        let target = UiPreviewVisibilityTarget {
            source_path: source.document_uri.clone(),
            root_node_id: root.to_string(),
            source_node_id: source_node_id.to_string(),
            instance_path: Vec::new(),
        };
        self.preview_host
            .as_mut()
            .ok_or(Error::PreviewUnavailable)?
            .begin_visibility_gesture(source, target)
    }

    pub(crate) fn apply_preview_visibility(
        &mut self,
        gesture: &UiPreviewVisibilityGesture,
        frame: &Arc<UiSurfaceFrame>,
        sequence: u64,
        visible: bool,
    ) -> Outcome {
        self.preview_visibility_request(gesture, frame, sequence, Some(visible))
    }

    /// Consume the existing pure sampler at caller-supplied seconds, without owning a clock.
    /// The result retains the real property/publication outcome; it never confirms a clock/present.
    pub(crate) fn sample_preview_visibility(
        &mut self,
        gesture: &UiPreviewVisibilityGesture,
        frame: &Arc<UiSurfaceFrame>,
        sequence: u64,
        channel: &AnimationChannelAsset,
        requested_seconds: Real,
    ) -> Result<UiPreviewVisibilitySample, Error> {
        self.check_preview_visibility_source(gesture)?;
        let sampled_visible =
            zircon_runtime::animation::sample_animation_bool_channel(channel, requested_seconds)
                .ok_or(Error::UnsupportedChannelSample)?;
        let outcome = self.apply_preview_visibility(gesture, frame, sequence, sampled_visible);
        Ok(UiPreviewVisibilitySample {
            requested_seconds,
            sampled_visible,
            outcome,
        })
    }

    /// Keep the current value and retire a verified lease, without source/history mutation.
    pub(crate) fn finish_preview_visibility(
        &mut self,
        gesture: &UiPreviewVisibilityGesture,
        frame: &Arc<UiSurfaceFrame>,
        sequence: u64,
    ) -> Result<Arc<UiSurfaceFrame>, Error> {
        self.check_preview_visibility_source(gesture)?;
        self.preview_host
            .as_mut()
            .ok_or(Error::PreviewUnavailable)?
            .finish_visibility_gesture(gesture, frame, sequence)
    }

    pub(crate) fn cancel_preview_visibility(
        &mut self,
        gesture: &UiPreviewVisibilityGesture,
        frame: &Arc<UiSurfaceFrame>,
        sequence: u64,
    ) -> Outcome {
        self.preview_visibility_request(gesture, frame, sequence, None)
    }

    /// Rebuild the existing root PreviewHost from the current valid source, not an old baseline.
    pub(crate) fn reset_preview_visibility(
        &mut self,
    ) -> Result<Arc<UiSurfaceFrame>, UiPreviewVisibilityResetError> {
        if let Some(host) = self.preview_host.as_mut() {
            host.retire_visibility_gestures();
            if host.builtin_reference().is_some() {
                return Err(Error::UnsupportedTarget.into());
            }
        }
        self.visibility_source()?;
        let document = self.visibility_document()?;
        let size = self
            .preview_host
            .as_ref()
            .ok_or(Error::PreviewUnavailable)?
            .preview_size();
        let next = build_v2_preview_host(
            document,
            size,
            &self.v2_compiler_imports,
            &self.route.asset_id,
        )?
        .ok_or(Error::PreviewUnavailable)?;
        let frame = next.surface().surface_frame();
        self.preview_host = Some(next);
        self.invalidate_preview_hit_index();
        Ok(frame)
    }

    fn preview_visibility_request(
        &mut self,
        gesture: &UiPreviewVisibilityGesture,
        frame: &Arc<UiSurfaceFrame>,
        sequence: u64,
        visible: Option<bool>,
    ) -> Outcome {
        if let Err(error) = self.check_preview_visibility_source(gesture) {
            return Outcome::Rejected(error);
        }
        match self.preview_host.as_mut() {
            Some(host) => host.apply_visibility_gesture(gesture, frame, sequence, visible),
            None => Outcome::Rejected(Error::PreviewUnavailable),
        }
    }

    fn check_preview_visibility_source(
        &mut self,
        gesture: &UiPreviewVisibilityGesture,
    ) -> Result<(), Error> {
        let source_check = self.visibility_source().and_then(|source| {
            if &source == gesture.source() {
                self.visibility_document().map(|_| ())
            } else {
                Err(Error::StaleSource)
            }
        });
        if source_check.is_err() {
            if let Some(host) = self.preview_host.as_mut() {
                host.retire_visibility_gesture(gesture);
            }
        }
        source_check
    }

    fn visibility_source(&self) -> Result<UiPreviewVisibilitySource, Error> {
        if self.route.asset_id.trim().is_empty()
            || !self.diagnostics.is_empty()
            || !self.resource_diagnostics.is_empty()
            || self.source_buffer.text() != self.last_valid_source_text
        {
            return Err(Error::InvalidSource);
        }
        let document = self
            .last_valid_v2_document
            .as_ref()
            .ok_or(Error::InvalidSource)?;
        Ok(UiPreviewVisibilitySource {
            document_uri: self.route.asset_id.clone(),
            asset_id: document.asset.id.clone(),
            source_revision: self.source_buffer.revision(),
            last_valid_generation: self.last_valid_source_generation,
        })
    }

    fn visibility_document(&self) -> Result<&UiV2AssetDocument, Error> {
        let document = self
            .last_valid_v2_document
            .as_ref()
            .ok_or(Error::InvalidSource)?;
        if document.asset.kind != UiV2AssetKind::View
            || !document.imports.widgets.is_empty()
            || !document.imports.styles.is_empty()
            || !document.imports.resources.is_empty()
            || !self.v2_compiler_imports.widgets.is_empty()
            || !self.v2_compiler_imports.styles.is_empty()
            || !document.components.is_empty()
            || !document.stylesheets.is_empty()
            || !document.tokens.is_empty()
        {
            return Err(Error::UnsupportedDocument);
        }
        let root = document.root_node_id().ok_or(Error::UnsupportedDocument)?;
        if document
            .nodes
            .get(root)
            .is_none_or(|node| node.component != "Panel")
        {
            return Err(Error::UnsupportedDocument);
        }
        // Constrain the whole document, not just a field that appears unbound.
        if document.nodes.values().any(|node| {
            !matches!(node.component.as_str(), "Panel" | "Label")
                || node.repeat.is_some()
                || !node.events.is_empty()
                || !node.params.is_empty()
                || !node.slots.is_empty()
                || !node.state.is_empty()
                || !node.classes.is_empty()
                || !node.style.self_values.is_empty()
                || !node.style.slot.is_empty()
                || node.widget.is_some()
                || node.props.iter().any(|(key, value)| match key.as_str() {
                    "text" => node.component != "Label" || !value.is_str(),
                    "visible" => !value.is_bool(),
                    _ => true,
                })
        }) {
            return Err(Error::UnsupportedDocument);
        }
        Ok(document)
    }
}
