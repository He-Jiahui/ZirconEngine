---
related_code:
  - zircon_runtime/src/core/runtime/tests/resolution/behavior.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior/dependency_cycles.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior/exact_dependency_resolution.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior/factory_panics.rs
  - zircon_runtime/src/core/runtime/tests/resolution/structure.rs
  - docs/engine-architecture/core-runtime-service-registry.md
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - rustfmt --edition 2021 zircon_runtime/src/core/runtime/tests/resolution/behavior.rs zircon_runtime/src/core/runtime/tests/resolution/behavior/dependency_cycles.rs
  - git diff --check -- zircon_runtime/src/core/runtime/tests/resolution/behavior.rs zircon_runtime/src/core/runtime/tests/resolution/behavior/dependency_cycles.rs docs/engine-architecture/core-runtime-service-registry.md
  - current-source physical line and test-name inventory
  - runtime_15_no_oversized_test_files
  - runtime_15_render_pass_gpu_context_mesh_command_lists_are_child_owner
  - runtime_15_mesh_draw_command_list_is_folder_backed
  - core::runtime::tests::resolution::behavior::dependency_cycles::four_frame_resolution_cycle_reports_canonical_registry_key
  - core::runtime::tests::resolution::behavior::dependency_cycles::five_frame_resolution_cycle_reports_canonical_registry_key
  - fresh standalone resolution structure guard
doc_type: milestone-detail
---

# Runtime 15 Resolution 测试 Owner 拆分

## 状态与完成项目

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | Core runtime resolution dependency-cycle test owner split | `runtime_15_resolution_dependency_cycle_test_owner_split_current_source_3_passed` | 2026-07-14 | `behavior.rs` 以结构性 `mod dependency_cycles;` 接入 folder-backed child；四层与五层 resolution cycle 测试完整迁入 `behavior/dependency_cycles.rs`。父 owner 从 821 行降到 709 行，child 为 115 行；12 个 resolution behavior 测试名完整保留。current-source child 行为 2/2、fresh standalone structure guard 1/1。未提高 800 行测试预算，未增加豁免、兼容层或重复实现。 |
| M3 | Exact dependency cached-key test owner split | `runtime_02_15_resolution_exact_dependency_test_owner_split_static_passed_cargo_deferred` | 2026-08-27 | exact 4/5 dependency initialization 两测原样迁入 `behavior/exact_dependency_resolution.rs`。当前父 owner 631 行 / 10 tests；dependency-cycle、exact-dependency、factory-panic child 为 115/217/258 行及 2/2/4 tests。Python 结构回归 1/1，测试体规范化 SHA-256 等价；Cargo 延后。 |

## Owner 边界

- `behavior.rs` 保留 lazy resolve、并发 factory、registered identity、失败重试与 exact dependency 初始化行为。
- `behavior/dependency_cycles.rs` 只承接四层和五层依赖循环的 canonical registry key 回归。
- `behavior/exact_dependency_resolution.rs` 只承接 exact 4/5 dependency cached-key 初始化回归。
- `behavior/factory_panics.rs` 保持另一会话新增的 factory panic/lifecycle owner，不并入本切片。
- 测试名、依赖图、`CoreError::DependencyCycle` 错误断言与运行时公开路径均未改变。
- 当前未提交的并发 service-resolution 与 stable identity 回归原样保留在父 owner，本切片没有重写或回退它们。

## 验证

- `rustfmt`：父/child 两个 Rust 文件通过。
- `git diff --check`：父/child 与模块文档通过，仅有仓库既有 LF-to-CRLF 提示。
- current-source inventory：父 709 行 / 10 tests，child 115 行 / 2 tests。
- standalone structure 的 `runtime_15_no_oversized_test_files` 精确门禁重新运行后为 1/1 通过，证明 Resolution 父/child 已恢复通用测试预算。
- 18:24 current-source Runtime lib-test binary 明确挂载 `behavior::dependency_cycles`，四层与五层 cycle 测试各 1/1 通过。
- fresh standalone `resolution/structure.rs` guard 编译并 1/1 通过；守卫读取父/child 双 owner，要求 `mod dependency_cycles;`、两个 child 测试名及父级无重复，并同步当前 single-flight/reentry/stable-identity source anchors。
- 完整 resolution 过滤为 11/13；本切片两个 child 测试均通过。`deactivation_invalidates_registered_manager_identity_before_reactivation` 暴露 Frameworks05 M4 reactivation 没有把 service 从 `Unloaded` 恢复为 `Registered`，按最低 owner 另行修复，不计成本切片回归。
- Render18 两个局部 owner 门禁仍按当前源码失败：GPU context `got 897`，mesh command-list tests `got 515`；本切片没有修改或放宽这两个守卫。
- 受管 Cargo 编译和 resolution 行为测试留到 Runtime15 测试阶段；当前共享 Cargo 池仍由其他会话占用。

## 未关闭范围

Runtime15 仍为 `in_progress`。Render18 当前增量使 `mesh_draw_command_list/tests.rs` 为 515 行、`render_pass_execution_context/gpu.rs` 为 897 行；这两个最低 owner 属于活动 Render18 会话。尝试创建 canonical cross-plan Failure 时，协调器证明新边会与现有 `Render18 -> Plugins08 -> Runtime15` 历史链形成依赖环，因此 artifact 与计划链接已撤回，Failure graph 恢复为既有 109 nodes / 3 diagnostics；该责任改由非 Failure 协调警告交给 Render18 owner。优先 review F16 与完整 structure/code-review/Cargo 门禁也尚未关闭。
