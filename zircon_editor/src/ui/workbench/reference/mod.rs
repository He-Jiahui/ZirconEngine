//! Reference editor workbench surface built from runtime UI primitives.

#[path = "builder/mod.rs"]
mod builder;
#[path = "ids.rs"]
mod ids;
#[path = "metrics.rs"]
mod metrics;
#[path = "surface.rs"]
mod surface;
#[path = "template_surface.rs"]
mod template_surface;
#[path = "tokens.rs"]
mod tokens;

pub use ids::EditorWorkbenchReferenceIds;
pub use metrics::EditorWorkbenchReferenceMetrics;
pub use surface::{build_editor_workbench_reference_surface, EditorWorkbenchReferenceSurface};
pub use template_surface::{
    build_editor_workbench_template_surface, EditorWorkbenchTemplateControlIds,
    EditorWorkbenchTemplateFrames, EditorWorkbenchTemplateSurface,
    EditorWorkbenchTemplateSurfaceError,
};
pub use tokens::EditorWorkbenchReferencePalette;
