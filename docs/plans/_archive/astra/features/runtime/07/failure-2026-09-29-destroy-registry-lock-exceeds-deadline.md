---
handoff_kind: failure
status: open
created_at: 2026-09-29
summary_slug: destroy-registry-lock-exceeds-deadline
origin_plan: docs/plans/astra/optimize/01-review-and-repair.md
fixing_plan: docs/plans/astra/features/runtime/07-lifecycle-deadline-and-census.md
origin_child_dir: docs/plans/astra/optimize/01
fixing_child_dir: docs/plans/astra/features/runtime/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs
  - zircon_runtime/src/dynamic_api/session/registry/tests.rs
  - zircon_runtime/src/dynamic_api/session/registry/tests/destroy_registry_deadline.rs
tests:
  - cargo +1.94.1 check -p zircon_runtime --no-default-features --features dynamic-api --locked --lib --tests
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --features dynamic-api --locked --lib destroy_registry_deadline -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --features dynamic-api --locked --lib destroy_session_removes_registry_entry_only_after_event_mirror_quiescent_teardown -- --test-threads=1
---

# Runtime07: registry lock contention escapes the destroy deadline

## 来源执行者

- 来源计划：`docs/plans/astra/optimize/01-review-and-repair.md`
- 来源执行切片： LIFE-A1/A4 dynamic Session teardown deadline and retry audit.
- 修复责任计划：`docs/plans/astra/features/runtime/07-lifecycle-deadline-and-census.md`
- 交接原因： Runtime07 owns the session and allocation registries that define destroy timing and retention.

The Astra lifecycle review requires one absolute deadline across dynamic Session teardown and retryable retention when the deadline expires. Session `failure-roll-01a0df1a-runtime07-destroy-registry-deadline-r1` owns this lifecycle and the four exact source and test paths through audited transfer.

## 失败现象与复现证据

At the original source boundary, `destroy_session_slot_with_timeout` created a deadline before calling `find_session_slot`, but that lookup called the blocking session-registry mutex. The allocation census check called a second blocking mutex, and final session removal plus census cleanup locked the same registries again without a deadline. A thread holding either registry mutex could therefore keep a caller in destroy beyond its requested budget. This is source-level evidence; no managed Cargo result for this finding is claimed.

The regression must hold each registry mutex from another thread, call destroy with a short budget, and assert that it returns `teardown_incomplete` within a bounded tolerance while retaining the slot and census for a later successful retry. It must also cover final removal contention after the owner shutdown stage. The exact test filters above must execute nonzero tests on Windows through the managed coordinator.

## 最低共享层根因

The initial slot lookup, allocation census inspection, and final removal each use an unbounded mutex acquisition. The deadline is passed to action, wake, and owner shutdown stages but not to these registry stages.

## 架构修复验收

- Make initial slot lookup, allocation census inspection, and final registry removal honor the same absolute deadline used by action, wake, and owner shutdown stages.
- Keep the session slot and allocation census retryable on timeout. Recheck slot identity and zero outstanding allocations under the final locks before removing either record.
- Preserve invalid-handle and missing-session statuses, release-action behavior, and event-mirror quiescence semantics.
- Run the focused lower-layer contention and retry regressions, the original session lifecycle test, then Runtime07's declared managed Windows and product teardown gates on the current source snapshot.

## 禁止临时方案

Do not extend the caller's budget at a later stage, remove a slot before acquiring every required lock, discard an outstanding allocation, bypass the census, or treat static inspection as dynamic acceptance.

## 修复结果与回传

Open. A test-first source candidate now bounds these registry acquisitions and keeps the final slot/census removal atomic. Formatting and structural checks are local only; managed Cargo, independent final review, failure return, and closeout remain open.
