//! Reference editor workbench surface built from runtime UI primitives.

mod builder;
mod ids;
mod metrics;
mod surface;
mod template_surface;
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
