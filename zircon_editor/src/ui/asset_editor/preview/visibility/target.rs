use std::sync::Arc;

use zircon_runtime_interface::ui::{event_ui::UiNodeId, tree::UiVisibility};

use super::super::UiAssetPreviewHost;
use super::{
    contract::PreviewVisibilityState, UiPreviewVisibilityError as Error,
    UiPreviewVisibilityGesture, UiPreviewVisibilitySource, UiPreviewVisibilityTarget,
};

pub(super) struct TargetState {
    pub node: UiNodeId,
    pub incarnation: u64,
    pub root: UiNodeId,
    pub root_incarnation: u64,
    pub visible: bool,
    pub attribute: Option<bool>,
}

impl UiAssetPreviewHost {
    pub(in crate::ui::asset_editor) fn begin_visibility_gesture(
        &mut self,
        source: UiPreviewVisibilitySource,
        target: UiPreviewVisibilityTarget,
    ) -> Result<UiPreviewVisibilityGesture, Error> {
        if let Some(state) = &self.visibility_state {
            if state.gesture.source == source {
                return Err(Error::GestureInProgress);
            }
            // A new source never inherits an old instance's restoration baseline.
            self.visibility_state = None;
        }
        let proof = self.visibility_target_state(&target)?;
        let gesture = UiPreviewVisibilityGesture {
            lease: Arc::new(()),
            source,
            target,
        };
        self.visibility_state = Some(PreviewVisibilityState {
            gesture: gesture.clone(),
            frame: self.surface.surface_frame(),
            node: proof.node,
            incarnation: proof.incarnation,
            root: proof.root,
            root_incarnation: proof.root_incarnation,
            baseline_visible: proof.visible,
            baseline_attribute: proof.attribute,
            current_visible: proof.visible,
            current_attribute: proof.attribute,
            last_sequence: 0,
        });
        Ok(gesture)
    }

    pub(in crate::ui::asset_editor) fn retire_visibility_gesture(
        &mut self,
        gesture: &UiPreviewVisibilityGesture,
    ) {
        if self
            .visibility_state
            .as_ref()
            .is_some_and(|state| Arc::ptr_eq(&state.gesture.lease, &gesture.lease))
        {
            self.visibility_state = None;
        }
    }

    pub(in crate::ui::asset_editor) fn retire_visibility_gestures(&mut self) {
        self.visibility_state = None;
    }

    pub(super) fn visibility_target_state(
        &self,
        target: &UiPreviewVisibilityTarget,
    ) -> Result<TargetState, Error> {
        if self.builtin_reference.is_some() || !target.instance_path.is_empty() {
            return Err(Error::UnsupportedTarget);
        }
        if self.surface.window_state.closed || self.surface.window_state.destroyed {
            return Err(Error::WindowUnavailable);
        }
        let [root_id] = self.surface.tree.roots.as_slice() else {
            return Err(Error::PreviewUnavailable);
        };
        let root = self
            .surface
            .tree
            .node(*root_id)
            .ok_or(Error::PreviewUnavailable)?;
        let root_metadata = root
            .template_metadata
            .as_ref()
            .ok_or(Error::PreviewUnavailable)?;
        if root_metadata.component != "Panel"
            || root_metadata.source_path.as_deref() != Some(target.source_path.as_str())
            || root_metadata.source_node_id.as_deref() != Some(target.root_node_id.as_str())
            || root_metadata.instance_path.as_deref() != Some(target.instance_path.as_slice())
            || !root.state_flags.enabled
            || !root.is_render_visible()
        {
            return Err(Error::UnsupportedTarget);
        }
        let mut matches = self.surface.tree.nodes.values().filter(|node| {
            node.template_metadata.as_ref().is_some_and(|metadata| {
                metadata.source_path.as_deref() == Some(target.source_path.as_str())
                    && metadata.source_node_id.as_deref() == Some(target.source_node_id.as_str())
                    && metadata.instance_path.as_deref() == Some(target.instance_path.as_slice())
            })
        });
        let node = matches.next().ok_or(Error::PreviewUnavailable)?;
        if matches.next().is_some() {
            return Err(Error::AmbiguousTarget);
        }
        let metadata = node
            .template_metadata
            .as_ref()
            .ok_or(Error::PreviewUnavailable)?;
        if metadata.component != "Label"
            || node.parent != Some(*root_id)
            || !node.children.is_empty()
            || !metadata.bindings.is_empty()
            || !metadata.localized_text_references.is_empty()
            || !node.state_flags.enabled
            || node.state_flags.clickable
            || node.state_flags.hoverable
            || node.state_flags.focusable
            || node.visibility != UiVisibility::Visible
            || !node.layout_cache.measure_valid
        {
            return Err(Error::UnsupportedTarget);
        }
        let attribute = match metadata.attributes.get("visible") {
            None => None,
            Some(toml::Value::Boolean(value)) if *value == node.state_flags.visible => Some(*value),
            _ => return Err(Error::TargetChanged),
        };
        Ok(TargetState {
            node: node.node_id,
            incarnation: self
                .surface
                .tree
                .node_incarnation(node.node_id)
                .ok_or(Error::TargetChanged)?,
            root: *root_id,
            root_incarnation: self
                .surface
                .tree
                .node_incarnation(*root_id)
                .ok_or(Error::TargetChanged)?,
            visible: node.state_flags.visible,
            attribute,
        })
    }
}
