use zircon_editor::core::commands::EditorCommandDescriptor;
use zircon_editor::core::editor_authoring_extension::ViewportToolModeDescriptor;
use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorMenuItemDescriptor,
    EditorUiTemplateDescriptor,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_plugin_editor_support::{EditorAuthoringSurface, register_authoring_surface};
use zircon_runtime::core::framework::ai::{AiPerceptionSense, AiPerceptionStimulus};
use zircon_runtime::core::framework::render::{
    OverlayLineSegment, OverlayPickShape, SceneGizmoKind, SceneGizmoOverlayExtract,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::{Real, Vec3, Vec4};

use crate::capability::AI_DEBUG_CAPABILITY;
use crate::extension_ids::{
    AI_PERCEPTION_DEBUG_TEMPLATE_ID, AI_PERCEPTION_DEBUG_VIEW_ID, AI_PERCEPTION_OVERLAY_MODE_ID,
    AI_PERCEPTION_OVERLAY_PROVIDER_ID, AI_TOGGLE_PERCEPTION_OVERLAY_OPERATION,
};
use crate::runtime_mirror::AiPieMirror;

const FOV_COLOR: Vec4 = Vec4::new(0.2, 0.85, 1.0, 1.0);
const HEARING_COLOR: Vec4 = Vec4::new(1.0, 0.68, 0.18, 0.9);
const SIGHT_STIMULUS_COLOR: Vec4 = Vec4::new(0.35, 1.0, 0.52, 1.0);
const HEARING_STIMULUS_COLOR: Vec4 = Vec4::new(1.0, 0.72, 0.2, 1.0);
const OTHER_STIMULUS_COLOR: Vec4 = Vec4::new(0.95, 0.32, 0.76, 1.0);

pub trait AiPerceptionViewportGizmoSink {
    fn replace_ai_perception_overlay(&mut self, overlay: Option<SceneGizmoOverlayExtract>);
}

pub struct AiPerceptionOverlayController<S> {
    enabled: bool,
    options: AiPerceptionOverlayOptions,
    sink: S,
}

impl<S: AiPerceptionViewportGizmoSink> AiPerceptionOverlayController<S> {
    pub fn new(sink: S) -> Self {
        Self {
            enabled: false,
            options: AiPerceptionOverlayOptions::default(),
            sink,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.sink.replace_ai_perception_overlay(None);
        }
    }

    pub fn toggle(&mut self) -> bool {
        let enabled = !self.enabled;
        self.set_enabled(enabled);
        enabled
    }

    pub fn publish(&mut self, owner: EntityId, world: &WorldHandle, mirror: &AiPieMirror) -> bool {
        if !self.enabled {
            return false;
        }
        self.sink
            .replace_ai_perception_overlay(Some(build_ai_perception_overlay_with_options(
                owner,
                world,
                mirror,
                self.options,
            )));
        true
    }

    pub fn options_mut(&mut self) -> &mut AiPerceptionOverlayOptions {
        &mut self.options
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiPerceptionOverlayOptions {
    pub sight_cone: bool,
    pub hearing_radius: bool,
    pub stimuli: bool,
}

impl Default for AiPerceptionOverlayOptions {
    fn default() -> Self {
        Self {
            sight_cone: true,
            hearing_radius: true,
            stimuli: true,
        }
    }
}

pub(crate) fn register_ai_perception_overlay(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        AI_PERCEPTION_DEBUG_TEMPLATE_ID,
        "plugins://ai/editor/perception_debug.zui",
    ))?;
    register_authoring_surface(
        registry,
        EditorAuthoringSurface::new(
            AI_PERCEPTION_DEBUG_VIEW_ID,
            "AI Perception Debug",
            "AI",
            "Plugins/AI/Perception Debug",
        ),
    )?;
    let operation = EditorOperationPath::parse(AI_TOGGLE_PERCEPTION_OVERLAY_OPERATION)
        .map_err(EditorExtensionRegistryError::OperationPath)?;
    registry.register_command(
        EditorCommandDescriptor::operation(operation.clone(), "Toggle Perception Overlay")
            .with_menu_path("Plugins/AI/Toggle Perception Overlay")
            .with_required_capabilities([AI_DEBUG_CAPABILITY]),
    )?;
    registry.register_menu_item(
        EditorMenuItemDescriptor::new("Plugins/AI/Toggle Perception Overlay", operation.clone())
            .with_required_capabilities([AI_DEBUG_CAPABILITY]),
    )?;
    registry.register_viewport_tool_mode(
        ViewportToolModeDescriptor::new(
            AI_PERCEPTION_OVERLAY_MODE_ID,
            "AI Perception Overlay",
            AI_PERCEPTION_DEBUG_VIEW_ID,
            operation,
        )
        .with_overlay_provider_id(AI_PERCEPTION_OVERLAY_PROVIDER_ID)
        .with_required_capabilities([AI_DEBUG_CAPABILITY]),
    )
}

pub fn build_ai_perception_overlay(
    owner: EntityId,
    world: &WorldHandle,
    mirror: &AiPieMirror,
) -> SceneGizmoOverlayExtract {
    build_ai_perception_overlay_with_options(
        owner,
        world,
        mirror,
        AiPerceptionOverlayOptions::default(),
    )
}

fn build_ai_perception_overlay_with_options(
    owner: EntityId,
    world: &WorldHandle,
    mirror: &AiPieMirror,
    options: AiPerceptionOverlayOptions,
) -> SceneGizmoOverlayExtract {
    let mut overlay = SceneGizmoOverlayExtract::new(
        owner,
        SceneGizmoKind::AiPerception,
        false,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    for frame in mirror.agents_in_world(world) {
        let Some(debug) = frame.perception_debug.as_ref() else {
            continue;
        };
        if !debug.position.is_finite() {
            continue;
        }
        overlay.pick_shapes.push(OverlayPickShape::Sphere {
            center: debug.position,
            radius: 0.18,
        });
        if options.sight_cone {
            append_sight_cone(
                &mut overlay.lines,
                debug.position,
                debug.forward,
                debug.sight_fov_degrees,
                debug.sight_range,
            );
        }
        if options.hearing_radius {
            append_circle(
                &mut overlay.lines,
                debug.position,
                debug.hearing_radius,
                HEARING_COLOR,
            );
            if valid_radius(debug.hearing_radius) {
                overlay.pick_shapes.push(OverlayPickShape::Circle {
                    center: debug.position,
                    normal: Vec3::Y,
                    radius: debug.hearing_radius,
                    thickness: 0.08,
                });
            }
        }
        if options.stimuli {
            for stimulus in frame
                .perception
                .as_ref()
                .into_iter()
                .flat_map(|perception| perception.stimuli.iter())
            {
                append_stimulus(&mut overlay, debug.position, stimulus);
            }
        }
    }
    overlay
}

fn append_sight_cone(
    lines: &mut Vec<OverlayLineSegment>,
    origin: Vec3,
    forward: Vec3,
    fov_degrees: Real,
    range: Real,
) {
    if !valid_radius(range) || !fov_degrees.is_finite() {
        return;
    }
    let fov_degrees = fov_degrees.clamp(0.0, 360.0);
    if fov_degrees >= 359.9 {
        append_circle(lines, origin, range, FOV_COLOR);
        return;
    }
    let forward = horizontal_forward(forward);
    let start_angle = forward.x.atan2(forward.z) - fov_degrees.to_radians() * 0.5;
    let segment_count = ((fov_degrees / 15.0).ceil() as usize).clamp(1, 24);
    let first = origin + direction_from_yaw(start_angle) * range;
    lines.push(OverlayLineSegment {
        start: origin,
        end: first,
        color: FOV_COLOR,
    });
    let mut previous = first;
    for index in 1..=segment_count {
        let ratio = index as Real / segment_count as Real;
        let angle = start_angle + fov_degrees.to_radians() * ratio;
        let next = origin + direction_from_yaw(angle) * range;
        lines.push(OverlayLineSegment {
            start: previous,
            end: next,
            color: FOV_COLOR,
        });
        previous = next;
    }
    lines.push(OverlayLineSegment {
        start: previous,
        end: origin,
        color: FOV_COLOR,
    });
}

fn append_circle(lines: &mut Vec<OverlayLineSegment>, origin: Vec3, radius: Real, color: Vec4) {
    if !valid_radius(radius) {
        return;
    }
    const SEGMENTS: usize = 24;
    let mut previous = origin + Vec3::Z * radius;
    for index in 1..=SEGMENTS {
        let angle = std::f32::consts::TAU * index as Real / SEGMENTS as Real;
        let next = origin + direction_from_yaw(angle) * radius;
        lines.push(OverlayLineSegment {
            start: previous,
            end: next,
            color,
        });
        previous = next;
    }
}

fn append_stimulus(
    overlay: &mut SceneGizmoOverlayExtract,
    origin: Vec3,
    stimulus: &AiPerceptionStimulus,
) {
    if !stimulus.position.is_finite() {
        return;
    }
    let color = stimulus_color(stimulus.sense);
    overlay.lines.push(OverlayLineSegment {
        start: origin,
        end: stimulus.position,
        color,
    });
    overlay.pick_shapes.push(OverlayPickShape::Sphere {
        center: stimulus.position,
        radius: 0.13,
    });
}

fn horizontal_forward(forward: Vec3) -> Vec3 {
    let horizontal = Vec3::new(forward.x, 0.0, forward.z);
    if horizontal.length_squared() <= f32::EPSILON {
        Vec3::Z
    } else {
        horizontal.normalize()
    }
}

fn direction_from_yaw(yaw: Real) -> Vec3 {
    Vec3::new(yaw.sin(), 0.0, yaw.cos())
}

fn valid_radius(radius: Real) -> bool {
    radius.is_finite() && radius > 0.0
}

fn stimulus_color(sense: AiPerceptionSense) -> Vec4 {
    match sense {
        AiPerceptionSense::Sight => SIGHT_STIMULUS_COLOR,
        AiPerceptionSense::Hearing => HEARING_STIMULUS_COLOR,
        AiPerceptionSense::Damage | AiPerceptionSense::Touch | AiPerceptionSense::Custom => {
            OTHER_STIMULUS_COLOR
        }
    }
}
