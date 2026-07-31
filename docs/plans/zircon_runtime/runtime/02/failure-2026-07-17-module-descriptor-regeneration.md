---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: module-descriptor-regeneration
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/02
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_runtime/src/engine_module/engine_module.rs
tests:
  - descriptor generation count during bootstrap-with-report
  - bootstrap report and activated module descriptor equivalence
  - dynamic descriptor text is reclaimed after entry drop
---

# Runtime02：bootstrap report 重复生成 module descriptors

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F0 启动路径逐文件静态审查
- 修复责任计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 交接原因：descriptor snapshot 的所有权属于 Runtime02 模块启动契约，性能 Session 不应在 app/bin 层建立旁路缓存。

## 失败现象与复现证据

旧`bootstrap_with_report`为报告和注册重复生成module descriptors；current `ResolvedPluginGroup`已让报告、排序与注册共享冻结snapshot，但注册仍复制descriptor。2026-07-23复核又确认更低层所有权缺口：`EngineModule::module_name/module_description`强制返回`&'static str`，`DescriptorBackedEngineModule::new`因此在每次动态plugin module包装时clone name+description并`Box::leak`。重复module-selection/report/bootstrap/reload即使Drop entry也不会回收文本。

## 最低共享层根因

根因一是启动消费者需要共享一次冻结后的descriptor snapshot；根因二是`EngineModule`把静态builtin常量的寿命要求错误施加到动态descriptor文本。两者都属于Runtime02模块契约，不应在editor/bin缓存或用global永久interner旁路。

## 架构修复验收

- 先加每模块 descriptor 生成次数测试，再让报告、排序与注册消费同一冻结结果。
- 硬切`EngineModule`名称/描述读口为owner-borrowed `&str`，或让冻结artifact显式持有可回收`Arc<str>`；static builtin实现保持零分配，动态wrapper生产`Box::leak`=0。
- 报告中的顺序、依赖和 capability 必须与最终激活模块完全一致。
- dynamic modules 1/100/1,000及entry/report/reload 1/1,000/100,000记录descriptor calls、String owners、leaked bytes和RSS；entry/drop或generation retire后动态文本回收。
- 当前源码 cold/warm bootstrap trace 对比descriptor次数、clone bytes和耗时；没有数据不得声称启动收益。

## 禁止临时方案

- 不得在 editor/bin 单独缓存 descriptors。
- 不得用process-global永久interner、更多`Box::leak`或限制reload次数掩盖所有权缺口。
- 不得删除 bootstrap report 或降低一致性检查来换取时间。

## 修复结果与回传

2026-07-17 current-source implementation:

- `ResolvedPluginGroup` now owns activation-order-aligned module and descriptor vectors. `try_finish` creates one descriptor for each enabled direct entry; entries disabled before that generation resolves create zero. Nested groups remain immediate typed-validation generations, so a later outer disable cannot erase their first call but also performs no outer regeneration.
- Nested groups transfer their already-resolved module/descriptor pairs instead of discarding the descriptor and regenerating it in the outer builder. Replacing an entry through `set` clears only that entry's pending snapshot.
- Built-in selection reports and bootstrap registration read the same frozen group snapshot. Registration still clones each descriptor into `CoreRuntime`; no global cache, editor/bin cache, compatibility module, or cfg-gated bypass was added.
- TDD source evidence: the new generation-count test is RED against the previous implementation because `try_finish` plus two snapshot reads invoke `descriptor()` three times while the contract requires one. A direct disabled generation requires zero. Nested regressions require the inherited snapshot to remain generation 1 and prove a later outer disable does not invoke generation 2.

Current state: `implemented_pending_managed_validation`. The old reservation `ed67b4bee45f40d2b4e16f7ce379604e` bound exact3 fingerprint `74d5fd81bed89111b8a8ff9f64552f8ac11bf19a7e3b7630b7301c3d970a38b8`, but Frameworks05 preference host wiring subsequently changed the shared `zircon_app/src/entry/engine_entry.rs` owner. The reservation is absent from the current coordinator ledger and is permanently stale; it must not be consumed or cited as acceptance. Snapshot `903` established the combined descriptor/preference path set but is superseded by this record correction; a fresh exact15 snapshot and managed `zircon_app --lib` gate with full compile-input pre/post attestation remain pending. The nested typed-error integration gate and final report/activation equivalence review also remain required before this handoff may be renamed `fixed-*`; no pass or startup-speed improvement is claimed.

2026-07-23 ownership addendum: current entry root 11/11、1,816行复核确认snapshot实现仍在，但dynamic wrapper每构造泄漏两段文本；现有75个entry tests没有reclaim/RSS断言。故本failure不得在原descriptor generation Cargo转绿后直接fixed，必须同时删除生产`Box::leak`并取得repeated-entry reclamation证据。

2026-07-31 current-source correction:

- `EngineModule::module_name` and `module_description` now expose owner-borrowed `&str`; static builtin implementations retain their `&'static str` constants without allocation, while this no longer forces that lifetime on dynamic modules.
- `zircon_app/src/entry/builtin_modules.rs` now stores the full `ModuleDescriptor` inside `DescriptorBackedEngineModule` and returns slices of its owned `String` fields. Its 1/100/1,000 cardinality regression verifies both returned pointers borrow the descriptor-owned allocation; the current production wrapper contains no `Box::leak` call.
- The frozen `ResolvedPluginGroup` snapshot implementation described above remains in place. This review is static evidence only: the fresh exact source-bound `zircon_app --lib` gate, dynamic repeated-entry reclamation measurement, and report/activation parity gate remain required before renaming this handoff `fixed-*`.

Open state: `descriptor generation and dynamic-text ownership source repairs are present; managed validation and reclamation evidence remain pending`; no dynamic pass or startup gain is claimed.
