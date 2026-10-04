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

## 2026-09-19 successor intake (failure-roll-01a084c8-runtime02-module-descriptor-r2)

The stale Runtime02 reservations were not reused. The current coordinator
index admitted a fresh successor and applied ownership-transfer fingerprint
`a1d2d1da39b561665968a8452bc46d0d3352e863ea52718d404633637b594f95` for this
failure record plus the four source/test paths in the successor scope. A live
lease and baseline attribution were acquired for exactly those paths. Intake
hashes were:

- `zircon_app/src/entry/builtin_modules.rs`
  `f0e21b1343a03d501f8bc1ab9d52366f900f9d232b3583d7bc67dc25928bae5e`
- `zircon_app/src/plugins/builder.rs`
  `2690203e565620046ba48c7fd1145b438d2c6685b46b229bcdadb930fc47a22a`
- `zircon_runtime/src/engine_module/engine_module.rs`
  `821f6a4a5d3045e1d0d6c849828eab21a31116fc6b80fa2ca596d2022bb653ae`
- `zircon_runtime/src/engine_module/tests.rs`
  `3a0523eb76b873d7fb0ebf5b3c78ea75d0052225e48738a4075d2f84771594f3`

Current-source call-chain correction: the frozen composition and dynamic
descriptor wrapper now live under
`zircon_runtime/src/builtin/runtime_modules/{composition,assembly,plugin_modules}`
and `BuiltinEngineEntry` consumes the composition snapshot. Those paths, plus
the entry bootstrap and app-level parity tests, were inspected read-only and
remain outside this successor's write scope because their current attributions
belong to other archived sessions. The original `related_code` list is
therefore retained as historical evidence; it is not evidence that the old
app-only path is still the production owner.

No source edit is inferred from this intake. The next coordinator ticket will
check the owner-borrowed `EngineModule` contract, single-generation snapshot
guards, and the dynamic wrapper's absence of `Box::leak`; managed
`zircon_app --lib`/Runtime gates, report-versus-activation parity, repeated
entry reclamation at 1/100/1,000 and 1/1,000/100,000 scales, independent
review, fixed/return artifacts, and closeout remain pending.

### Current call-chain scope expansion

The same successor then applied transfer fingerprint
`bd1d3f39642a2abde25647958235040bd0029197d21f535344038ae01122f5ca` for the
current composition/entry implementation and its focused tests, extending the
audited write scope (without editing any source) to:

- `zircon_app/src/entry/engine_entry.rs`
  `14e61568cfec32e48713b051f7da98e4c78d2f36ac80f9d587a18d75bad77c80`
- `zircon_app/src/entry/entry_runner/bootstrap.rs`
  `d10a4596b8c46ffd7a54490a49d43d8ed75fa83705daa10476ab3d9ae4d05ac7`
- `zircon_app/src/entry/tests/builtin_engine_entry.rs`
  `45688a85c3c823fadfc640cbb9a3259cac3d9480eca9db71d4fa4b02f1b06882`
- `zircon_app/src/plugins/tests.rs`
  `fcd07dabc343cf552ebe3171a8e7752df73925c24a1b8813eaa5528ddd7fcd82`
- `zircon_runtime/src/builtin/runtime_modules/assembly/compiled_plan.rs`
  `323dea1de64354d2057b26eb28e3dd764ab486854d5040a4ec311dd4f78c9358`
- `zircon_runtime/src/builtin/runtime_modules/composition/outcome.rs`
  `4e183dbae7e450198ea89d894e106c072e4c13f69bc3a3385a0f833b9e3883b7`
- `zircon_runtime/src/builtin/runtime_modules/plugin_modules/descriptor_backed.rs`
  `9909f70a8b39b1519b5c5309d81137c1ed8b18421d137cf1e834b5714d4defa0`

This makes the validation snapshot cover the actual frozen-composition path,
rather than treating the historical app-only `builtin_modules.rs` path as the
whole fix. The source remains unchanged and the lifecycle remains open.

### Coordinator validation receipts

- The first submission was rejected during admission as
  `validation_ticket_dependency_roots_missing` (ticket/request record
  `12d99e3a0c5c449c8b30d2d956f3a863`); it did not execute and is not pass
  evidence.
- Corrected request: `failure-roll-01a084c8-runtime02-module-descriptor-20260919-r2`
- Ticket: `abeead056d3b45f0b70ed7cdcdd47748`
- Source-manifest hash:
  `ab5ae4ab804f68eefc09a10705b4cfa8ff73472dd7dd76a952b6a0dc3d85eff8`
- Admission: `queued`, execution kind `pending`, with no command output yet.
  The static contract covers the owner-borrowed trait, frozen plugin-group and
  runtime-composition snapshots, dynamic wrapper cardinality guard, entry
 report/bootstrap consumers, and focused generation-count tests. It does not
 satisfy managed Cargo, reclamation, parity, review, return, or closeout
 requirements.

The corrected ticket completed on managed job/run
`b28630d3768440cb8cf506510e94e60b` with exit code 0 and marker
`RUNTIME02_MODULE_DESCRIPTOR_SNAPSHOT_SOURCE_CONTRACT_PARSE_PASS`; cleanup
completed. This receipt is static current-source evidence only. Managed app
and runtime Cargo gates, descriptor reclamation/parity scale evidence,
independent review, fixed return and closeout remain pending.

## 2026-09-21 independent source review receipt

- Reviewer Session `review-runtime02-module-descriptor-r2` inspected the eleven
  current source/test paths in the successor manifest without editing them; all
  hashes still match the sealed manifest `ab5ae4ab804f68eefc09a10705b4cfa8ff73472dd7dd76a952b6a0dc3d85eff8`.
- The review re-ran `rustfmt +1.94.1 --edition 2021 --config
  skip_children=true --check` over the complete scope and a scoped
  `git diff --check`; both passed with markers `RUNTIME02_RUSTFMT_PASS` and
  `RUNTIME02_DIFF_CHECK_PASS`.
- The independent source probe passed as
  `RUNTIME02_MODULE_DESCRIPTOR_INDEPENDENT_SOURCE_REVIEW_PASS`. It verified
  the owner-borrowed `EngineModule` contract, one-generation descriptor
  freezing and replacement invalidation in `ResolvedPluginGroup`, activation
  order/module-descriptor cardinality invariants, nested snapshot tests,
  descriptor-owned dynamic text at 1/100/1,000 cardinalities with no
  `Box::leak`, composition snapshot reuse, compiled-plan handoff, and
  BuiltinEngineEntry/bootstrap/report consumers.
- Independent review result: **Critical=0 / Important=0 / Moderate=0**. No
  foreign composition, Frameworks05 preference, or entry-owner source was
  absorbed.
- This receipt remains static/source-only. Fresh managed `zircon_app --lib` and
  `zircon_runtime` Cargo gates, report-versus-activation parity, repeated-entry
  reclamation measurements at the required scales, and Runtime02 upward gates
  remain pending because external `E:\Git\zr_vm` is dirty. Canonical
  `fixed-*` return, closeout, and WeCom notification remain pending until those
  source-bound dynamic gates pass.

## 2026-09-26 successor intake (failure-roll-01a084c8-runtime02-module-descriptor-r3)

- The stale r2 lifecycle was not reused. Coordinator successor
  `failure-roll-01a084c8-runtime02-module-descriptor-r3` was registered with a
  doc-only write scope and acquired the failure-record lease after ownership
  transfer fingerprint
  `880b9d909c3d974fc1d6c00b9f84c310f54a6343eda032a3e0a3a543901023e8`.
  Snapshot `3914` sealed the pre-review boundary at failure-record SHA
  `afcb56293b5c9dde29f975d01c8788d495e98e599e29a4a93dac910becd8f96d`.
- This successor deliberately does not claim the eleven production/test paths
  from the r2 source manifest. The coordinator still attributes those paths to
  the archived r2 session, and three current blobs have changed outside this
  lifecycle: `zircon_app/src/entry/builtin_modules.rs` is now
  `f2a8ee2c27a85f040b0781c91d8a59be336d6ff55c3f23690b053ef2ebfe086e`,
  `zircon_app/src/entry/engine_entry.rs` is now
  `a36cf668e85e880c9a4d78b9e6abd02ec8d66f1837172ad8f211ec8638365890`, and
  `zircon_runtime/src/builtin/runtime_modules/composition/outcome.rs` is now
  `6d8446bd0d49767318a72ab4832c3969c02f09f33869509747ead0424ce604b4`.
  The other eight source/test hashes still match the sealed manifest
  `ab5ae4ab804f68eefc09a10705b4cfa8ff73472dd7dd76a952b6a0dc3d85eff8`.
  No foreign source edit is absorbed, and no source lease or attribution is
  claimed by r3.
- Consequently, the earlier managed ticket
  `b28630d3768440cb8cf506510e94e60b` remains historical static evidence only;
  its manifest no longer describes the complete current source chain. The
  rejected admission `12d99e3a0c5c449c8b30d2d956f3a863` remains non-evidence.
  A fresh source-bound ticket must be generated by the owner of the changed
  paths after their current chain is stable; r3 will not submit a duplicate or
  absorb that work.
- The r3 handoff is therefore documentation and evidence reconciliation only.
  Independent review must cover this successor record and the stale-manifest
  boundary, while managed `zircon_app`/`zircon_runtime` Cargo gates,
  report-versus-activation parity, repeated-entry reclamation at the required
  scales, Runtime02 upward gates, canonical `fixed-*` return, closeout, and
  WeCom notification remain pending. The failure remains open.

## 2026-09-26 independent successor review receipt

- Read-only independent reviewer `/root/review_editor03_gizmo_private` checked
  the current successor record. The failure-record SHA still matches intake
  snapshot `3915` (`dfc42632c2d768a4e9eb8df33d3b51ee5511dc9cfd854342e67f776f17c9e667`),
  and the doc-only scope, transfer fingerprint, stale-manifest boundary, and
  three foreign current-source drifts are explicit.
- The review confirmed that no source lease or foreign change was absorbed,
  historical/rejected validation tickets were not reused, and no dynamic pass
  or closeout claim is present. C/I/M result: **Critical=0 / Important=0 /
  Moderate=0**.
- This is an evidence/reconciliation review only; the current-source owner
  must issue a fresh source-bound ticket before any managed Cargo, parity,
  reclamation-scale, or Runtime02 upward result can be accepted. The failure
  remains open pending those gates and canonical return/closeout/WeCom records.
