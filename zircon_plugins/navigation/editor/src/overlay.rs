use zircon_editor::core::editor_authoring_extension::ViewportToolModeDescriptor;
use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_runtime::core::framework::navigation::NavigationGizmoSnapshot;
use zircon_runtime::core::framework::render::{
    OverlayLineSegment, OverlayPickShape, SceneGizmoOverlayExtract,
};
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::{Vec3, Vec4};

use crate::capability::NAVIGATION_GIZMOS_CAPABILITY;
use crate::extension_ids::{NAVIGATION_DEBUG_VIEW_ID, NAVIGATION_TOGGLE_GIZMOS_OPERATION};
use crate::runtime_mirror::NavigationPieFrame;

pub const NAVIGATION_OVERLAY_MODE_ID: &str = "navigation.viewport.overlay";
pub const NAVIGATION_OVERLAY_PROVIDER_ID: &str = "navigation.viewport.overlay.provider";

pub trait NavigationViewportGizmoSink {
    fn replace_navigation_overlay(&mut self, overlay: Option<SceneGizmoOverlayExtract>);
}

pub struct NavigationOverlayController<S> {
    enabled: bool,
    options: NavigationOverlayOptions,
    sink: S,
}

impl<S: NavigationViewportGizmoSink> NavigationOverlayController<S> {
    pub fn new(sink: S) -> Self {
        Self {
            enabled: false,
            options: NavigationOverlayOptions::default(),
            sink,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.sink.replace_navigation_overlay(None);
        }
    }

    pub fn toggle(&mut self) -> bool {
        let enabled = !self.enabled;
        self.set_enabled(enabled);
        enabled
    }

    pub fn publish(
        &mut self,
        owner: EntityId,
        nav_mesh: &NavigationGizmoSnapshot,
        pie_frame: Option<&NavigationPieFrame>,
    ) -> bool {
        if !self.enabled {
            return false;
        }
        let overlay = build_navigation_overlay(owner, nav_mesh, pie_frame, self.options);
        self.sink.replace_navigation_overlay(Some(overlay));
        true
    }

    pub fn options_mut(&mut self) -> &mut NavigationOverlayOptions {
        &mut self.options
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavigationOverlayOptions {
    pub nav_mesh_areas: bool,
    pub off_mesh_links: bool,
    pub agent_paths: bool,
    pub desired_velocity: bool,
    pub avoidance_velocity: bool,
}

impl Default for NavigationOverlayOptions {
    fn default() -> Self {
        Self {
            nav_mesh_areas: true,
            off_mesh_links: true,
            agent_paths: true,
            desired_velocity: true,
            avoidance_velocity: true,
        }
    }
}

pub(crate) fn register_navigation_overlay(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    let operation = EditorOperationPath::parse(NAVIGATION_TOGGLE_GIZMOS_OPERATION)
        .map_err(EditorExtensionRegistryError::OperationPath)?;
    registry.register_viewport_tool_mode(
        ViewportToolModeDescriptor::new(
            NAVIGATION_OVERLAY_MODE_ID,
            "Navigation Overlay",
            NAVIGATION_DEBUG_VIEW_ID,
            operation,
        )
        .with_overlay_provider_id(NAVIGATION_OVERLAY_PROVIDER_ID)
        .with_required_capabilities([NAVIGATION_GIZMOS_CAPABILITY]),
    )
}

pub fn build_navigation_overlay(
    owner: EntityId,
    nav_mesh: &NavigationGizmoSnapshot,
    pie_frame: Option<&NavigationPieFrame>,
    options: NavigationOverlayOptions,
) -> SceneGizmoOverlayExtract {
    let source = NavigationGizmoSnapshot {
        triangles: if options.nav_mesh_areas {
            nav_mesh.triangles.clone()
        } else {
            Vec::new()
        },
        off_mesh_links: if options.off_mesh_links {
            nav_mesh.off_mesh_links.clone()
        } else {
            Vec::new()
        },
    };
    let mut overlay = source.to_scene_gizmo_overlay(owner, false);
    if let Some(frame) = pie_frame {
        append_agent_debug(&mut overlay, frame, options);
    }
    overlay
}

fn append_agent_debug(
    overlay: &mut SceneGizmoOverlayExtract,
    frame: &NavigationPieFrame,
    options: NavigationOverlayOptions,
) {
    const PATH_COLOR: Vec4 = Vec4::new(0.0, 0.9, 0.95, 1.0);
    const DESIRED_COLOR: Vec4 = Vec4::new(0.2, 0.65, 1.0, 1.0);
    const AVOIDANCE_COLOR: Vec4 = Vec4::new(1.0, 0.55, 0.1, 1.0);
    for agent in &frame.tick_report.debug_agents {
        let position = Vec3::from_array(agent.position);
        overlay.pick_shapes.push(OverlayPickShape::Sphere {
            center: position,
            radius: 0.18,
        });
        if options.agent_paths {
            for segment in agent.path.windows(2) {
                overlay.lines.push(OverlayLineSegment {
                    start: Vec3::from_array(segment[0]),
                    end: Vec3::from_array(segment[1]),
                    color: PATH_COLOR,
                });
            }
        }
        if options.desired_velocity {
            push_vector(
                &mut overlay.lines,
                position,
                Vec3::from_array(agent.desired_velocity),
                DESIRED_COLOR,
            );
        }
        if options.avoidance_velocity {
            push_vector(
                &mut overlay.lines,
                position,
                Vec3::from_array(agent.avoidance_velocity),
                AVOIDANCE_COLOR,
            );
        }
    }
}

fn push_vector(lines: &mut Vec<OverlayLineSegment>, origin: Vec3, vector: Vec3, color: Vec4) {
    if vector.length_squared() <= f32::EPSILON {
        return;
    }
    lines.push(OverlayLineSegment {
        start: origin,
        end: origin + vector,
        color,
    });
}
