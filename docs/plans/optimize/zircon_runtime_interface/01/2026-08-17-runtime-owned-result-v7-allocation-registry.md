Plan: docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
Milestone: M0.3 / M2 owned-result slice
Status: completed
Files: [
  ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_abi_inventory.py",
  ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_failure_inventory.py",
  ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_host_request_inventory.py",
  ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_validation_inventory.py",
  "docs/engine-architecture/runtime-architecture-review-m0.md",
  "docs/engine-architecture/runtime-interface-cdylib-loader.md",
  "docs/engine-architecture/runtime-interface-convergence.md",
  "docs/plans/optimize/zircon_runtime_interface/01/2026-08-17-runtime-owned-result-v7-allocation-registry.md",
  "docs/plans/zircon_runtime/frameworks/development-conventions.md",
  "docs/zircon_editor/core/gateway.md",
  "docs/zircon_runtime/dynamic_api/session.md",
  "docs/zircon_runtime/ui/accessibility.md",
  "docs/zircon_runtime_interface/profiling.md",
  "docs/zircon_runtime_interface/runtime_api.md",
  "zircon_app/src/entry/runtime_library/loaded_runtime.rs",
  "zircon_app/src/entry/runtime_library/runtime_session.rs",
  "zircon_app/src/entry/runtime_library/runtime_session/foreign_output/performance_tests.rs",
  "zircon_app/src/entry/runtime_library/runtime_session/foreign_output/tests.rs",
  "zircon_app/src/entry/runtime_library/runtime_session/operation.rs",
  "zircon_app/src/entry/runtime_library/runtime_session/owned_buffer.rs",
  "zircon_app/src/entry/runtime_library/runtime_session/tests.rs",
  "zircon_app/src/entry/runtime_library/tests.rs",
  "zircon_editor/src/core/gateway/session/frame.rs",
  "zircon_editor/src/core/gateway/session/gateway.rs",
  "zircon_editor/src/core/gateway/session/operations.rs",
  "zircon_editor/src/core/gateway/session/output.rs",
  "zircon_editor/src/core/gateway/session/plugin_events.rs",
  "zircon_editor/src/core/gateway/session/profile.rs",
  "zircon_editor/src/core/gateway/session/protocol.rs",
  "zircon_editor/src/core/gateway/session/tests.rs",
  "zircon_editor/src/core/gateway/session/world_sync.rs",
  "zircon_editor/src/core/play/tests.rs",
  "zircon_editor/src/tests/gateway/session/fixture.rs",
  "zircon_editor/src/tests/gateway/session/frame_demand.rs",
  "zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs",
  "zircon_editor/src/tests/runtime_event_consumer_bounded_pump/real_runtime_abi.rs",
  "zircon_editor/tests/runtime_foreign_output_policy.rs",
  "zircon_runtime/src/dynamic_api/exports.rs",
  "zircon_runtime/src/dynamic_api/frame.rs",
  "zircon_runtime/src/dynamic_api/mod.rs",
  "zircon_runtime/src/dynamic_api/session.rs",
  "zircon_runtime/src/dynamic_api/session/event_mirror.rs",
  "zircon_runtime/src/dynamic_api/session/ffi.rs",
  "zircon_runtime/src/dynamic_api/session/operation.rs",
  "zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs",
  "zircon_runtime/src/dynamic_api/session/registry/mod.rs",
  "zircon_runtime/src/dynamic_api/session/registry/session_slot.rs",
  "zircon_runtime/src/dynamic_api/session/registry/session_store.rs",
  "zircon_runtime/src/dynamic_api/session/registry/tests.rs",
  "zircon_runtime/src/dynamic_api/session/state.rs",
  "zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs",
  "zircon_runtime/src/dynamic_api/tests/accessibility.rs",
  "zircon_runtime/src/dynamic_api/tests/api_table.rs",
  "zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs",
  "zircon_runtime/src/dynamic_api/tests/host_requests.rs",
  "zircon_runtime/src/dynamic_api/tests/linked_plugins.rs",
  "zircon_runtime/src/dynamic_api/tests/operation.rs",
  "zircon_runtime/src/dynamic_api/tests/profile_control.rs",
  "zircon_runtime/src/dynamic_api/tests/session_entry_points.rs",
  "zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs",
  "zircon_runtime/src/dynamic_api/tests/support.rs",
  "zircon_runtime/src/dynamic_api/tests/viewport.rs",
  "zircon_runtime/src/script/vm/module/module_descriptor.rs",
  "zircon_runtime/src/script/vm/tests/module_surface.rs",
  "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs",
  "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs",
  "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/runtime_diagnostics.rs",
  "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/abi.rs",
  "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/behavior.rs",
  "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/host_requests.rs",
  "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/source_inventory.rs",
  "zircon_runtime/tests/runtime_owned_result_v7.rs",
  "zircon_runtime_host/src/foreign_output/mod.rs",
  "zircon_runtime_host/src/foreign_output/owned_buffer.rs",
  "zircon_runtime_host/src/foreign_output/state.rs",
  "zircon_runtime_host/src/foreign_output/tests.rs",
  "zircon_runtime_interface/src/buffer.rs",
  "zircon_runtime_interface/src/handles.rs",
  "zircon_runtime_interface/src/lib.rs",
  "zircon_runtime_interface/src/profiling.rs",
  "zircon_runtime_interface/src/runtime_api/api_table.rs",
  "zircon_runtime_interface/src/runtime_api/operation.rs",
  "zircon_runtime_interface/src/runtime_api/plugin_event_mirror.rs",
  "zircon_runtime_interface/src/runtime_api/requests.rs",
  "zircon_runtime_interface/src/tests/abi_safety_contracts.rs",
  "zircon_runtime_interface/src/tests/accessibility_contracts.rs",
  "zircon_runtime_interface/src/tests/contracts.rs",
  "zircon_runtime_interface/src/tests/mod.rs",
  "zircon_runtime_interface/src/tests/runtime_operation.rs",
  "zircon_runtime_interface/src/tests/runtime_owned_result.rs",
  "zircon_runtime_interface/src/version.rs"
]

# Runtime-Owned Result V7 Allocation Registry

## Scope Delivered

- Completes the concrete `M0.3` allocation-registry requirement while delivering it through the `M2` typed-carrier cutover milestone.
- Hard-cut the dynamic runtime ABI from V6 to V7. No compatibility loader, alias, legacy export, or dual ownership branch remains.
- Replaced pointer/capacity/free-token results with immutable `ZrOwnedResultV2 { data, len, allocation }` and opaque `ZrRuntimeAllocationId` handles.
- Added the mandatory `release_allocation` V7 table entry and migrated app, editor, host policy, frame capture, profiling, operations, plugin events, accessibility, host requests, and world invalidations to one explicit release contract.
- Added a runtime-owned allocation registry keyed by allocation ID and bound to session and output kind. It enforces exactly-once release, rejects forged, repeated, and cross-session IDs without changing owner census, exposes per-session census/high-water metrics, and blocks session destruction until outstanding allocations are released.
- Added a dedicated release lifecycle guard so allocation release remains legal after teardown enters retry-pending state while still participating in the destroy quiescence barrier.
- Kept allocation registration outside the main runtime-session lock while holding one continuous session action lease across state read/consume, encoding, allocation registration, and ABI out-parameter publication. Session destroy therefore cannot complete or permit DLL unload in the former producer-to-registration gap.
- Converged the editor frame gateway on `ZrRuntimeFrameV2`; every output ABI check now declares its expected version explicitly, and frame storage releases its allocation before dropping the last runtime-session/DLL owner on both explicit release and implicit Drop paths.
- Repaired the scripting module hierarchy so `ScriptModule.Manager.VmPluginManager` owns the singleton plugin facade through the legal Driver -> Manager -> Plugin relationship.
- The two shared runtime test fixtures `foundation_render.rs` and `vampire_hud.rs` were already integrated and validated by the prerequisite Tooling convergence commit `2065dc4cbf46f9166fe1a0af47572d1fc2800e97`; this candidate therefore owns the remaining 91 paths and does not duplicate those files.
- This record closes the `M0.3` allocation-registry gate and the owned-result portion of `M2`; it does not claim completion of the other `M0` or `M2` gates.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M0.3 / M2 owned-result slice | Runtime-owned result V7 typed carrier、session-bound allocation registry、exactly-once release、destroy gate、App/Editor/host hard cut | `completed` | 2026-08-17 | Owned gates: interface 69/69, host 8/8, runtime integration 5/5, runtime finalizer race 1/1, app 46/46, editor 2/2; final managed batch 11/11; static ABI audit 60 sources / 21 behavior anchors / V7 25 fields / 23 operations / 0 legacy hits; independent review Critical 0 / Important 0. Five final-source Release measurements: p99 700-800 ns, throughput 2,581,311.31-3,340,013.36/s, meeting <=1,000,000 ns and >=10,000/s. |

## Fresh Testing Evidence

- `zircon_runtime_interface`: the production library built successfully and all 69 plan-owned tests passed: 5 owned-result, 9 ABI-safety, 7 accessibility, and 48 contract tests.
- `zircon_runtime_host --lib`: all 8 library tests passed.
- `zircon_runtime`: the production package build passed; `runtime_owned_result_v7` passed all 5 integration cases in development mode, including cross-session rejection and destroy-after-release retry.
- The allocation-finalizer concurrency regression was introduced red: the managed lib-test build failed only because `with_session_result_finalized` and `register_runtime_allocation_in_action` did not yet exist. After implementation, the same non-dry-run filter passed and proved destroy remains blocked while finalization is paused, then reports outstanding allocation state until release and retry.
- `zircon_editor --test runtime_foreign_output_policy`: both tests passed, covering the shared foreign-output fuse and explicit/Drop exactly-once V7 frame release. The test release stub now records the session that produced each frame, so the fixture enforces the same session-bound allocation contract as production.
- `zircon_app`: all 46 plan-owned focused tests passed: 35 runtime-session tests, 10 runtime-API tests, and the V7 hard-cut contract.
- Static runtime ABI audit passed with 60 source anchors, 21 behavior anchors, a 25-field V7 table, 23 operation wrappers, no direct bypasses, and no legacy V6 ownership hits.
- Rust and Python ABI source inventories matched exactly at 60 entries; `git diff --check` and targeted `rustfmt` checks passed.
- The final coordinator-managed non-dry-run batch passed all 11 validation commands: interface owned-result, ABI-safety, accessibility, and contract suites; runtime-host library; runtime owned-result integration; allocation-finalizer race; app runtime-session, runtime-API, and V7 hard-cut suites; and editor foreign-output policy.

The unfiltered `zircon_runtime_interface --lib` test process exited with code 101 before emitting a libtest result, although the library build and its 69 plan-owned focused tests all passed. The full `zircon_app` package build remains blocked by six unrelated shader-viewer errors, including the missing `SceneViewportSurface` symbol and a private `render` boundary; its unfiltered runtime-library test process likewise exited with code 101 before a libtest result. The root `zircon_app::entry::runtime_library` group also contains one environment-only failure: `runtime_library_default_path_uses_the_physical_product_directory_identity` requires creating a Windows junction, but the current process lacks the `mklink /J` privilege. These constraints are outside the V7 owned-output acceptance surface and are recorded rather than masked.

The prerequisite Tooling convergence commit `2065dc4cbf46f9166fe1a0af47572d1fc2800e97` removed the 326 shared `zircon_runtime --lib` compile errors encountered during early V7 validation. Its non-dry-run acceptance compiled the full lib-test target, passed all 10 selected filters, and verified the resulting binary at 126 discovered tests, 125 executed/passed, and 1 ignored.

## Performance

Build and threshold command: `cargo test -p zircon_runtime --release --test runtime_owned_result_v7 runtime_v7_release_performance_acceptance` through the coordinator-managed validation lane, `Dry run: off`. The managed lane passed 5/5 rounds. The exact values below are a separate five-run measurement from the resulting final-source Release test binary, with no additional Cargo invocation.

Acceptance thresholds: p99 <= 1,000,000 ns and throughput >= 10,000 releases/s.

| Round | p50 | p95 | p99 | Throughput |
| --- | ---: | ---: | ---: | ---: |
| 1 | 300 ns | 500 ns | 800 ns | 2,581,311.31/s |
| 2 | 200 ns | 500 ns | 800 ns | 2,717,022.14/s |
| 3 | 200 ns | 300 ns | 700 ns | 3,340,013.36/s |
| 4 | 200 ns | 300 ns | 700 ns | 3,264,240.25/s |
| 5 | 200 ns | 400 ns | 800 ns | 2,894,356.01/s |

Aggregate p50 min/median/max: 200/200/300 ns. Aggregate p95: 300/400/500 ns. Aggregate p99: 700/800/800 ns. Throughput min/median/max: 2,581,311.31 / 2,894,356.01 / 3,340,013.36 releases/s.

The worst observed p99 is 1,250x below the latency ceiling, and the lowest observed throughput is 258.13x above the required floor.

## Review

- Reviewer sessions: `review-runtime-interface01-host-output-planck-r3-01a00797-20260817` and final re-review `/root/app01_foreign_output_review`.
- Critical findings: 0.
- Important findings: 0.
- The initial review found one Critical lifecycle race: destroy could enter between producer action completion and allocation registration. The implementation now releases the session mutex but retains the same action guard until registration and FFI output publication finish; the final re-review confirmed all eight production output paths close that window with no remaining Critical or Important finding.
- The reviewers confirmed that session-bound release validates ownership before registry/census mutation, release actions participate in the destroy barrier across retry-pending teardown, and forged, cross-session, concurrent double-release, explicit-release, and Drop paths are covered.
- The reviewer confirmed the Release harness and acceptance thresholds. The final-source exact measurements were captured after that review and are attached above with their aggregate calculations.
