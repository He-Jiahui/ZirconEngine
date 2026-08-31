use std::time::Duration;

use zircon_runtime::ui::{
    dispatch::{UiInputManager, DEFAULT_TOOLTIP_DELAY_MS},
    tree::UiRuntimeTreeLayoutExt,
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{UiInputTimestamp, UiPointerInputEvent},
    event_ui::UiNodeId,
    layout::{Position, StretchMode, UiFrame},
    surface::UiPointerEventKind,
    tree::UiTemplateNodeMetadata,
};

use crate::ui::retained_host::host_contract::{
    current_host_metrics, measure_runtime_text_width, FrameRect, WorkbenchTooltipPointerTarget,
};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;
use crate::ui::template_runtime::workbench_icon_tooltip_text as icon_tooltip_text;

const WORKBENCH_ICON_TOOLTIP_CONTROL_ID: &str = "WorkbenchIconTooltip";
const WORKBENCH_HOST_CHROME_TOOLTIP_ANCHOR_CONTROL_ID: &str = "WorkbenchHostChromeTooltipAnchor";
const TOOLTIP_ID_PREFIX: &str = "editor.workbench.icon.";
const OPEN: &str = "open";
const POPUP_OPEN: &str = "popup_open";
const TEXT: &str = "text";
const LABEL_TEXT: &str = "label_text";
const LAYOUT_MIN_WIDTH: &str = "layout_min_width";
const TOOLTIP_WRAP_WIDTH: &str = "tooltip_wrap_width";
const TRANSITION_PROGRESS: &str = "transition_progress";
const TRANSITION_STATUS: &str = "transition_status";
const FALLBACK_TOOLTIP_MIN_WIDTH: f32 = 96.0;
const UNREAL_TOOLTIP_WRAP_WIDTH: f32 = 1_000.0;

#[derive(Default)]
pub(super) struct WorkbenchIconTooltipInputState {
    manager: UiInputManager,
    candidate: Option<IconTooltipTarget>,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn update_workbench_icon_tooltip_candidate(
        &mut self,
        input: UiPointerInputEvent,
        target: Option<WorkbenchTooltipPointerTarget>,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let candidate = if matches!(input.event.kind, UiPointerEventKind::Move) {
            match target {
                Some(WorkbenchTooltipPointerTarget::SurfaceNode(node_id)) => {
                    let candidate = self.icon_tooltip_target_at_node(node_id);
                    if candidate.as_ref().is_some_and(|candidate| {
                        self.icon_tooltip_input
                            .candidate
                            .as_ref()
                            .is_some_and(|current| candidate.matches_owned(current))
                    }) {
                        return Ok(false);
                    }
                    candidate.map(IconTooltipTargetRef::into_owned)
                }
                Some(WorkbenchTooltipPointerTarget::HostChrome(target)) => self
                    .host_chrome_icon_tooltip_target(target.identity, target.label, target.frame),
                None => None,
            }
        } else {
            None
        };
        if candidate.is_none() && self.icon_tooltip_input.candidate.is_none() {
            return Ok(false);
        }

        if candidate.as_ref().is_some_and(|candidate| {
            self.icon_tooltip_input
                .candidate
                .as_ref()
                .is_some_and(|current| candidate.matches_identity(current))
        }) {
            let changed = candidate
                .as_ref()
                .and_then(|candidate| candidate.anchor_frame)
                .map(|frame| self.apply_host_chrome_tooltip_anchor_frame(frame))
                .transpose()?
                .unwrap_or(false);
            self.icon_tooltip_input.candidate = candidate;
            return Ok(changed);
        }

        let anchor_changed = candidate
            .as_ref()
            .and_then(|candidate| candidate.anchor_frame)
            .map(|frame| self.apply_host_chrome_tooltip_anchor_frame(frame))
            .transpose()?
            .unwrap_or(false);
        let hidden = self.hide_workbench_icon_tooltip()?;
        match candidate.as_ref() {
            Some(candidate) => {
                self.icon_tooltip_input.manager.arm_tooltip_candidate(
                    &mut self.template_surface.surface,
                    input.metadata.timestamp,
                    candidate.owner,
                    candidate.tooltip_id(),
                    candidate.delay_ms,
                );
            }
            None => self
                .icon_tooltip_input
                .manager
                .dismiss_tooltip(&mut self.template_surface.surface),
        }
        self.icon_tooltip_input.candidate = candidate;
        Ok(hidden || anchor_changed)
    }

    pub(crate) fn next_workbench_icon_tooltip_delay(
        &self,
        now: UiInputTimestamp,
    ) -> Option<Duration> {
        self.icon_tooltip_input
            .manager
            .next_frame_visible_delay(now)
    }

    pub(crate) fn tick_workbench_icon_tooltip(
        &mut self,
        now: UiInputTimestamp,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let _ = self
            .icon_tooltip_input
            .manager
            .tick(&mut self.template_surface.surface, now)?;
        let intro_progress = self
            .icon_tooltip_input
            .manager
            .tooltip_intro_progress(now)
            .unwrap_or(1.0);
        let candidate = self.icon_tooltip_input.candidate.take();
        let update_result: Result<bool, BuiltinHostWindowTemplateBridgeError> = (|| {
            let runtime_tooltip_is_visible = candidate.as_ref().is_some_and(|candidate| {
                self.template_surface
                    .surface
                    .input
                    .tooltip
                    .as_ref()
                    .is_some_and(|tooltip| {
                        tooltip.visible
                            && tooltip.owner == Some(candidate.owner)
                            && tooltip.tooltip_id.strip_prefix(TOOLTIP_ID_PREFIX)
                                == Some(candidate.identity.as_str())
                    })
            });
            let mut changed = match (runtime_tooltip_is_visible, candidate.as_ref()) {
                (true, Some(candidate)) => self.show_workbench_icon_tooltip(candidate)?,
                _ => false,
            };
            if runtime_tooltip_is_visible {
                changed |= self.apply_workbench_icon_tooltip_intro(intro_progress)?;
            }
            Ok(changed)
        })();
        self.icon_tooltip_input.candidate = candidate;
        let changed = update_result?;
        Ok(changed)
    }

    pub(crate) fn dismiss_workbench_icon_tooltip(
        &mut self,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        self.icon_tooltip_input.candidate = None;
        self.icon_tooltip_input
            .manager
            .dismiss_tooltip(&mut self.template_surface.surface);
        let changed = self.hide_workbench_icon_tooltip()?;
        Ok(changed)
    }

    fn icon_tooltip_target_at_node(
        &self,
        surface_node_id: UiNodeId,
    ) -> Option<IconTooltipTargetRef<'_>> {
        let mut node_id = Some(surface_node_id);
        while let Some(current_id) = node_id {
            if let Some(target) = self.icon_tooltip_target(current_id) {
                return Some(target);
            }
            node_id = self
                .template_surface
                .surface
                .tree
                .nodes
                .get(&current_id)?
                .parent;
        }
        None
    }

    fn icon_tooltip_target(&self, node_id: UiNodeId) -> Option<IconTooltipTargetRef<'_>> {
        let node = self.template_surface.surface.tree.nodes.get(&node_id)?;
        let metadata = node.template_metadata.as_ref()?;
        let label = icon_tooltip_text(metadata)?;
        let control_id = metadata.control_id.as_deref()?;
        let delay_ms =
            metadata_u64(metadata, "tooltip_delay_ms").unwrap_or(DEFAULT_TOOLTIP_DELAY_MS);
        Some(IconTooltipTargetRef {
            owner: node_id,
            label,
            control_id,
            identity: control_id,
            delay_ms,
        })
    }

    fn host_chrome_icon_tooltip_target(
        &self,
        identity: String,
        label: String,
        frame: FrameRect,
    ) -> Option<IconTooltipTarget> {
        let owner = self.control_node_id(WORKBENCH_HOST_CHROME_TOOLTIP_ANCHOR_CONTROL_ID)?;
        let anchor_frame = logical_host_chrome_tooltip_frame(
            &frame,
            self.mount_frame,
            self.presentation_scale_factor,
        )?;
        Some(IconTooltipTarget {
            owner,
            label,
            control_id: WORKBENCH_HOST_CHROME_TOOLTIP_ANCHOR_CONTROL_ID.to_string(),
            identity,
            delay_ms: DEFAULT_TOOLTIP_DELAY_MS,
            anchor_frame: Some(anchor_frame),
        })
    }

    fn apply_host_chrome_tooltip_anchor_frame(
        &mut self,
        frame: UiFrame,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let Some(node_id) = self.control_node_id(WORKBENCH_HOST_CHROME_TOOLTIP_ANCHOR_CONTROL_ID)
        else {
            return Ok(false);
        };
        let changed = {
            let Some(node) = self.template_surface.surface.tree.node_mut(node_id) else {
                return Ok(false);
            };
            let position = Position::new(frame.x, frame.y);
            let mut width = node.constraints.width;
            width.min = frame.width;
            width.preferred = frame.width;
            width.max = frame.width;
            width.stretch_mode = StretchMode::Fixed;
            let mut height = node.constraints.height;
            height.min = frame.height;
            height.preferred = frame.height;
            height.max = frame.height;
            height.stretch_mode = StretchMode::Fixed;
            let changed = node.position != position
                || node.constraints.width != width
                || node.constraints.height != height;
            node.position = position;
            node.constraints.width = width;
            node.constraints.height = height;
            changed
        };
        if changed {
            self.template_surface
                .surface
                .tree
                .mark_layout_dirty(node_id)?;
        }
        Ok(changed)
    }

    fn show_workbench_icon_tooltip(
        &mut self,
        target: &IconTooltipTarget,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.has_control(WORKBENCH_ICON_TOOLTIP_CONTROL_ID) {
            return Ok(false);
        }

        let anchor_is_current = self
            .control_node_id(WORKBENCH_ICON_TOOLTIP_CONTROL_ID)
            .and_then(|node_id| self.template_surface.surface.tree.nodes.get(&node_id))
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.widget.popup_anchor.control_id())
            == Some(target.control_id.as_str());
        let is_current = self.control_bool(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, POPUP_OPEN)
            && self
                .control_string(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, TEXT)
                .as_deref()
                == Some(target.label.as_str())
            && self
                .control_string(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, LABEL_TEXT)
                .as_deref()
                == Some("")
            && anchor_is_current;
        if is_current {
            return self.apply_workbench_icon_tooltip_extent(target.label.as_str());
        }

        let Some(tooltip_node_id) = self.control_node_id(WORKBENCH_ICON_TOOLTIP_CONTROL_ID) else {
            return Ok(false);
        };
        if !self
            .template_surface
            .surface
            .set_popup_control_anchor(tooltip_node_id, target.control_id.clone())
            .map_err(
                |source| BuiltinHostWindowTemplateBridgeError::LayoutMutation {
                    node_id: tooltip_node_id,
                    property: "widget.popup_anchor".to_string(),
                    source,
                },
            )?
            && !anchor_is_current
        {
            return Ok(false);
        }
        let _ = self.apply_workbench_icon_tooltip_extent(target.label.as_str())?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            TEXT,
            UiValue::String(target.label.clone()),
        )?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            LABEL_TEXT,
            UiValue::String(String::new()),
        )?;
        self.mutate_control_property(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, OPEN, UiValue::Bool(true))?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            POPUP_OPEN,
            UiValue::Bool(true),
        )?;
        self.set_visible(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, true)?;
        Ok(true)
    }

    fn apply_workbench_icon_tooltip_extent(
        &mut self,
        text: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let scale_factor = normalized_tooltip_scale_factor(self.presentation_scale_factor);
        let metrics = current_host_metrics();
        let logical_shell_width = (self.mount_frame.width / scale_factor).max(1.0);
        let edge_inset = metrics.gap_m / scale_factor;
        let available_width = (logical_shell_width - edge_inset * 2.0).max(1.0);
        let minimum_width = self
            .control_float(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, LAYOUT_MIN_WIDTH)
            .unwrap_or(FALLBACK_TOOLTIP_MIN_WIDTH)
            .max(1.0);
        let wrap_width = self
            .control_float(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, TOOLTIP_WRAP_WIDTH)
            .unwrap_or(UNREAL_TOOLTIP_WRAP_WIDTH)
            .max(1.0)
            .min(available_width);
        let title_font_size = metrics.font_body + metrics.border_width * 2.0;
        let measured_text_width = measure_runtime_text_width(text, title_font_size) / scale_factor;
        let horizontal_padding = (metrics.gap_m * 2.0 + metrics.text_clip_guard) / scale_factor;
        let maximum_width = wrap_width.max(1.0);
        let minimum_width = minimum_width.min(maximum_width);
        let width = (measured_text_width + horizontal_padding).clamp(minimum_width, maximum_width);

        let Some(node_id) = self.control_node_id(WORKBENCH_ICON_TOOLTIP_CONTROL_ID) else {
            return Ok(false);
        };
        let changed = {
            let Some(node) = self.template_surface.surface.tree.node_mut(node_id) else {
                return Ok(false);
            };
            let mut next_width = node.constraints.width;
            next_width.min = width;
            next_width.preferred = width;
            next_width.max = width;
            next_width.stretch_mode = StretchMode::Fixed;
            let changed = node.constraints.width != next_width;
            node.constraints.width = next_width;
            changed
        };
        if changed {
            self.template_surface
                .surface
                .tree
                .mark_layout_dirty(node_id)?;
        }
        Ok(changed)
    }

    fn apply_workbench_icon_tooltip_intro(
        &mut self,
        progress: f32,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let progress = progress.clamp(0.0, 1.0);
        let status = if progress >= 1.0 {
            "entered"
        } else {
            "entering"
        };
        let progress_changed = self
            .control_float(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, TRANSITION_PROGRESS)
            .is_none_or(|current| (current - progress).abs() > f32::EPSILON);
        let status_changed = self
            .control_string(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, TRANSITION_STATUS)
            .as_deref()
            != Some(status);
        if !progress_changed && !status_changed {
            return Ok(false);
        }
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            TRANSITION_PROGRESS,
            UiValue::Float(progress as f64),
        )?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            TRANSITION_STATUS,
            UiValue::String(status.to_string()),
        )?;
        Ok(true)
    }

    fn hide_workbench_icon_tooltip(
        &mut self,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.control_bool(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, POPUP_OPEN) {
            return Ok(false);
        }

        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            OPEN,
            UiValue::Bool(false),
        )?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            POPUP_OPEN,
            UiValue::Bool(false),
        )?;
        self.set_visible(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, false)?;
        Ok(true)
    }
}

fn normalized_tooltip_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > f32::EPSILON {
        scale_factor
    } else {
        1.0
    }
}

fn logical_host_chrome_tooltip_frame(
    frame: &FrameRect,
    mount_frame: UiFrame,
    scale_factor: f32,
) -> Option<UiFrame> {
    if !frame.x.is_finite()
        || !frame.y.is_finite()
        || !frame.width.is_finite()
        || !frame.height.is_finite()
        || frame.width <= 0.0
        || frame.height <= 0.0
    {
        return None;
    }
    let scale_factor = normalized_tooltip_scale_factor(scale_factor);
    Some(UiFrame::new(
        super::componentized_window::logical_axis_from_physical(
            frame.x,
            mount_frame.x,
            scale_factor,
        ),
        super::componentized_window::logical_axis_from_physical(
            frame.y,
            mount_frame.y,
            scale_factor,
        ),
        frame.width / scale_factor,
        frame.height / scale_factor,
    ))
}

#[derive(Clone, Debug, PartialEq)]
struct IconTooltipTarget {
    owner: UiNodeId,
    label: String,
    control_id: String,
    identity: String,
    delay_ms: u64,
    anchor_frame: Option<UiFrame>,
}

struct IconTooltipTargetRef<'a> {
    owner: UiNodeId,
    label: &'a str,
    control_id: &'a str,
    identity: &'a str,
    delay_ms: u64,
}

impl IconTooltipTargetRef<'_> {
    fn matches_owned(&self, target: &IconTooltipTarget) -> bool {
        self.owner == target.owner
            && self.label == target.label
            && self.control_id == target.control_id
            && self.identity == target.identity
            && self.delay_ms == target.delay_ms
            && target.anchor_frame.is_none()
    }

    fn into_owned(self) -> IconTooltipTarget {
        IconTooltipTarget {
            owner: self.owner,
            label: self.label.to_owned(),
            control_id: self.control_id.to_owned(),
            identity: self.identity.to_owned(),
            delay_ms: self.delay_ms,
            anchor_frame: None,
        }
    }
}

impl IconTooltipTarget {
    fn matches_identity(&self, target: &Self) -> bool {
        self.owner == target.owner
            && self.label == target.label
            && self.control_id == target.control_id
            && self.identity == target.identity
            && self.delay_ms == target.delay_ms
    }

    fn tooltip_id(&self) -> String {
        format!("{TOOLTIP_ID_PREFIX}{}", self.identity)
    }
}

fn metadata_u64(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<u64> {
    match metadata.attributes.get(key)? {
        toml::Value::Integer(value) => u64::try_from(*value).ok(),
        toml::Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as u64),
        toml::Value::String(value) => value.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::HostChromeTooltipTarget;
    use zircon_runtime_interface::ui::dispatch::{
        UiInputEventMetadata, UiInputSequence, UiPointerEvent,
    };
    use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};
    use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

    #[test]
    fn disabled_control_keeps_its_explicit_tooltip_reason() {
        let mut metadata = UiTemplateNodeMetadata::default();
        metadata
            .attributes
            .insert("disabled".to_string(), toml::Value::Boolean(true));
        metadata.attributes.insert(
            "tooltip".to_string(),
            toml::Value::String("Save is unavailable until the project is valid".to_string()),
        );

        assert_eq!(
            icon_tooltip_text(&metadata),
            Some("Save is unavailable until the project is valid")
        );
    }

    #[test]
    fn host_chrome_anchor_crosses_the_physical_boundary_once() {
        let frame = logical_host_chrome_tooltip_frame(
            &FrameRect {
                x: 148.0,
                y: 92.0,
                width: 200.0,
                height: 48.0,
            },
            UiFrame::new(100.0, 60.0, 800.0, 600.0),
            2.0,
        )
        .unwrap();

        assert_eq!(frame, UiFrame::new(24.0, 16.0, 100.0, 24.0));
    }

    #[test]
    fn host_chrome_tab_uses_the_runtime_delay_and_real_control_anchor() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(800.0, 600.0)).unwrap();
        let input = UiPointerInputEvent {
            metadata: UiInputEventMetadata::new(
                UiInputTimestamp::from_micros(0),
                UiInputSequence::new(1),
            ),
            event: UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(140.0, 72.0)),
            precise_scroll: None,
        };

        assert!(bridge
            .update_workbench_icon_tooltip_candidate(
                input,
                Some(WorkbenchTooltipPointerTarget::HostChrome(
                    HostChromeTooltipTarget {
                        identity: "DocumentSceneTab".into(),
                        label: "Scene".into(),
                        frame: FrameRect {
                            x: 100.0,
                            y: 60.0,
                            width: 96.0,
                            height: 28.0,
                        },
                    },
                )),
            )
            .unwrap());
        bridge.refresh_prepared_state_change().unwrap();
        assert_eq!(
            bridge
                .control_frame(WORKBENCH_HOST_CHROME_TOOLTIP_ANCHOR_CONTROL_ID)
                .unwrap(),
            UiFrame::new(100.0, 60.0, 96.0, 28.0)
        );
        assert!(!bridge
            .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(149_999))
            .unwrap());
        assert!(bridge
            .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(150_000))
            .unwrap());

        let popup_anchor = bridge
            .control_node_id(WORKBENCH_ICON_TOOLTIP_CONTROL_ID)
            .and_then(|node_id| bridge.surface().tree.nodes.get(&node_id))
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.widget.popup_anchor.control_id());
        assert_eq!(
            popup_anchor,
            Some(WORKBENCH_HOST_CHROME_TOOLTIP_ANCHOR_CONTROL_ID)
        );
    }
}
