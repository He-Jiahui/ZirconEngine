---
handoff_kind: failure
status: open
failure_scope: cross_plan
created_at: 2026-08-15
summary_slug: world-sync-contract-guard-generation-lease-drift
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/02
plan_link_mode: child_record_only
related_code:
  - tools/tests/test_editor02_world_sync_watch_map_contract.py
  - zircon_editor/src/core/sync/pump.rs
  - zircon_editor/src/core/sync/pump/tests.rs
tests:
  - python -m unittest tools.tests.test_editor02_world_sync_watch_map_contract tools.tests.test_editor02_world_sync_subscription_table_contract
---

# Editor02: WorldSync source guard drifted from generation-guard implementation

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：MVP performance audit WorldSync generation-lease preflight
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：Editor02 owns the WorldSync generation protocol and guards; Performance01 exposed the stale source matcher but does not own the foreign sync sources.

## 失败现象与复现证据

The two current Python WorldSync suites ran 13 tests on 2026-08-15: 12 passed and
`test_live_runtime_token_collision_preserves_existing_editor_binding` errored before assertions.
The guard splits `pump.rs` on the retired literal
`let token = gateway.watch_world(registration.clone())?;`. Current source calls
`runtime.watch_world(registration.clone())?` inside `with_current_gateway_generation`, so the split
returns no suffix and raises `IndexError`.

The failing Python file is foreign modified and `pump.rs` plus its tests are foreign modified or
untracked. Performance01 did not overwrite either owner.

## 最低共享层根因

Editor02 owns the contract. Do not restore the retired direct gateway call or a redundant
post-transport generation synchronization merely to satisfy source text. Replace the brittle split
with behavior/structure checks for the actual generation protocol:

- allocation and local binding use the same captured gateway generation;
- a live same-generation token collision preserves the existing binding and does not unwatch it;
- replacement cannot cause a stale token to bind or revoke a current-session token;
- the final architecture performs foreign work outside the replacement mutex and commits only a
  current generation lease, as routed by the performance review.

The current Rust blocking-provider tests demonstrate the interim mutex behavior. The successor test
must be updated together with the generation-lease hard cut, not freeze that interim lock as the
target architecture.

## 架构修复验收

- Both Python suites pass all 13 current tests without matching retired call syntax.
- Rust tests cover replacement before/during/after delayed watch, unwatch and drain, token reuse,
  stale completion rejection and exact compensation.
- Source guard rejects old direct snapshot/drain races and foreign calls under the replacement mutex.
- Current managed editor Cargo is green from a non-C target root.

## 禁止临时方案

- Do not reintroduce `gateway.watch_world(...)` or an alias solely for the source guard.
- Do not weaken token collision, stale unwatch or replacement tests.
- Do not mark this fixed from 12/13 or static inspection; managed Rust behavior remains required.

## 修复结果与回传

Open state: Editor02 still needs to complete the generation-lease hard cut and make both Python guard suites plus managed Rust behavior GREEN before returning this handoff as fixed.
