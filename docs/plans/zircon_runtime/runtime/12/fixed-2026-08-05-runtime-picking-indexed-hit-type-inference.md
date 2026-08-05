---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: runtime-picking-indexed-hit-type-inference
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
origin_child_dir: docs/plans/zircon_runtime/runtime/12
fixing_child_dir: docs/plans/performance/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/picking/pointer_hits.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/picking.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --lib picking --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --lib runtime_12_input_stack_mirror_docs_match_structure_audit_counts --locked --jobs 1 -- --nocapture --test-threads=1
resolved_at: 2026-08-05
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
- 默认 feature 的 Runtime12 mirror-doc managed command 保留为无关 Graphics/Text/UI 编译阻断证据，不把 `0 tests` 声明为镜像测试红绿。
- 同一 Runtime12 mirror-doc 断言在 `--no-default-features` 隔离图中重新执行，并以 raw stdout 证明目标 test 恰好执行 1 个且通过。

## 禁止临时方案

- 不增加 alias、compatibility shim、silent fallback、duplicated truth、test-only bypass 或 call-site exception。
- 不回退 Performance01 已完成的单次分组/排序架构，不通过额外 clone 掩盖 collection 类型错误。
- 不弱化 Runtime12 mirror-doc gate，也不从本次编译失败推断其测试结果。

## 修复结果与回传

- 根因：sorted_hits_for_pointer passed an unconstrained collect() result only to a slice-taking helper, so removal of the owned collection annotation left rustc unable to infer Vec<IndexedHit> and produced the E0277 unsized-slice chain.
- 架构修复：Restore the explicit owned Vec<IndexedHit> type anchor at the shared picking collection boundary while preserving the single grouping/sort pass and owned HitRecord flow; keep current-source guard repairs structural, with no alias, shim, clone fallback, or Runtime12 test bypass.
- 验证：Windows coordinator immutable input hash 850692c2f0901d742c0edb248323fc7b6e6042ee44f980c5de05d2d457f554f7: focused --no-default-features picking job c9d89842c7904b7081ebc17b4985e514 run c1ba70bee7a043c9bdcc6b543d45850a exited 0 with 23 passed, 0 failed, 4187 filtered; isolated Runtime12 mirror job 3439da1e065946dd93f029ee0cd49104 run 2e7a6c0a40a048c981e2934d51dba12d exited 0 with the named test exactly once, 1 passed, 0 failed, 4209 filtered. The default-feature mirror attempt job 5f8c48656c5845389c0caa60cae4b0b0 run 06dd59e91eee427dbbae06186ad57c54 exited 101 before tests on 134 unrelated optional Graphics/Text/UI compile errors and is retained only as external-blocker evidence. rustfmt, scoped git diff --check, and handoff validation passed.
- 回传：Return the source-complete picking inference repair to Runtime12 with focused 23/23 and isolated mirror 1/1 managed GREEN. Default-feature Graphics/Text/UI compile blockers remain owned by their respective plans and are not absorbed by this return.
