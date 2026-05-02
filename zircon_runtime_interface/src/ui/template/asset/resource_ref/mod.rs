mod dependency;
mod diagnostic;
mod fallback_policy;
mod resource_kind;
mod resource_ref;

pub use dependency::{UiResourceDependency, UiResourceDependencySource};
pub use diagnostic::{UiResourceDiagnostic, UiResourceDiagnosticSeverity};
pub use fallback_policy::{UiResourceFallbackMode, UiResourceFallbackPolicy};
pub use report::UiResourceCollectionReport;
pub use resource_kind::UiResourceKind;
pub use resource_ref::UiResourceRef;

mod report;
