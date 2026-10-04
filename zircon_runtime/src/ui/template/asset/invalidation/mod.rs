//! 为编译缓存和包验证提供确定性输入修订、失效阶段与作者性能提示。
//! 报告描述所需工作；实际热重载执行另有 watch 计划和事务边界。

mod diagnostic;
mod fingerprint;
mod graph;

pub use diagnostic::{
    collect_invalidation_diagnostics, BROAD_SELECTOR_WARNING_THRESHOLD,
    LARGE_DOCUMENT_NODE_WARNING_THRESHOLD, NON_VIRTUALIZED_SCROLL_CHILD_WARNING_THRESHOLD,
};
pub use fingerprint::{
    component_contract_fingerprint, declared_imports_fingerprint, document_import_fingerprints,
    fingerprint_document, resource_dependencies_fingerprint,
};
pub use graph::UiInvalidationGraph;
