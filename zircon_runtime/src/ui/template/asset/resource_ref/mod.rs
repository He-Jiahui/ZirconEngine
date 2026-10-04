//! 为编译依赖、Editor 文件诊断和运行时注册表查找提供各自的资源边界。
//! 收集不执行加载；磁盘路径检查与运行时句柄解析需要由宿主分别配置。

mod collect;
mod resolution_report;
mod resolve;
mod resolver;

pub use collect::{collect_document_resource_dependencies, unique_resource_references};
pub use resolution_report::{UiResolvedResourceDependency, UiResourceResolutionReport};
pub use resolve::{validate_resource_dependency_files, UiResourcePathResolver};
pub use resolver::{
    UiResolvedUiResource, UiResourceResolveDiagnostic, UiResourceResolveDiagnosticCode,
    UiResourceResolver, UiResourceResolverCacheInvalidationReport, UiResourceResolverSchemeMap,
};
