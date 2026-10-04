---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: typed-asset-event-shared-bounded-dispatch
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/facade/event/declaration.rs
  - zircon_runtime/src/asset/facade/event/projection.rs
  - zircon_runtime/src/asset/facade/event/receiver.rs
  - zircon_runtime/src/asset/facade/event/tests.rs
  - zircon_runtime/crates/zr_resource/src/event_stream.rs
  - zircon_runtime/crates/zr_resource/src/event_stream/event_order_tests.rs
  - zircon_runtime/crates/zr_resource/src/event_stream/publication_index_tests.rs
  - zircon_runtime/crates/zr_resource/src/manager/resource_manager.rs
  - zircon_runtime/src/scene/dynamic_scene/asset_reload/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/asset_reload/queue/event_processing.rs
  - zircon_runtime/src/scene/dynamic_scene/asset_reload/queue/reconciliation.rs
  - zircon_runtime/src/scene/dynamic_scene/asset_reload/reports.rs
tests:
  - cargo test -p zircon_runtime --lib typed_asset_receiver --locked --jobs 1 -- --nocapture --test-threads=1
  - asset event storm, stalled consumer, rename/remove ordering and bounded RSS fixtures
---

# Runtime04：typed asset event共享有界分发缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset root/load/facade逐Rust文件性能审查，PERF-MVP-492
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：类型化facade的局部线程可直接删除，但resource/asset generation事件的容量、合并、顺序和consumer cursor属于Runtime04共享资产生命周期合同。
- 生命周期键：`typed-asset-event-shared-bounded-dispatch`

## 失败现象与复现证据

本轮已让`AssetEventReceiver<T>`直接消费底层resource receiver并原地过滤，消除每订阅一个`asset-event-filter-*` OS线程、shutdown通道和第二个无界队列。剩余`ResourceManager::subscribe`仍为每subscriber创建无界channel；`broadcast`持subscriber mutex逐项clone+send，慢或暂停的动态场景consumer会持续积压事件与RSS。

## 最低共享层根因

资源管理器没有generation-owned有界事件日志、consumer cursor、同resource revision合并规则或slow-consumer诊断；每个receiver私有队列成为无限历史truth。

## 架构修复验收

- 发布共享有界event log/ring，consumer以cursor读取；容量同时受entry/bytes/age预算约束，并暴露depth/age/coalesce/drop/lag诊断。
- 同`(resource kind,id)`可覆盖的Added/Updated按revision合并；rename/remove/reload-failed与生命周期边沿保持确定顺序，slow consumer可检测generation gap并重取snapshot。
- producer不在subscriber全局锁内执行N次channel send；订阅/退订和广播并发不阻塞资产发布热路径。
- typed receiver保持现有`recv/recv_timeout/try_recv`语义且不创建线程/二级队列；dynamic scene reload复用PERF-MVP-471预算与cancel。
- assets/subscribers 1/100/10k、events 1/1k/1M、stall 0/1/60s下RSS硬有界，producer p95不随历史积压增长，latest revision与边沿事件可验证。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止只把第二级typed channel改为bounded而保留每订阅专用线程和底层无界队列。
- 禁止静默drop rename/remove/failure，或让consumer无法识别cursor gap并恢复一致snapshot。

## 修复结果与回传

Open state: typed facade局部止损已完成；共享有界分发与产品storm验收仍`待修复`，no pass is claimed.

### 2026-09-09 current-source validation correction

Current source already provides the Runtime04 shared bounded resource event log:
an immutable publisher-owned cursor stream with entry/byte/age budgets,
revision-aware coalescing, lifecycle-edge ordering, lag gaps, and stream
diagnostics. `AssetEventReceiver<T>` consumes that receiver directly and filters
in place, so the typed facade does not create a filter thread or a second queue.

The declared typed facade regression was stale rather than a dispatch defect.
It registered `Pending` records (whose resource revision is `0`) and asserted
`revision() == 2`, confusing the second publication sequence with the resource
revision. The test now asserts the `Added` event and the contract revision `0`.

The first source-bound run on the prior input reproduced that exact failure:
job `f2b0bf0b1ebd40d3b0d455d5469d4b37`, 0 passed / 1 failed. After the test-only
correction, snapshot `3312` sealed the owned file with source manifest
`03e62d73910441d6bfecf66677d1ef1c0d6ccf5269ecae3098936a8f63729520`.
Windows native managed validation then ran the exact `typed_asset_receiver`
filter under Cargo 1.94.1, no default features, static linking, and `--locked`:
job `5585765421d948678c47137764abd1a0`, 1 passed / 0 failed / 0 ignored.
The receipt log is
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime04-typed-events-3312-20260909/results/runtime04-typed-events-3312-r1.log`.

Source and test evidence for this narrow stale-assertion repair is complete.
Independent C0/I0/M0 review, failure return, and coordinator closeout remain
pending; external `zr_vm` work remains excluded.

### 2026-09-11 canonical path correction

The original `related_code` entries above the current-source correction named
the pre-extraction paths `zircon_runtime/src/core/resource/manager/*` and the
`asset_reload` directory. Those paths are retained only as historical failure
evidence: they no longer exist after the Resource foundation moved to the
`zircon_runtime/crates/zr_resource` crate. The lifecycle key is unchanged.
The coordinator index now binds this lifecycle to the concrete facade,
`zr_resource` event-stream/manager, and dynamic-scene consumer files listed in
the frontmatter, including the exact regression tests. No compatibility alias
or second dispatch path was introduced.

### 2026-09-11 current-source Cargo admission blocker

The fixing Session `failure-roll-01a084c8-runtime04-typed-events-r1` sealed the
current canonical source manifest and submitted the direct structured Cargo
command
`cargo test -p zircon_runtime --no-default-features --locked --lib typed_asset_receiver`
with Cargo/Rust 1.94.1, Windows MSVC, and static linking. Coordinator request
`9148c1ff9bf34110a8972f42561a5260` was rejected during admission with
`validation_ticket_external_worktree_dirty`: the external repository
`E:\\Git\\zr_vm` has uncommitted changes. No validation ticket or Cargo run
was created, so this is recorded as an external prerequisite rather than a
pass. The Session is parked at `waiting_validation`; return, independent
review, and closeout remain pending until the external owner provides a clean
revision and the exact command is admitted and executed.
