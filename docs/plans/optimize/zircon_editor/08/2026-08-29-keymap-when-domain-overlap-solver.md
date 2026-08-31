---
plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
review_source: docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
status: source_review_complete_profiling_pending
date: 2026-08-29
---

# Editor08 Keymap When-Domain Overlap Solver Review

## 结论

当前在途实现已经修复“同 chord 字典序首项遮蔽唯一 enabled 命令”：keyboard dispatch 会在 signature bucket 内逐项执行当前 `CommandEvalCtx` enablement，唯一 enabled 才返回，多个 enabled 则拒绝歧义。这个运行时行为应保留。

但 `WhenClause::can_overlap_in_interactive_context` 不应作为最终 keymap admission/conflict 算法。它收集两个任意布尔表达式的全部 atom，再深度优先枚举 true/false assignment；最坏递归节点上界为 `2^(A+1)-1`，并在每个分支 clone `BTreeMap<WhenAtom, bool>`、exclusive-axis `String/newtype` 状态。32 个独立 atom 的理论上界已经是 8,589,934,591 个递归节点，因此不能用“设置页面不是每帧热路径”掩盖结构风险。

求解器也不是其注释所称的 exact interactive solver。`PlayMode` 实际总处于 `Edit | Building | Playing | CleanupFailed` 之一，但 assignment 允许四个 play atom 同时为 false；四个 `Not(PlayMode(...))` 的合取会被判为可满足。结果虽然偏保守、不会静默误派发，却会把真实不重叠域误报为冲突。

本切片停止生产算法改动，先建立 profiler/benchmark receipt。修复方向不是微调递归，而是采用 Unreal 同级的显式 binding context：稳定 context ID、父子关系和 owner 形成 key binding 的结构域；动态 `WhenClause` 只负责当前 context 的 `CanExecute`。复杂复合 when 仍需按 E-CMD-P1-08 编译为有 depth/node/instruction budget 的迭代 program，不能继续以无界 serde 树进入输入路径。

## 当前源码证据

| 范围 | 行数 | bytes | SHA-256 |
| --- | ---: | ---: | --- |
| `core/commands/keymap.rs` | 247 | 8,856 | `cb086498a4c0308e27dcc0a57f80800a898399ccdfe2e0c9244dae72f0a2d6ab` |
| `core/commands/keymap/tests.rs` | 361 | 11,744 | `c4ac74758d77e9e85ddd4a53284da1fc4730a1e0d9d7468ae8e52d3bd9dcaa88` |
| `core/commands/when.rs` | 497 | 16,174 | `9ced7b2bb1cf06951a4d959e6b85a47a0693e9ffa9d442ebe72d624f9339b64a` |
| `tests/commands/when.rs` | 134 | 5,609 | `0820a2461fea4e8eb8873bfed6eb140c83f227adde757c64d8bec7ccc1f46b25` |
| `ui/host/editor_manager.rs` | 456 | 17,391 | `1e299478eda06ff34436a4af386044b76e97b7829e7ba765bcea16cb8f7acd51` |
| `input_dispatch.rs` | 48 | 1,737 | `55555e82817d066f05d2a373e68a6c4f26464e869261c771830787627f13f836` |

调用链是 `UiKeyboardInputEvent -> EditorManager::resolve_keyboard_input -> EditorKeymapService -> signature_index bucket -> descriptor.is_enabled(shared CommandEvalCtx)`。冲突诊断链是 `EditorKeymap::conflicts_with_when -> WhenClause::can_overlap_in_interactive_context -> when_clauses_are_satisfiable`。

## 参考引擎复核

- Unreal `UICommandInfo.h` 的 `FBindingContext` 明确定义“input bindings are valid”的 context，并规定 parent/child context 不得复用 chord；`FUICommandInfo` 保存稳定 `BindingContext`。
- Unreal `UICommandList.h` 将 command 映射到 action、`CanExecuteAction` 和可选 `FUIActionContext`；command list 保存已知 context 集，并在实际 context 图中解析/执行。
- 因此参考标准是“显式 context 做冲突结构域 + 当前 action context 做 enablement”，不是对任意 can-execute 表达式做全局 SAT。

## Profiling 计划与验收门

1. 在受管 Windows validation artifact 中加入 solver probe，输入 atom 数 `4/8/12/16/20`，分别覆盖 satisfiable early-exit、unsatisfiable full-search、exclusive-axis contradiction、capability-heavy CNF。
2. 记录每组 `p50/p95/p99`、递归节点、assignment clone 次数/bytes、最大深度和 allocations；release profile 单线程，固定迭代与 warm-up，输出到仓库 E 盘 target/artifact，不落 C 盘。
3. 同一 fixture 对照显式 context collision：context index lookup + parent relation check，记录相同指标。
4. 生产修复前必须取得 profiler receipt；当前 `shell_projection.rs` 四个跨 owner 旧构造仍会阻断 lib-test 编译，因此本记录不伪造 timing/allocation 数据。
5. MVP 门：10,000 bindings、1,000 contexts、最大父链 8；effective snapshot 构建 p95 <= 8 ms，单 chord dispatch p95 <= 50 us、0 heap allocation，冲突事务 p95 <= 16 ms；歧义必须 typed reject，不能按 command ID 静默选取。

## 结构修复顺序

1. 新增 validated `EditorCommandBindingContextId` 和 owner-scoped context registry，显式声明 parent；禁止循环、未知 parent 和跨 owner 冒名。
2. command descriptor 持稳定 binding context；built-in/plugin registration batch 原子绑定 context，默认 global 仅作为明确选择，不从 `WhenClause` 猜测。
3. effective keymap 按 `(chord signature, full chord, context)` 建索引；设置事务在相同/祖先后代 context 内报告 collision，平行 context 可复用。
4. keyboard gateway 从 focus/document/tool/modal 栈捕获 active context chain，在 bucket 内按 context specificity 与 current enablement 解析；同优先级多个 enabled 返回 typed ambiguity。
5. 删除 `can_overlap_in_interactive_context` 的 keymap owner；`WhenClause` 转入有预算的 context program，仅供 enablement，不承担 binding domain identity。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-29 | current-source、优化总报告与 Unreal command context 复核 | `completed` | 6 个当前链路文件完成 hash/line/byte inventory；确认 runtime unique-enabled dispatch 已接入，确认 SAT owner 的指数复杂度、clone 放大与 PlayMode 全否精确性缺口。 |
| 2026-08-29 | Windows profiler/benchmark receipt | `pending` | 等待跨 owner `shell_projection.rs` 完成新 descriptor 构造迁移后走 coordinator managed validation；无 timing/allocation 数值被声明。 |
| 2026-08-29 | explicit binding-context hard cut | `blocked_by_profiling_gate` | 已确定结构方向和量化门；按用户要求，在 profiler receipt 前不开始生产算法替换。 |
