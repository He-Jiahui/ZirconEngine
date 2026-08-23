---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-15
summary_slug: gameplay-scene-transition-ledger-sync
origin_plan: docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
fixing_plan: docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
origin_child_dir: docs/plans/zircon_runtime/runtime/13
fixing_child_dir: docs/plans/zircon_runtime/runtime/13
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/scene_transition.rs
  - docs/zircon_runtime/script/vm/host/function_ledger.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py
tests:
  - cargo test -p zircon_runtime --lib script::vm::tests::module_surface --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib script --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime13: gameplay scene-transition ledger synchronization

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md`
- 来源执行切片：gameplay scene-transition producer ledger synchronization
- 修复责任计划：`docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md`
- 交接原因：Runtime13 owns the producer descriptor, capability, ledger, and structural audit atom; Runtime10 separately owns the consuming scene-transition transaction.

## 失败现象与复现证据

The current source adds `request_scene_transition` and the
`gameplay.scene_transition` capability in
`zircon_runtime/src/script/vm/gameplay_host.rs`, with its implementation in the
untracked `gameplay_host/scene_transition.rs` leaf. The Runtime13 structural
audit on 2026-08-15 reports `gameplay_callback_count = 40` while the documented
and guarded baseline remains 39. All other Runtime13 boundary fields were
clean: 23/23 source files, 3/3 test files, 8/8 current guard files, no missing
ledger anchors, no native-ECS ABI references, and no oversized guard owner.

This is current-source contract drift, not a reason to raise the expected count
or suppress the audit. The new capability has not been accepted into the
published function ledger, module-surface coverage, or an owned Runtime13
milestone.

## Static Forward Progress (2026-08-15)

The complete current-source atom now reports 24/24 source files, 3/3 test
files, and 8/8 guard files. It records 40 gameplay callbacks and 13
capabilities with no missing source, guard, ledger-anchor, or Runtime13 guard
entries. `python -m unittest tools/tests/test_runtime_script_binding_audit.py`
passes after its two inventory expectations were synchronized to that current
audit.

This is structural evidence only. The producer atom is a bounded, latest-pending
request in the active World; it neither replaces a scene nor publishes a
terminal result. The declared managed module-surface and script Cargo gates
have not run against an immutable source snapshot, so this failure is not fixed.

## 最低共享层根因

The failure is local Runtime13 contract drift: the scene-transition producer was
added to current source before its capability count, function ledger, module
surface, and structural audit were synchronized as one owned atom. The separate
Runtime10 consumer boundary is not a substitute for this producer-side contract.

## 架构修复验收

- Keep the producer ABI, capability grant, canonical URI validation, host
  function descriptor, ledger, module-surface vector, and Runtime13 audit in
  one atom. The return value is a request id only; it must never be described
  as successful scene replacement.
- Keep `gameplay.scene_transition` capability-gated. Do not make it an
  unguarded gameplay callback, silently delete it, update the count alone, or
  add a script-side polling loop around the World resource.
- The Runtime10 [project-script-scene-transition-host-request](../10/failure-2026-07-19-project-script-scene-transition-host-request.md)
  handoff owns the consuming transaction: deterministic frame-boundary pickup,
  staged load/validation, old-world preservation on failure, lifecycle/focus
  handoff, duplicate/supersede policy, and terminal result publication.
- Run the declared managed Runtime13 validation gates after the producer atom
  is published as a source-complete snapshot. No Cargo result or Runtime10
  product transition is claimed by this handoff.

## 禁止临时方案

- Do not raise only the callback or capability count, suppress the audit, or remove the producer ABI.
- Do not make `gameplay.scene_transition` unguarded or describe a request id as successful scene replacement.
- Do not move Runtime10's frame-boundary consumer transaction into Runtime13 or add a script-side polling loop.

## 修复结果与回传

Open state: the producer descriptor, capability, ledger anchors, and Runtime13
structural inventory are synchronized in current source. The declared managed
Runtime13 Cargo gates and the separate Runtime10 consuming transaction remain
outstanding, so no fixed return or product scene replacement is claimed.

## Scope Boundary

Runtime13 owns the script producer and its descriptor/ledger/audit contract.
It does not own the Dynamic Session consumer or project-scene transaction; the
Runtime10 handoff above is canonical for that lower product boundary. This
record preserves the producer's static synchronization without pretending that
a World resource insertion completes a scene transition.
