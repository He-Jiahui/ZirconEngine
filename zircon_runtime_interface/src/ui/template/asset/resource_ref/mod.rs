#[path = "dependency.rs"]
mod dependency;
#[path = "diagnostic.rs"]
mod diagnostic;
#[path = "fallback_policy.rs"]
mod fallback_policy;
#[path = "resource_kind.rs"]
mod resource_kind;
#[path = "value.rs"]
mod value;

pub use dependency::{UiResourceDependency, UiResourceDependencySource};
pub use diagnostic::{UiResourceDiagnostic, UiResourceDiagnosticSeverity};
pub use fallback_policy::{UiResourceFallbackMode, UiResourceFallbackPolicy};
pub use report::UiResourceCollectionReport;
pub use resource_kind::UiResourceKind;
pub use value::UiResourceRef;

#[path = "report.rs"]
mod report;
