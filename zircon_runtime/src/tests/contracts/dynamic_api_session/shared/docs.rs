//! 动态会话的二进制接口、宿主请求和诊断路由需与共享契约同步。集中保存路径、锚点或预期清单；消费方负责读取实际源码。
pub(in super::super) const EXPECTED_RUNTIME_10_MIRROR_DOCS: &[&str] = &[
    "docs/zircon_runtime/dynamic_api/session.md",
    "docs/engine-architecture/runtime-architecture-review-m0.md",
    "docs/engine-architecture/runtime-interface-convergence.md",
    "docs/engine-architecture/runtime-interface-cdylib-loader.md",
];
