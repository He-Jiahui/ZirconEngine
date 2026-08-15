//! Reference editor workbench surface built from runtime UI primitives.

mod builder;
mod ids;
mod metrics;
mod surface;
mod template_surface;
mod tokens;

pub use ids::EditorWorkbenchReferenceIds;
pub use metrics::EditorWorkbenchReferenceMetrics;
pub use surface::{EditorWorkbenchReferenceSurface, build_editor_workbench_reference_surface};
pub use template_surface::{
    EditorWorkbenchTemplateControlIds, EditorWorkbenchTemplateFrames,
    EditorWorkbenchTemplateSurface, EditorWorkbenchTemplateSurfaceError,
    build_editor_workbench_template_surface,
};
pub use tokens::EditorWorkbenchReferencePalette;
