# Coordinator Validation Efficiency

| Scope | Status | Date | Evidence |
|---|---|---|---|
| Validation preparation, focused Cargo stages, warm pools and diagnostics | Scoped validation passed; independent review clear | 2026-09-05 | Session `01a07063-6f03-7803-a12d-13ea015ca645`; measurements below |

## Assessment

The initial read-only sample since 2026-08-29 contained 750 terminal tickets:
52 passed, 606 failed and 92 became snapshot-stale. Of the failures, 448 occurred
in preparation/coordinator stages and 158 had a nonzero command exit. The dominant
preparation errors were missing compile-time resources (196 tickets), unmanaged
artifacts (83) and overlay ownership errors (55 across materialization/queue).
These counts do not imply that every missing-resource failure was spurious.

Ticket latency includes queueing and preparation. It must not be presented as
compile time. One recorded Runtime library check took 25m45.78s and exited 101
with 235 compiler errors and 364 warnings. This change does not repair that Rust
baseline or establish a new full-engine validation time.

## Delivered Changes

- Focused library/integration tests compile through `cargo test` without a prior
  product build. Explicit build/publish requirements keep their build gate.
- A failed build stops the dependent test stage. `-CheckOnly` provides a managed
  package check for diagnosing the build baseline; it does not replace test gates.
- Cargo input planning tracks test context per target. Dependency unit-test
  resources do not block unrelated focused tests; production and shared-source
  requirements remain checked.
- Unchanged Failure imports reuse one content-keyed parsed snapshot. Hash and
  controlled-action expectation checks still run; concurrent parses coalesce.
- Automatic Cargo pool retention increases from two hours to seven days.
  Existing disk-pressure eviction and process protection remain active.
- A read-only diagnostics command separates preparation, command and runner
  failures, actual run durations, stale tickets and reclaimable SQLite pages.

## Measurements

| Operation | Before / First | After / Repeated | Meaning |
|---|---:|---:|---|
| Failure snapshot preparation | 56.430s | 6.507s | 88.5% less time on unchanged content; 779 artifacts and 116 diagnostics identical |
| Managed `zircon_reflect_derive` check | 6.42s | 0.70s | Same target pool and Cargo home; small-crate warm-cache smoke only |
| Coordinator SQLite file | 2,742,210,560 bytes | 539,271,168 bytes | About 2.05 GiB reclaimed through managed retention compaction |

Managed Cargo jobs: `c1663b3badfe44f3997c6783ed2e7533` and
`06c671a524eb42c990778792256aa14a`. Compaction batch:
`manifest-retention-acd2bff81bc6d8b3cb74c376`; maintenance request:
`89b00b4381a942ec8f8395d22043026a` completed. The database can grow again as new
events arrive. No active build cache was manually deleted.

## Validation And Limits

Initial coordinator regression batch: 138 passed. After review fixes, the affected
input-closure, pinned-planner, test-scope and diagnostics batch passed 62 tests
(154.144s), including the 12 focused regressions. Independent re-review found no
remaining actionable issues. PowerShell stage/fail-fast tests:
9 passed. The complete existing PowerShell matrix suite passed 108/108 (89.663s)
after its feature fixture was aligned with the existing
`first-party-runtime-plugins` manifest feature.

Existing full-engine compiler failures, stale ownership and artifact governance
violations retain their actual failed status. The operational sequence is a
focused package check, repair of the first shared compiler failures, package
tests with a stable feature/profile combination, then the applicable integration
gate. Repeating full workspace tests against the same broken baseline is not
evidence of progress.

The shared checkout already contained foreign changes. This record covers the
efficiency delta and does not accept the entire numbered plan. The new
`.codex/skills/zircon-dev/scripts/validation-stages.ps1` and
`validation-stages.Tests.ps1` are covered by the blanket `.codex` ignore and must
remain in the coordinator-attributed, force-added integration scope.

The final backend changes are loaded by healthy instance
`7b6d8c4e92d245b181a71a66dbd3b461`. Rollover action
`2f624b6ed4954d489a53a477cd3aa555` durably succeeded after the existing managed
Cargo job released. The client timed out during startup, so completion was
reconciled from the exact persisted action and the new service health response.

Usage and maintenance contract: [local coordinator guide](../../../../cli-and-tooling/local-session-coordinator.md#validation-efficiency).
Cargo target semantics: [official Cargo test reference](https://doc.rust-lang.org/cargo/commands/cargo-test.html#target-selection).
