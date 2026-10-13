//! UI 资产绑定的共享契约入口。
//!
//! schema 模块约束跨资产名称，target 模块定义缺值和写入端点，expression 模块定义有预算的源语法；
//! 本模块将这些类型作为 `ui::template` 的一部分统一重导出。
#[path = "diagnostic.rs"]
mod diagnostic;
#[path = "expression.rs"]
mod expression;
#[path = "schema.rs"]
mod schema;
#[path = "target.rs"]
mod target;

pub use diagnostic::{
    UiBindingDiagnostic, UiBindingDiagnosticCode, UiBindingDiagnosticSeverity, UiBindingReport,
};
pub use expression::{
    UiBindingExpression, UiBindingExpressionEvaluationError, UiBindingExpressionParseError,
    UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY, UI_BINDING_EXPRESSION_MAX_DEPTH,
    UI_BINDING_EXPRESSION_MAX_NODES, UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
    UI_BINDING_EXPRESSION_MAX_TOKENS,
};
pub use schema::{
    UiActionPayloadFieldName, UiBindingContractTerm, UiBindingSchemaNameError,
    UiBindingSchemaNameKind, UI_BINDING_SCHEMA_NAME_MAX_BYTES,
};
pub use target::{
    UiBindingMissingValuePolicy, UiBindingMissingValueResolution, UiBindingTarget,
    UiBindingTargetAssignment, UiBindingTargetKind, UiBindingTargetSchema,
};
