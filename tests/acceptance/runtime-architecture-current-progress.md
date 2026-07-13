---
related_code:
  - zircon_runtime/src
  - zircon_app/src/entry
  - zircon_runtime_interface
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
output_records:
  - docs/plans/zircon_runtime/runtime/02/2026-07-11-runtime02-current-cargo-baseline.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-11-stable-evidence-owner-hard-cutover.md
status: runtime01_complete_runtime02_14_owned_gates_mostly_accepted_program_in_progress
---

# Runtime Architecture Current Progress

Date: 2026-07-11

This matrix separates implemented Runtime-owned scope from broad gates that
still include active Render, Runtime UI, sample-project, or external ZrVM
environment ownership. A focused green result accepts only the listed child
plan boundary; it does not silently mark the complete Runtime program green.

| Plan | Current result | Completed implementation / accepted evidence | Remaining gate |
|---|---|---|---|
| Runtime 01 | `completed` | Five dependency/runtime/plugin Cargo gates and the direct dependency audit are green. | None in declared Runtime 01 scope. |
| Runtime 02 | `in_progress` | Core spine, root surface, generated boundary, and app package gate are accepted. | Broad core filter retains 10 Render/UI-owned failures. |
| Runtime 03 | owned gates accepted | Schedule 77/77, time 4/4, session 162/162 with 10 documented ignores, parallel 15/15; app 135/135 plus one documented ignore. | Real ZrVM environment cases and full-program closeout. |
| Runtime 04 | owned gates accepted | Asset structure audit is risk-free; watcher 19/19 and worker 17/17. | Broad asset has five sample/Render/UI failures. |
| Runtime 05 | focused gates accepted | Scene-patch 5/5, dynamic-scene guards 8/8, owner tree 2/2, mobility and freshness 1/1 each. | Broad scene rerun terminated with access violation; mesh ordering and Render/text failures remain. |
| Runtime 06 | owned gates accepted | Plugin surface 5/5, VM fallback 5/5, native plugin 139/139; poisoned FFI test mutex handling and shared-payload fixture serialization implemented. | Cross-package plugin gates and real ZrVM gate. |
| Runtime 07 | ECS-query, durable performance-guard, and profiling-build gates accepted | ECS query 58/58; current-source performance-hotspots 28/28 with parent-and-child no-session-evidence coverage; risk-free performance audit; and two optimized `profiling,profiling-chrome` library builds completed. The second build exited 0 in 1902.799 seconds and localized the bottleneck to `zircon_runtime` release optimization/codegen. | Extract has Render/UI/scene failures; two-run FPS remains blocked by the unavailable current ZrVM link library, and the trace test was stopped before execution when its optimized lib-test compile reduced D: free space to about 1.09 GiB. |
| Runtime 08 | owned gates accepted | ECS 340/340 plus observer/command/message/change-tick filters and risk-free structure audit. | Entity filter retains one Render neutral-LUT failure. |
| Runtime 09 | structure gates accepted | UI architecture package guard 20/20, root/surface maps 19/23, naming 100/100, risk-free audit. | Active UI behavior/layout/render failures remain. |
| Runtime 10 | Runtime filter accepted | Dynamic API 93/93 with 10 documented ZrVM ignores; app gate previously 135/135 plus one ignore. | Full interface/app matrix and real ZrVM environment. |
| Runtime 11 | owned gates accepted | Tasks 22/22, ECS schedule 77/77, worker 17/17, job 14/14, Rayon 5/5. | Prescribed full Runtime lib regression. |
| Runtime 12 | owned gates accepted | Action map 8/8, gamepad 27/27, risk-free input audit. | Broad input retains 13 active UI failures and one ignore. |
| Runtime 13 | owned gates accepted | Current package script result is 354/356; all owned route and ledger guards are green. The Animation owner has since closed the state-machine typed-fallback handoff with focused passing evidence. | Current Runtime package rerun, Render pipeline feature descriptor failure, and full regression. |
| Runtime 14 | partially accepted | Diagnostics 15/15, engine module 7/7, risk-free family audit; state-machine current roundtrip/v1 migration harness 2/2. The Animation owner closed the typed-fallback handoff and reports post-transition 77/77 plus later production bridge 78/78. | Runtime package animation rerun; navigation retains three UI failures; full lib/app regressions. |
| Runtime 15 / priority plans | priority review, stable-evidence cutover, and current naming-debt cleanup accepted | `code_review_findings` is 298/298; all 24 staged Runtime structure child-audits report zero explicit risks. Asset pipeline count and current artifact-cache path guards pass focused. All 456 session-note path consumptions were removed from 449 guards with no fallback. The naming boundary is now `classified` with zero debt/unclassified entries, and the module-convention gate is `classified-and-clear` with zero migration debt and `risks=[]`; the five former locations were test-only fixture/name cleanup. | Latest current-source standalone is 1297/1304 after the full-family hard cut. Seven active Render/UI production/test budgets, workload owner, and deferred-lighting dispatch failures remain; Cargo verification for the naming-only slice is deferred by the shared resource gate. |

## Priority-plan decision

The requested priority review plan is green at its current-package boundary.
The structure-convention plan is not marked complete: its four current failures
remain explicitly visible and are not bypassed with compatibility paths or
stale status mirrors.

A prior recompiled standalone structure rerun was 1224/1303. Three removed
Render/Text session-note inputs caused 71 direct read failures. The stable
evidence hard cut now removes that false dependency without restoring notes or
adding compatibility lookup; the latest current-source result is 1297/1304.

The one Runtime-owned failure in that eight was a stale asset-pipeline manager
child-test count after a second-manifest-root watcher test landed. Its guard now
expects 12 child tests, locks the new test name, and passes focused 1/1; the
other seven remain Render/UI-owned.

A monolithic repository-wide runtime-interface structure-audit retry produced
no JSON before its 124-second timeout. A staged execution of the same audit
functions then completed in 128.5 seconds: all 24 Runtime child boundaries
reported `risks = []`; at that historical starting point the aggregate naming
gate was blocked by four UI test-fixture `editor` strings and one graphics test
name carrying `legacy`. The test-only, same-length fixture/name cleanup has now
removed all five locations. A fresh full audit covers 37 top-level groups with
no non-empty risk group; naming is `classified` with empty debt/unclassified
lists, and module convention is `classified-and-clear` with zero migration
debt. This does not broaden the Runtime 15 completion claim: seven Render/UI
structure failures remain visible.

The 71 missing-note failures were traced to unstable evidence ownership. The
72 affected files already had canonical plan inputs, so their retired-note
reads and tuple consumers are removed directly and a recursive regression guard
prevents those paths returning. Representative former failures and the new
guard pass focused; seven Render/UI-owned structure failures remain visible.

## Program decision

The architecture implementation remains `in_progress`. Runtime 01 is complete,
and most Runtime 02-14 owned boundaries are accepted, but the program-level
definition of done still requires the listed cross-owner regressions, real ZrVM
environment evidence, Runtime 07 performance artifacts, and a full package
regression without external failures.

The final coordinator audit reports zero diagnostics under
`docs/plans/zircon_runtime/runtime`, zero invalid session statuses, and zero
unsafe cleanup candidates. Its overall baseline is still degraded by 35
external handoff-record diagnostics: 30 under Editor and five under Render/Text.
Those are retained as external evidence and are not rewritten by this Runtime
plan session.
