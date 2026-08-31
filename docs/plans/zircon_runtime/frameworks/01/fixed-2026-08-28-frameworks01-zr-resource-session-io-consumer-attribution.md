---
handoff_kind: fixed
status: fixed
created_at: 2026-08-26
resolved_at: 2026-08-28
summary_slug: frameworks01-zr-resource-session-io-consumer-attribution
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session/io/atomic.rs
tests:
  - .\tools\zircon-session.ps1 ownership matrix --prefix zircon_runtime/src/scene/dynamic_scene/session/io/atomic.rs -Json
---

# Runtime11: return executable attribution for the Resource session-I/O consumer

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：M1 `zr_resource` current-source atomic scope rotation
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：Runtime11 owns the bounded session-I/O lifecycle that introduced and attributed this
  exact consumer; Frameworks01 does not own that operation's persistence semantics.

## 失败现象与复现证据

`zircon_runtime/src/scene/dynamic_scene/session/io/atomic.rs` is the sole consumer added after the
earlier 502-file Resource inventory. Its current SHA-256 is
`f946f2e2fe0cb55e7e5560d283345cb7a153485c6a701a7b37a7663a0abcc44d`, while the stored Runtime11
attribution hash is stale and Session `runtime11-bounded-stream-root-20260826` remains
`resolving_failure`. Ownership transfer therefore returns `source_owner_executable`; no live lease
overlaps the consumer union.

## 最低共享层根因

The lowest proven failure is the executable Runtime11 attribution lifecycle for a current-source
consumer. It is not evidence that the durable Resource I/O API should be duplicated or bypassed.

## 架构修复验收

- Preserve the exact current blob and terminalize or return its Runtime11 ownership through the
  coordinator after the bounded session-I/O owner has reconciled it.
- A fresh exact-path transfer preview must match the current hash and contain no
  `source_owner_executable` reason.
- Frameworks01 must retain the file in the refreshed 503-consumer manifest and rerun the structured
  scan before successor registration.

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses,
  or call-site exceptions.
- Do not change session-I/O behavior merely to change ownership, and do not copy durable Resource
  I/O into the Scene owner.

## 修复结果与回传

- 根因：Runtime11 source Session expired to stale; its prior executable attribution gate no longer represented a live owner, and the consumer blob advanced after the original handoff hash.
- 架构修复：Re-previewed the unchanged latest current session-IO consumer and transferred it atomically to Frameworks01 using fingerprint ecccb84f96c9b674b42377e1534f6bdb62c05cd08aa4a7e33b1e995965f8204d; no behavior or durable IO facade change.
- 验证：Post-apply ownership matrix matches current hash 04d20053813bd6f7b545d02379c2a67f26959d5d169de4b7a38212f212e7cedc with a live Frameworks01 lease, zero blocking reasons, and integration_ready state.
- 回传：Runtime11 session-IO consumer attribution blocker fixed and returned; full zr_resource hard cut remains blocked by eight MVP00 consumers.
