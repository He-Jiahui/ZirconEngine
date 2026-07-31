---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-picking-indexed-hit-type-inference
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
origin_child_dir: docs/plans/zircon_runtime/runtime/12
fixing_child_dir: docs/plans/performance/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/picking/pointer_hits.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib picking --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib runtime_12_input_stack_mirror_docs_match_structure_audit_counts --locked --jobs 1 -- --nocapture --test-threads=1
---

# Performance01：runtime picking indexed-hit 类型推断编译失败

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
- 来源执行切片：M4 mirror-doc current-source managed retry
- 修复责任计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 交接原因：Performance01 的 picking 热路径重构移除了 indexed-hit collection 的 owned `Vec` 类型锚点；Runtime12 mirror gate 仅在整库 lib-test 编译时暴露该最低层失败。

## 失败现象与复现证据

Coordinator job `9d73f359158d442184b7975ee3bec370` / run `f1ea7588172f4c869511fc0f14cf96c1` 执行：

`cargo +1.94.1 test -p zircon_runtime --lib runtime_12_input_stack_mirror_docs_match_structure_audit_counts --locked --jobs 1 -- --nocapture --test-threads=1`

终态为 `exit_code=101`，run `completed`，job 已释放且无 live process；stdout 为空，目标 Runtime12 test 未执行。`stderr.log` 在 `zircon_runtime/src/core/framework/picking/pointer_hits.rs:25-40` 给出同一推断链上的 4 个 E0277：`collect()` 被反推为不可定长的 `[(usize, usize, f32, HitRecord)]` slice，局部变量因此不满足 `Sized`，最终 `into_iter()` 产生 `&HitRecord` 而不能收集为 `Vec<HitRecord>`。该 job 同时含其他 owner 的 4 个错误；本记录不吸收它们，也不把本次结果声明为 Runtime12 red/green。

## 最低共享层根因

`sorted_hits_for_pointer` 在抽取共享 `sort_indexed_hits(&mut [IndexedHit])` 时删除了原有显式 `Vec<(usize, usize, Real, HitRecord)>` 类型。由于唯一后续约束只要求 slice，编译器无法为无 turbofish 的 `collect()` 恢复 owned collection 类型；这不是 hit cloning、排序规则或 Runtime12 mirror guard 的行为错误。

## 架构修复验收

- 恢复 `sorted_hits_for_pointer` 的 owned `Vec<IndexedHit>` 类型锚点，保持现有单次分组/排序重构与 owned `HitRecord` 数据流不变。
- focused picking lib tests 在 current source 上通过。
- 原始 Runtime12 mirror-doc managed command 重新执行并以 raw stdout 证明目标 test 恰好执行 1 个且通过。

## 禁止临时方案

- 不增加 alias、compatibility shim、silent fallback、duplicated truth、test-only bypass 或 call-site exception。
- 不回退 Performance01 已完成的单次分组/排序架构，不通过额外 clone 掩盖 collection 类型错误。
- 不弱化 Runtime12 mirror-doc gate，也不从本次编译失败推断其测试结果。

## 修复结果与回传

Open state: 最低源码修复已落地：`sorted_hits_for_pointer` 的 collection 现在显式为 `Vec<IndexedHit>`，现有单次分组/排序与 owned-hit 数据流保持不变。`rustfmt --edition 2021`、scoped `git diff --check` 和类型锚点静态检查通过。

2026-07-31 的 source-bound Windows managed retry 使用兼容池 `6251ba21a09d6381d260abce229b35e5262dd449b8618701b39b402c1bb8358f`，job `85de04bb87674001bd32ebab526f876b` 执行 focused picking lib-test command。该次编译不再报告本记录的 picking `E0277`，但 `zircon_runtime` lib-test binary 被 35 个其他 owner 的 current-source 编译错误阻断并以 `exit_code=101` 结束，未执行任何 picking test。可见最低 blocker 分属 Text04 glyph-atlas/page-shadow、Render13/17 Graphics/RHI、Runtime04 artifact store 和 Plugins01/Runtime06 native loader；详细 owner 路由见 `.codex/sessions/20260731-2230-performance01-plan-code-review.md`。因此本记录继续保持 `open`：该 job 只证明原类型错误未复现，不构成 focused GREEN，也不能替代原始 Runtime12 mirror 的恰好 1/1 证据。待外部编译 blocker 收敛后必须重跑两条验收命令，才能转为 `fixed` 并回传。

同日 job `c5599d3916104845bb1de93314e260bf` 在精确修复 Runtime04 artifact deserialize 类型锚、Runtime06 fixture accessor 和 Frameworks04 loader reborrow 后重跑同一 focused command。前一 job 的 6 个 Asset/Plugin 错误全部未复现，picking `E0277` 也继续未复现；但 Text04/Graphics current-source 硬切在编译期间再次漂移，产生 32 个新的或重新出现的 glyph-atlas、pipelined render framework、RHI/UI 和 IBL compile errors，job 以 `exit_code=101` 结束且仍未执行 picking test。该结果继续属于外部编译 blocker 证据，不改变本记录的 `open` 状态或两条最终验收要求。
