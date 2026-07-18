---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: kira-graph-sync-repeated-compilation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/02-sound.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/sound/runtime/src/kira_bridge/manager.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_compile.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph
tests:
  - graph compile and validation invocation-count test
  - 10/100/1000 track mutation benchmark
resolved_at: 2026-07-18
---


# Sound02：Kira graph mutation 重复编译并延长全局状态锁

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP 默认 Sound 插件、Kira manager 与 mixer mutation 静态审查
- 修复责任计划：`docs/plans/zircon_plugins/02-sound.md`
- 交接原因：这些文件正由 Sound02 的 Kira 迁移 Session 修改；性能审查不并行改写同一实现，只交付最低根因和验收要求。

## 失败现象与复现证据

`KiraEngine::sync_graph` 的增量路径调用 `diff_graphs(previous, graph)`；`diff_graphs` 先执行一次只用于校验、结果被丢弃的 `compile_graph(after)?`，随后再次执行 `compile_graph(after)?` 得到 `compiled_after`。返回 `sync_graph` 后又第三次 `compile_graph(graph)?`。

track/effect/send mutation caller 在持有 `DefaultSoundManager.state` mutex 时先 clone 完整 graph，并通常显式调用 `validate_graph(&graph)?`，随后再进入上述三次 compile/validate。一次 graph 编辑因此可能重复验证四遍，并把 clone、diff、HashMap/HashSet 构建与 Kira handle 更新都纳入同一全局锁持有窗口。

## 最低共享层根因

graph diff 与 compiled next graph 是分离返回值，调用链没有“一次验证/编译、同时供 diff 与 apply 使用”的中间表示；caller 与 Kira owner 也没有明确谁负责验证。

## 架构修复验收

- 冻结单一入口：一次 next-graph validate/compile 后，同时生成 diff 与 apply 所需数据；调用计数测试约束每次 mutation 不重复 compile/validate。
- 保持 graph 原子提交：Kira apply 失败时 `state.graph` 不改变；所有现有 UnknownTrack/InvalidMixerGraph/unsupported M1 surface 语义不变。
- 对 10/100/1000 track 的 add/update/remove/send mutation 记录 p50/p95、分配与 state-lock hold time。
- 若把纯计算移出 mutex，必须通过 generation/CAS 或其他契约防止两个并发 mutation 覆盖，不能牺牲串行一致性。

## 禁止临时方案

- 不得只删除 caller 的 `validate_graph` 而保留 `diff_graphs`/`sync_graph` 内三次编译。
- 不得在锁外 clone 后无版本检查地写回，制造 lost update。

## 修复结果与回传

- 根因：Graph mutations treated active and inactive graphs alike, recompiling Kira state under the public lock without an active-state compare-and-swap boundary and conflating logical limits with physical Kira sub/send capacity.
- 架构修复：Separated inactive neutral authoring from active Kira compilation, added revision plus active-state CAS, compiled outside the lock, reused the production commit primitive in the active harness, and preflighted physical sub/send capacity atomically.
- 验证：Final current-source benchmark job 402c6c99e45d45489082cdffa3154d05 passed 1/1; 1000-track active public lock p95 6.166ms and Kira p95 75.562ms stayed within budget; broad 344/344 and package check exit 0.
- 回传：Repeated active graph compilation and lock-window risk are fixed in the full Sound M1 current source; immutable M1 milestone SHA remains the downstream acceptance boundary.
