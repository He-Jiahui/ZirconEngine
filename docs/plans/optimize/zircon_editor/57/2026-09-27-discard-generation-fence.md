---
title: Editor57 Active Scene Reload Discard Generation Fence
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: product_gate_pending
---

# Editor57 active scene reload discard generation fence

## Current defect and scope

This follows ED57-P0-01, ED57-P1-47 and ED57-G27/G28. The current source already uses the active scene identity, a typed background scene-open job and a Runtime project-generation commit fence. The historical default-scene replacement and retained UI `ProjectManager::open` / `scan_and_import` path are no longer the current defect.

The remaining defect was the unqualified `DiscardRequested -> Discard` policy. A dirty scene could publish a prompt, accept Discard, and receive another local edit before asynchronous completion. Ordinary edits do not change the lifecycle activation revision or necessarily publish a new Runtime asset generation. The host skipped dirty admission for Discard, and installation cleared the newer world/history too. The same authorization gap existed between prompt publication and choosing Discard.

Review also confirmed that a pending reload could finish after entering Play/Simulate: `scene_history_context()` then selected volatile Play history, whose dirty status is always false, while installation still replaced the authoring world. The final reload entry now defers in play mode before selecting any history context, including when the active selection remains in the Edit domain.

## Implemented contract

- A published prompt captures the actual `HistoryContextId` and transaction-engine generation in `SceneReloadDiscardAuthorization`. Choosing Discard compares that snapshot with current dirty history. A changed dirty history receives a new prompt; the old choice does not authorize its new generation.
- Admission retries and asynchronous scene loading retain the original authorization. A newer external asset generation may update its existing source-generation fence, but never refreshes the local discard authorization.
- Installation reserves the existing `ExclusiveTransition`, reads `HistoryStatus` without reentering its operation gate, and checks admission before cancelling an interactive transform, changing selection, clearing history or replacing the world. Discard always requires matching history identity and generation, including when intervening edits have subsequently been saved or undone to a clean state. A reload without discard authorization requires the current history to be clean.
- An active gizmo session or handle drag returns `Deferred` before admission or mutation. Its uncommitted preview does not advance history generation, so it needs this separate check. The retained pending request keeps the same prepared authoring seed, ticket, identity, discard token and coalesced reload flag. Later ticks only recheck installation: there is no second scene job, read or preparation. Committing the preview advances generation and requires a fresh decision; cancelling it permits the original matching authorization to resume. A clean preview also defers without entering a conflict/reload loop.
- Play/Simulate also retains that prepared request via `SceneEditingDisabledDuringPlay -> Deferred`. It cannot clear Play history or install an authoring scene while play is active. After Stop, the same request is admitted against the authoring Document history, so a dirty authoring scene still requires its decision.
- A competing transaction cannot commit between that check and replacement. The existing exclusive-entry operation-group flush preserves and commits already applied edits; the guard observes their resulting generation and never treats the flush as authorization to discard them.
- A rejected install maps to the existing host `Conflict` outcome. Dirty history gets another decision; an already clean history queues a fresh clean reload instead of installing the obsolete prepared world. Save, Keep Editing, active-scene identity supersession and Runtime project-generation supersession keep their existing owners.

The cross-module contract is documented in `docs/crates/zircon_editor/core/project.md` by the parent session. The implementation changes six existing production files: transaction `exclusive_transition.rs`, workbench startup `editor_state_project.rs` and its module binding, host `editor_scene_document_submission.rs`, retained `assets/workspace.rs`, and `workspace/active_scene_reload_conflict.rs`.

## Reference evidence

Local Unreal source `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageTools.cpp:734-835` separates dirty packages from reload candidates and requires a user decision before admitting them. Its synchronous/modal flow is reference evidence for explicit discard ownership; it does not provide Zircon's asynchronous generation protocol.

The concrete Rust contracts come from the existing `close_prompt/model.rs:118-136` decision-generation restriction, `transaction/save_token.rs` history-generation ownership, `transaction/lifecycle.rs:35-53` exclusive operation, and `core/document/scene_reload.rs` lifecycle identity fence. No new world owner, coordinator, worker or public generic transition API was introduced.

## Real regression coverage

The initial twelve regressions were authored before the production repair; review added the saved-newer-edit case, three real gizmo preview cases, and two play-mode cases while tightening the guards. Managed red/green execution has not run in this slice; source inspection is not substituted for execution.

| Layer | New regressions | Assertions |
| --- | --- | --- |
| Transaction engine | 2 in `src/tests/editing/transaction_engine/exclusive_transition.rs` | Exclusive history observation retains the operation gate against a real competing thread; clear and Undo generations remain observable. |
| EditorState | 7 in `src/ui/workbench/startup/editor_state_project/reload_tests.rs` | Stale discard preserves world/selection/Undo/Redo; exact authorization commits; clean preparation followed by an edit rejects; equal generations in different histories do not match; ordinary saved clean reload works; a newer edit saved to clean does not validate an old discard token; playing with Edit selection defers before resolving history and still rejects dirty history after Stop. |
| Retained host on Windows | 9 in `src/ui/retained_host/app/assets/workspace/reload_tests.rs` | Real CoreRuntime, ProjectAuthority, notification decisions and scene-load jobs cover edit-after-prompt, edit-after-submission, unchanged authorization, A-to-B-to-A scene activation and project retirement with an already deferred seed. Three pointer press/move/release/cancel cases wait until the real job has produced a prepared seed, then verify preview preservation, stable ticket/seed retention, commit-to-conflict, cancellation-to-resume and clean-preview deferral. A Play/Simulate test attaches an independent real InProcessGateway level and commits an actual Play history command: both selection domains preserve the pending seed, authoring world and both histories until Stop, after which dirty Document history produces Conflict and remains undoable. Keep Editing after a refreshed prompt submits no reload. |

Fixtures create secondary scenes through ProjectAuthority before watcher admission. They do not copy the production state machine, read source text, or manufacture a fake scene-load result. The retained Windows restriction follows the existing project-creation fixture environment.

## Evidence and acceptance

| Gate | Status |
| --- | --- |
| Exact shared scope | Eleven owned paths are outside sealed Batch N; claim `a27c61e030874531aedc7bbccacf183e`. Preimages contain original bytes and an empty preexisting diff for all seven existing owned Rust files. |
| Record policy | Optimize authorization `0d112aceb07f4bd2adb5be0714f61cab`; Astra authorization `11ded3b2e479447486823c2cefe069f3`. |
| Static review | Independent source review is complete, including the saved-clean, gizmo-preview and Play/Simulate repairs. Passing scoped rustfmt/diff checks and byte-preservation evidence are recorded under `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-o-editor57-discard-*`. |
| Managed behavior | Pending the grouped Runtime and Editor library suite, which includes the `reload_tests` and `exclusive_transition` groups. No direct Cargo, validation submission or coordinator compilation monitoring was performed by this implementation agent. |
| ED57 product/performance acceptance | Pending. This correctness repair does not certify the native 100,000-item p95/p99 gates or remove the existing synchronous authoring-world preparation cost. |

The extra commit guard reads one history generation and status; it does not scan assets or history records. The generation owner uses its existing map. One existing pending reload may retain one prepared authoring seed while a gizmo gesture or Play/Simulate blocks installation; the existing maintenance poll retries its guarded commit without loading or preparing the scene again. Actual frame latency remains an integrated measurement requirement.
