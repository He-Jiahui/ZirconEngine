# Runtime 10 Reactive Frame Wake V3 Current-Source Record

status: in_progress
date: 2026-07-22
base_head: 9cbc07ca2316f752b05dbef95ade9d70e893afeb
baseline_epoch: 210

## Scope

This is the canonical numbered-plan record for the approved Runtime10/Runtime03 V3-only reactive frame wake hard cut.

Authoritative design and execution plan:

- `docs/superpowers/specs/2026-07-18-runtime-reactive-frame-wake-v3-hard-cut-design.md`
- `docs/superpowers/plans/2026-07-18-runtime-reactive-frame-wake-v3-hard-cut.md`

The milestone remains one atomic Runtime10/Runtime03 source and commit boundary. This record does not authorize a V2 fallback, a constant-Idle V3 placeholder, an isolated producer commit, or a fixed return.

## Completed Items

| Item | Current evidence | State |
|---|---|---|
| V3-only interface/export/app/editor migration candidate | Current production scan has no V2 table, symbol, or loader identifier; ABI layout, app cadence, host token registry, editor gateway, and runtime lifecycle diffs are present in the shared checkout | `static_candidate_uncommitted` |
| Session action and wake quiescence | Folder-backed `SessionSlot`, `RuntimeWakeRegistration`, demand accumulator, closing admission barrier, callback drain, and destroy ordering are present | `static_candidate_uncommitted` |
| Registry structure hard cut | snapshot 693 exact10; flat owner deleted, `registry/mod.rs` zero behavior, `session_store.rs` behavior owner; independent review Critical/Important/Minor 0/0/0 | `static_reviewed_pending_atomic_validation` |
| Registry owner documentation and Runtime15 mirrors | snapshot 698 exact2 plus snapshot 699 exact9; bounded lifecycle guards and zero-behavior owner docs; independent review Critical/Important/Minor 0/0/0 | `static_reviewed_pending_atomic_validation` |
| Active-animation producer | snapshot 702 established the managed RED; snapshot 732 exact10 adds existing-scan OR aggregation for clip, sequence, graph, and state-machine players, asset/manager early-return coverage, LevelSystem Idle/Immediate mapping, real registry-handle ABI coverage, and failed-tick state rollback; independent review Critical/Important/Minor 0/0/0 | `implementation_static_reviewed_waiting_managed_green` |

## Managed Evidence

| Evidence | Result |
|---|---|
| Runtime10 animation RED | job `4378aef4845148e5bcb922286a1a65d8` / run `f6675db5410a45889302b97a12c20c45`, released `exit 101`, no live PIDs; exact10 compiler errors proved the missing `animation_frame_demand`, LevelSystem record method, and scan flag. The focused test did not execute, and the other shared-tree compile errors are not feature evidence. |
| Runtime10 animation implementation | snapshot 732 exact10, baseline epoch 214; source manifest fingerprint `ee828a39502419f50636479d269537c233d6f727e7c3ed30c2abf462da527feb`; rustfmt and diff-check passed; independent review Critical 0 / Important 0 / Minor 0 |
| Runtime10 animation green attempt | reservation `5ca8bd76b7a34316814b7734e8c61747`, job `04edf97fb8d74fa988ef7639137314b3` / run `7cb8bb602497464486c2344d6ec5c697`, released `exit 101` with no live PIDs after 36m42s. The focused test did not execute (`stdout` target matches: 0). The lib-test build stopped on 14 errors outside the animation exact10: six Plugin01 feature-projection visibility/import errors, two Text01 missing-`FontQuery` errors, four expected Runtime09 timer RED errors, and two Frameworks05 unsupported `FilesystemQuotaExceeded` errors. No green evidence is claimed. |
| Resource + Frameworks01 + Runtime11 prerequisite | successor `frameworks01-runtime11-resource-current-source-atomic-prerequisite-r7-20260722`, historical snapshot 738 exact57; Frameworks01/02 guards, Runtime11 audit, rustfmt, and diff-check passed for that snapshot. A 2026-07-22 05:42Z freshness audit found current hashes at 56/57 because `11-job-system-task-model.md` gained later performance handoffs; the three Plugins01 mirrors also still lack owner SHA/fixed handoff. Exact60 is not frozen; no Cargo, final review, failure return, or commit is claimed. |
| Scene-asset arrival/terminal wake audit | 2026-07-22 05:33Z read-only exact20 (11 production, 5 test, 4 docs) confirms both wake stages are absent. All 20 paths had no live lease, but seven remain in declared active scopes, so no implementation ownership or completion is inferred. Audit risk: Critical 1 / Important 4 / Minor 2; no Cargo/pass claim. |
| Registry/docs scoped formatting, static assertions, and diff checks | passed; no Cargo acceptance claim |
| Registry/docs independent review | snapshots 693/698/699, Critical 0 / Important 0 / Minor 0 |

No Runtime10 managed Cargo gate has passed for the current atomic V3 candidate. No milestone commit SHA exists.

### Scene-asset arrival exact20 current-source manifest

This manifest is read-only audit evidence, not a lease, implementation handoff, validation snapshot, or acceptance claim.

| Kind | Path | SHA256 |
|---|---|---|
| production | `zircon_runtime/src/core/resource/manager/resource_manager.rs` | `221215657e5aa5e1a4cb8fc037367be6e2b52005564e7bf8af85abd33e88dde0` |
| production | `zircon_runtime/src/core/resource/manager/events.rs` | `866aed6d786553a60ec0f4eadad1347e405a1f6fb56a719308c4a9798d8c681f` |
| production | `zircon_runtime/src/asset/facade/event.rs` | `bfc1889a71535bec37c020a7eda54a6b266cde44d9df4c00a32eb5d810a0f7eb` |
| production | `zircon_runtime/src/asset/facade/assets.rs` | `f492e79e8495d48c5e6b49e6dddf694d9a719f175a1c1661fe7eedc699422032` |
| production | `zircon_runtime/src/asset/facade/manager.rs` | `9e52f8abead32b3cf54e24cb7cac2243c0c8fc01dd99d066f4137cb4d0a54892` |
| production | `zircon_runtime/src/scene/dynamic_scene/asset_reload/queue.rs` | `095eb5874230ebaf78a75f260cb63e08c5a2de065a0485e5fc3aeb1e591aad9d` |
| production | `zircon_runtime/src/scene/dynamic_scene/asset_reload/task.rs` | `6114a3c4b31b68bd3edb20af9c82ba971a368519e35dfc7659b10b245a9a2e70` |
| production | `zircon_runtime/src/dynamic_api/session/project.rs` | `8d6ca06b843d09928f430bdce237b4fa5324b849c9233cc9bb63ca3855b4786a` |
| production | `zircon_runtime/src/dynamic_api/session/construction.rs` | `fd615c57004acb5d9414661b18b424e12b5da277519be56e2c40fc34ac13451a` |
| production | `zircon_runtime/src/dynamic_api/session/state.rs` | `9938fefbe927046fd8b92442db07d78ba4a4a614b0a006d06fcb4879dd6caebd` |
| production | `zircon_runtime/src/dynamic_api/session/ffi.rs` | `8bda9b8a6260c02a4f94b420eb1c12c277d7b8ffd085dbe20ef229cd9eb7201a` |
| test | `zircon_runtime/src/asset/tests/facade/handle_events.rs` | `183b69f32b15ab0f304cdad5f9a78cda08dd85275fc547bafcf69d1512c30aa7` |
| test | `zircon_runtime/src/asset/tests/pipeline/manager/watcher.rs` | `edfa0df3a4b378f42ba5d4fdfa208cbf03c20eb5502056522e1c2eaf62f1f384` |
| test | `zircon_runtime/src/scene/tests/dynamic_scene_asset_reload.rs` | `26fd4cb30aaf1d04af866140e439e69939c2ed6b8f78d99e02f9f1b786a03eee` |
| test | `zircon_runtime/src/dynamic_api/session/tests/frame_demand.rs` | `385a09b96593bff2bf78dd40d135722d9972ab89f5466dddd269a0f9218c8419` |
| test | `zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs` | `d5197adeb17dbd7ea5ac32075ce61c6cef9ad0c656952543938f37511b3cca73` |
| docs | `docs/zircon_runtime/asset/facade.md` | `04c5a7b8bff2580858fb76e41708d7feb21dabd1ff9a06d1d1f4e85f8b65cf35` |
| docs | `docs/zircon_runtime/asset/watcher.md` | `ac3e3a5e4af27c356e55081b4618d07e144b72621c13f811c75c71132faf2bfc` |
| docs | `docs/zircon_runtime/scene/dynamic_scene.md` | `995c38bd99242898f68feb6307e940eb11788f7a200c8689a9ab38ababb78482` |
| docs | `docs/zircon_runtime/dynamic_api/session.md` | `c33192246e57f7ce1e0ef79ac4f8cce5dd544191a8372470de62797716d51adf` |

## Remaining Items

- Retry the source-bound snapshot 732 focused green only after the four lower owner groups from job `04edf97fb8d74fa988ef7639137314b3` are current-source compilable; then include the producer in the final atomic Runtime10/Runtime03 gates. The failed attempt did not execute the target and is not green evidence.
- Land the Runtime09 production UI-surface owner, then route its real timer deadlines into the same-tick earliest-delay demand. Current `UiInputManager` timer state is not constructed by production code, so a test-only timer must not be presented as the producer. Double-click candidate expiry is not frame-visible and must not manufacture a deadline.
- Runtime11 terminal-observer work is an atomic prerequisite with its lower ResourceRegistryError and Frameworks01 typed-error owners because no independently compilable intermediate commit exists. Snapshot 738 is now historical rather than current-source: the Runtime11 plan path drifted after later performance handoffs, and the Plugins01-owned three mirror documents still have no SHA/fixed handoff. Original owners must reconcile and re-freeze exact60 before Cargo/review/commit. After that atomic prerequisite commit, attach the observer only to `DynamicSceneAssetReloadQueue` tasks whose results remain queue-owned and frame-visible; preserve destroy quiescence and suppress superseded/dropped-task wakes. Do not add a scheduler-wide or generic `DynamicSceneSpawnTask` wake.
- Add a session-scoped asset-event arrival notifier as a separate real producer after all active sessions covering the seven declared-scope paths hand off their current diffs. The 2026-07-22 05:33Z exact20 manifest above has 20/20 live leases free but only 13/20 declared scopes free; `lease=0` is not authorization to absorb the other seven paths. Current production has neither stage: typed `SceneAsset` enqueue does not wake the idle session, and the queue does not register the existing `JobHandle::on_terminal` primitive. The required contract remains two-stage: enqueue wakes once to schedule work, then eligible queue-owned task terminal wakes once to collect/apply; terminal-only wake does not close the idle file-watch-to-schedule gap. First RED: `scene_asset_event_arrival_wakes_idle_session_before_reload_task_is_scheduled`; second RED: `queue_owned_scene_asset_reload_terminal_wakes_once_and_suppresses_superseded_task`.
- Obtain the ResourceRegistryError + Frameworks01 + Runtime11 current-source prerequisite managed commit before atomic Runtime10/Runtime03 acceptance; do not restore deleted error owners or add compatibility shims to manufacture an intermediate commit.
- Reconcile all V3 runtime-absorption mirrors and authoritative docs that still describe V2.
- Freeze the final atomic manifest only after zero-mod, producers, mirrors, docs, app, editor, interface, and lifecycle owners are current-hash attributed.
- Run the required coordinator-managed interface, runtime, app, editor, product, shutdown-race, and WPR gates.
- Complete independent atomic review with Critical 0 / Important 0, then create the managed milestone commit and only afterwards return the Runtime03 failure as fixed.

## Open Failure Links

- `docs/plans/zircon_runtime/runtime/03/failure-2026-07-17-mvp-idle-frame-cadence.md`
- `docs/plans/zircon_runtime/runtime/09/failure-2026-07-17-woc-project-runtime-ui-bridge.md`
- `docs/plans/zircon_runtime/runtime/10/failure-2026-07-19-app-entry-host-request-and-wake-boundary.md`
- `docs/plans/zircon_runtime/runtime/10/failure-2026-07-19-dynamic-session-action-lock-domain.md`

All linked failures remain open. This current-source record is status evidence only, not a fixed handoff.
