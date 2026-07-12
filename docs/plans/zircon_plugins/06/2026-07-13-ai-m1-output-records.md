---
related_code:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/catalog.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/compile.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/mod.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/runtime/src/tick_lod.rs
  - zircon_runtime/src/core/framework/ai/behavior_tree.rs
  - zircon_runtime/src/core/framework/ai/error.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/owner_revocation.rs
  - zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs
implementation_files:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/catalog.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/compile.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/condition.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/composite.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/decorator.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/service.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/task.rs
  - zircon_plugins/ai/runtime/src/manager/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/manager/tick.rs
  - zircon_plugins/ai/runtime/src/manager/execution_gate.rs
  - zircon_plugins/ai/runtime/src/manager/validation.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/runtime/src/tick_lod.rs
  - zircon_runtime/src/core/framework/ai/behavior_tree.rs
  - zircon_runtime/src/core/framework/ai/error.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/owner_revocation.rs
  - zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs
plan_sources:
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-12 严格按 zircon_plugins 架构计划完成插件功能
tests:
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_compile.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_execution.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_node_catalog.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/ai/runtime/src/tests/tick_lod.rs
doc_type: milestone-detail
---

# 2026-07-13 AI M1 行为树编译与标准节点库产出记录

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | T1 资产与编译 | `完成` | 2026-07-13 | 显式 `format_version = 1` 的 `.btree.toml` 经过与 manager 注册相同的结构/参数校验，编译为前序 dense node 与独立 direct-child index table；错误保留 TOML、validation、compile 的 typed source。 |
| M1 | T2 节点目录与标准库 | `完成` | 2026-07-13 | 18 个标准实现分属 composite/decorator/service/task 文件；目录由 `TypedExtensionPoint` 维护稳定 slot，并在插件注册时将 bootstrap sentinel 原位迁移到实际 intern AI owner，不改变已编译树引用的 slot；`ai.behavior_node_registry.v1` 接受每 agent Rust 工厂贡献。中心 owner revoke 会等待 execution lease、删除私有 slot 并退役引用工厂的编译树与 agent 状态。 |
| M1 | T3 无打断执行内核 | `完成` | 2026-07-13 | 删除 `manager/execution.rs`，迁移到 `behavior_tree/executor.rs`；Sequence/Selector/Parallel 续跑不重放 terminal sibling，RandomSelector、计时 decorator、RunSubtree 和外部有状态节点均有三态回归。 |
| M1 | T4 Update 系统与 LOD | `完成` | 2026-07-13 | `ai.behavior_tick` 注册为 Update 系统；Full/Half/Quarter 确定性错峰，跳过帧的 delta 累积到下一实际 tick；事件、system anchor、provided interface 与 `plugin.toml` 保持一致。 |
| M1 | Testing | `完成` | 2026-07-13 | Windows：AI runtime 44/44 tests；标准验证矩阵 `OK`；生产源码 panic/expect/unwrap 扫描为 0。 |

## 架构与参考证据

- 当前仓库：中立 DTO 与 typed manager error 留在 `zircon_runtime::core::framework::ai`；资产编译、节点工厂、执行状态、目录和系统注册留在可选 AI runtime 插件。
- Unreal：标准节点保持 Composite/Decorator/Service/Task 四分类；Running 分支续跑、优先级条件重求值、计时 decorator 与 subtree 三态被测试固化。Observer abort 明确保留到 M2，不在 M1 伪实现。
- Fyrox：采用共享不可变 compiled tree + 每 agent 轻量运行态的所有权形态；Zircon 额外以显式目录 slot 和运行时扩展接口支持插件节点。

## 结构规范闭合

- 原 531 行 `manager/execution.rs` 已删除，执行责任硬切换到 folder-backed `behavior_tree/executor.rs`；blackboard/perception 条件继续拆入 `executor/condition.rs`。
- 标准节点按四个领域文件拆分；插件注册行为从 `plugin.rs` 根拆到 `plugin/registration.rs`。
- 当前 AI production 最大文件为 `manager/validation.rs` 818 行、`behavior_tree/executor.rs` 811 行，均低于约 1000 行结构门槛；未新增 compatibility shim 或 free-function registration site。
- 错误使用 typed variants，生产 AI 源码不存在 `.unwrap()`、`.expect()` 或 `panic!()`。

## 验证证据

- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime --locked --offline --jobs 1 --target-dir E:\\cargo-targets\\zircon-ai-m1`：44 passed，0 failed。
- owner 生命周期聚焦：真实非零 AI owner、在途 tick revoke barrier、revoke 期间新代际 add 线性化共 3/3 通过；revoke 不会在旧 runtime/vtable 析构前返回。
- `validate-matrix.ps1 -Package zircon_plugin_ai_runtime -TargetDir E:\\cargo-targets\\zircon-ai-m1 -SkipBuild`：Cargo test `OK`。
- 运行输出中的 69 条 warning 来自共享 `zircon_runtime` 并发工作树；AI crate 无新增 warning。
- 插件结构审计中的 AI free-function registration sites 与 compatibility shims 均为 0；全局遗留项归属 Navigation，不计入 AI M1。

## 未完成边界

- M2 dense Blackboard、generation/observer 与 AbortPolicy 未完成。
- M3 MoveTo/PlayAnimation/ScriptTask 的真实子系统交接未完成；M1 只交付目录身份与稳定三态占位分派。
- M4 Perception 扫描、遮挡、遗忘和事件刺激适配未完成。
- M5 Editor 图编辑器、运行态镜像与 overlay 未完成。
