//! 资源声明、来源和静态诊断的共享契约；Runtime 收集器显式校验，解析器随后查找注册表。
//! 此处拥有可序列化的声明数据，不持有加载器、缓存或运行时资源句柄。
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
