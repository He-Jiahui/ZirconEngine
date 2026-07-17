---
related_code:
  - zircon_editor/src/ui/retained_host/ui/tests/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01
  - docs/plans/zircon_editor/editor_ui/05
  - docs/plans/zircon_editor/editor_ui/08
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/slint/internal/core/model/repeater.rs
tests:
  - retained_host/ui/tests source inspection 10/10
  - coordinated Windows zircon_editor performance tests pending
  - F4 pointer, pane, projection, floating-window and component-showcase interaction trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained host UI tests 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/ui/tests` 当前共 **10** 个 Rust 文件、按物理行计 **4,229** 行，已逐文件阅读 **10/10**。范围包括 presentation shell、component showcase、floating windows、host scene projection、scene document pane、Welcome、Workbench layout frames、共享 fixture，以及 host-scene assertions。

这些文件均为测试代码，没有产品每帧、pointer、pane projection或插件生命周期执行入口。因此本批没有把测试夹具分配成本误报为运行时瓶颈，也没有为缩短测试时间修改产品语义。当前源码的协调式 Cargo、F4 交互 trace与相关产品热路径计数仍未完成，整个 `zircon_editor/src/ui` 继续留在 `pending.md`。

## 测试侧性能风险

- PERF-MVP-136：component-showcase 测试多次构建 root shell fixture、加载 builtin host templates、投影完整 pane并遍历 host rows；多个测试通过全局 `env_lock` 串行。它影响验证墙钟时间和可并发度，不影响发布版 editor frame time。
- `component_showcase_contract_source` 每次调用读取并拼接九个 ZUI 资源，但当前只在一个源码契约测试中调用，不足以证明需要局部缓存。
- host-scene 与 Workbench layout fixtures包含大型 DTO literal；主要成本发生在测试二进制编译和单次 fixture构建，不是产品运行时冗余。
- 现有测试覆盖 frame、clip、floating host、pane DTO、structured options与 runtime binding parity，可作为 PERF-MVP-130 至 PERF-MVP-135 动态验收的行为护栏，但不能替代规模计数或真实窗口 trace。

## 后续验收

先完成 F4 产品路径的 current-source Cargo与交互/clone/rebuild计数，再测量本测试目录的逐 case墙钟和并发阻塞。只有确认 fixture构建显著占据验证时间，才引入 suite-scoped immutable builtin fixture并缩小环境锁；必须证明测试顺序无关、环境隔离与产品 generation/rebuild路径未被缓存绕过。
