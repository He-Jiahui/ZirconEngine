//! 运行时根公开面和图形别名的收敛结果需与架构文档一致。集中保存路径、锚点或预期清单；消费方负责读取实际源码。
pub(super) const LIB_RS: &str = include_str!("../../../lib.rs");
pub(super) const PRELUDE_RS: &str = include_str!("../../../prelude.rs");
pub(super) const CORE_MOD_RS: &str = include_str!("../../../core/mod.rs");
pub(super) const ROOT_SURFACE_DOC: &str =
    include_str!("../../../../../docs/crates/zircon_runtime/core/root_surface.md");
pub(super) const ROOT_SURFACE_M1_DOC: &str =
    include_str!("../../../../../docs/architecture/runtime-root-surface-m1.md");
pub(super) const INTERFACE_CONVERGENCE_DOC: &str =
    include_str!("../../../../../docs/architecture/runtime-interface-convergence.md");
pub(super) const RUNTIME_02_PLAN: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md"
);
pub(super) const RUNTIME_02_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/02/2026-07-09-core-spine-and-root-surface-output-records.md"
);
pub(super) const RUNTIME_INDEX: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
