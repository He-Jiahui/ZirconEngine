---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-24
summary_slug: powershell-wrapped-cargo-validation-lane-bypass
origin_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
origin_child_dir: docs/plans/optimize/zircon_tooling/06
fixing_child_dir: docs/plans/optimize/zircon_tooling/06
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/validation_ticket_worker.py
  - tools/session_coordinator/tests/test_validation_tickets.py
tests:
  - python -u -B -m unittest tools.session_coordinator.tests.test_validation_tickets.ValidationTicketTests.test_worker_routes_cargo_toolchain_wrappers_through_a_cargo_workspace_copy -v
  - python -u -B -m unittest tools.session_coordinator.tests.test_validation_tickets.ValidationTicketTests.test_worker_does_not_route_a_not_required_cargo_marker_to_cargo -v
  - python -u -B -m unittest tools.session_coordinator.tests.test_validation_tickets -v
resolved_at: 2026-08-24
---

# Tooling06: PowerShell-wrapped Cargo validation bypasses the Cargo lane

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 来源执行切片：validation FIFO current-source lane and terminal-evidence review
- 修复责任计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 交接原因：Tooling06 owns validation-ticket execution classification and the immutable-copy/Cargo-lane boundary.

## 失败现象与复现证据

Validation ticket `85a4a2ca5c434ae3a644f47f8d874af9` declared
`toolchain.cargo=1.94.1` and executed Cargo from a PowerShell `-Command` wrapper.
The worker classified only `command[0]`, so it linked generic validation copy
`ca8150ce1c07406e8ea12ad7b387e251` with `materialization_kind=null` instead of a
Cargo copy. The run spent about 52 minutes compiling from the generic copy and
terminalized with Cargo exit 101, but no Cargo-lane job identity governed that
execution. Its downstream Runtime compile error is foreign current-source debt
and is not part of this failure.

Ticket `0294e3a635da42c4a34696669a9051ff` independently preserves the same command
shape and immutable Cargo toolchain declaration. It terminalized
`snapshot_stale` before materialization and therefore is classification evidence,
not a product validation result.

Focused RED reproduced the defect with a PowerShell entrypoint and a durable
`cargo` toolchain declaration: the worker populated `generic_materializations`
instead of `materializations`.

## 最低共享层根因

`ValidationTicketWorker._is_cargo_command()` recognizes only a direct `cargo` or
`cargo.exe` executable. Validation tickets already persist `toolchain_json` as
part of their dedupe identity, but the worker ignores that declaration when
choosing the execution lane. Shell wrappers that prepare pinned external inputs
before invoking Cargo consequently bypass Cargo materialization and reservation.

## 架构修复验收

- Direct Cargo commands continue to use Cargo materialization without requiring
  duplicate metadata.
- A wrapper whose immutable ticket toolchain contains `cargo` as
  a usable string Cargo identity uses `materialize_cargo_async`, even when
  `command[0]` is PowerShell.
- Sentinel metadata such as `cargo: not_required`, an empty value, or boolean
  `false` does not grant the Cargo lane to an otherwise generic command.
- The same classification controls removed-copy recovery so a Cargo wrapper is
  never restarted as an ordinary generic copy.
- A PowerShell/Python validation that does not declare Cargo remains on the
  generic dependency-root path.
- Focused and complete validation-ticket suites pass, then a committed and loaded
  worker must produce a real wrapper ticket with `materialization_kind=cargo` and
  normal Cargo terminal evidence before this record returns fixed.

## 禁止临时方案

- Do not parse arbitrary PowerShell source text for the word `cargo` or maintain a
  shell-specific command allowlist.
- Do not route every generic validation through Cargo or weaken dependency-root
  closure for non-Cargo tools.
- Do not rewrite prior ticket/copy evidence, synthesize a Cargo job for the old
  generic run, or claim its foreign compile error is fixed.

## 修复结果与回传

- 根因：Validation classified declared Cargo wrappers incompletely and WorkspaceCopyService then launched their linked immutable copies outside the durable CPU reservation and CargoJobRunner lifecycle; the first lane repair also omitted the ticket selected source manifest required by source-copy admission.
- 架构修复：Route durable Cargo declarations through Cargo materialization, execute linked validation copies only through an exact FIFO CPU reservation and CargoJobRunner, and bind the ticket source manifest plus immutable copy job and full input-manifest hash into reservation, Cargo job, run, and terminal projections.
- 验证：Focused validation-copy Cargo tests 5/5 and Python compile passed; daemon successor 218011aebe7e416593b93cb51fc196e4 loaded commit 1b2684b40; production ticket 235eba0a940e4ab28e1cc3d9bfe2a415 passed with copy 2d7467d431564dc8a3615c7529bc4cdf, reservation 275e9c1dc6d94923a96f7e0bec050174, Cargo job b224f0b9d2fd42f88c719bc3f2a1a728 released exit 0, Cargo run fd03a172d06844e5ba039d4ed5f0c1f3 completed error-free, and identical c6385f8a input identity.
- 回传：PowerShell-wrapped declared Cargo validation now materializes an immutable Cargo copy, waits in FIFO, runs only under the Coordinator Cargo process tree, and durably preserves source-copy and selected-manifest identity through terminal evidence.
