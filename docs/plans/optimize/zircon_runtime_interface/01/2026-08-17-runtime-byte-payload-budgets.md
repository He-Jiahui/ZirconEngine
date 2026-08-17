Plan: docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
Milestone: M0 steps 1-2 - checked borrowed bytes and producer-side payload budgets
Status: completed
Files: [
  "docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md",
  "docs/plans/optimize/zircon_runtime_interface/01/2026-08-17-runtime-byte-payload-budgets.md",
  "docs/zircon_runtime/dynamic_api/session.md",
  "docs/zircon_runtime_interface/runtime_api.md",
  "zircon_app/src/entry/runtime_library/runtime_session.rs",
  "zircon_app/src/entry/runtime_library/runtime_session/request_encoding.rs",
  "zircon_app/src/entry/runtime_library/runtime_session/tests.rs",
  "zircon_editor/src/core/gateway/session/protocol.rs",
  "zircon_editor/src/tests/gateway/session/fixture.rs",
  "zircon_plugins/sound/runtime/src/dynamic_event_abi/status.rs",
  "zircon_plugins/sound/runtime/src/module.rs",
  "zircon_plugins/sound/runtime/src/service_types/manager_state.rs",
  "zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/callbacks/capture.rs",
  "zircon_runtime/src/dynamic_api/bounded_json.rs",
  "zircon_runtime/src/dynamic_api/frame.rs",
  "zircon_runtime/src/dynamic_api/mod.rs",
  "zircon_runtime/src/dynamic_api/session/construction.rs",
  "zircon_runtime/src/dynamic_api/session/event_mirror.rs",
  "zircon_runtime/src/dynamic_api/session/events.rs",
  "zircon_runtime/src/dynamic_api/session/ffi.rs",
  "zircon_runtime/src/dynamic_api/session/host_requests.rs",
  "zircon_runtime/src/dynamic_api/session/operation.rs",
  "zircon_runtime/src/dynamic_api/session/project.rs",
  "zircon_runtime/src/dynamic_api/session/registry/mod.rs",
  "zircon_runtime/src/dynamic_api/session/registry/session_store.rs",
  "zircon_runtime/src/dynamic_api/session/registry/tests.rs",
  "zircon_runtime/src/dynamic_api/session/runtime_ui.rs",
  "zircon_runtime/src/dynamic_api/session/state.rs",
  "zircon_runtime/src/dynamic_api/session/status.rs",
  "zircon_runtime/src/dynamic_api/session/world_sync.rs",
  "zircon_runtime/src/dynamic_api/tests/profile_control.rs",
  "zircon_runtime/src/dynamic_api/tests/session_entry_points.rs",
  "zircon_runtime/src/dynamic_api/tests/support.rs",
  "zircon_runtime/src/operation/maintenance.rs",
  "zircon_runtime/src/operation/service.rs",
  "zircon_runtime/src/operation/service/admission.rs",
  "zircon_runtime/src/operation/service/completion.rs",
  "zircon_runtime/src/operation/task.rs",
  "zircon_runtime/src/operation/tests.rs",
  "zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/abi_decode/read.rs",
  "zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/bridge_scope/tests.rs",
  "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs",
  "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs",
  "zircon_runtime/src/scene/event_mirror/error.rs",
  "zircon_runtime/src/scene/event_mirror/subscription.rs",
  "zircon_runtime/src/scene/inspection/snapshot.rs",
  "zircon_runtime/src/scene/inspection/tests.rs",
  "zircon_runtime/src/scene/mod.rs",
  "zircon_runtime/src/ui/accessibility/budget.rs",
  "zircon_runtime/src/ui/accessibility/diagnostics.rs",
  "zircon_runtime/src/ui/accessibility/extract.rs",
  "zircon_runtime/src/ui/accessibility/extract/resolution.rs",
  "zircon_runtime/src/ui/accessibility/mod.rs",
  "zircon_runtime/src/ui/accessibility/name.rs",
  "zircon_runtime/src/ui/surface/surface.rs",
  "zircon_runtime/tests/runtime_owned_result_v7.rs",
  "zircon_runtime_host/src/foreign_output/budget.rs",
  "zircon_runtime_host/src/foreign_output/decode.rs",
  "zircon_runtime_host/src/foreign_output/error.rs",
  "zircon_runtime_host/src/foreign_output/policy.rs",
  "zircon_runtime_host/src/foreign_output/tests.rs",
  "zircon_runtime_interface/src/buffer.rs",
  "zircon_runtime_interface/src/lib.rs",
  "zircon_runtime_interface/src/status.rs",
  "zircon_runtime_interface/src/tests/accessibility_contracts.rs",
  "zircon_runtime_interface/src/tests/contracts.rs",
  "zircon_runtime_interface/src/tests/window_runtime_event_adapter_contracts.rs",
  "zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs"
]

# Runtime checked-byte and payload-budget convergence

## Scope Delivered

- Replaced unchecked foreign byte access with `ZrByteSlice::checked_slice`, which rejects
  null-plus-nonzero carriers, lengths above `isize::MAX`, and operation-specific byte limits before
  constructing a Rust slice. Status diagnostics, App, Editor, native plugin, and sound callback
  paths now consume the same checked contract.
- Centralized V1 byte, item, depth, time, frame-dimension, and frame-RGBA limits in
  `zircon_runtime_interface`. Producer and consumer policies derive from that source instead of
  carrying divergent local constants.
- Added bounded JSON validation and request encoding before FFI work. Frame capture uses checked
  arithmetic and hard dimension/byte gates; operation, plugin, host-request, profile, project,
  world, input, and status paths return structured limit failures without retaining rejected data.
- Replaced plugin-event `to_vec` materialization with a streaming bounded writer. The writer checks
  bytes and nesting before retention, samples its deadline every 4 KiB, and always checks the
  deadline again at completion. This removes per-byte clock reads while preserving the hard time
  budget.
- Made world-query accounting include the exact empty, not-modified, and rows envelopes before row
  retention. Accessibility extraction now uses one shared budget across every surface and accounts
  names, descriptions, paths, children, diagnostics, actions, and final envelope replacements before
  mutation. Resolution work moved to `extract/resolution.rs`; all touched source files remain below
  1,000 lines.
- Kept V7 owned-result publication transactional across host/plugin/operation/world outputs so a
  rejected consumer payload cannot silently consume producer state. Host preflight diagnostics now
  preserve the shared maximum nesting depth.
- Closed the support-layer compile regression exposed by the sound ABI gate: service factories pass
  `CoreWeak`, while `DefaultSoundManager` retained its public `CoreHandle` constructors and gained a
  crate-private weak-core constructor. The manager therefore avoids an Arc cycle without breaking
  its public construction API.

This record completes only M0 steps 1-2 for the reviewed runtime-interface slice. It does not claim
the full M0 gate: destroy deadlines/cancellation and build-set identity remain in the parent plan;
the previously completed V7 allocation registry remains recorded separately.

## Status And Evidence

| Milestone slice | Status | Evidence |
| --- | --- | --- |
| M0 step 1: checked borrowed-byte conversion | completed | Interface carrier tests, App/Editor/sound consumers, and native plugin decode paths reject invalid shape/size before slice construction. |
| M0 step 2: producer-side frame and JSON budgets | completed | Runtime producer gates cover frame arithmetic plus operation/plugin/host/profile/project/world/accessibility JSON; host policies derive from the interface limits. |
| Lossless output admission | completed | Producer queue/state is committed only after bounded encoding/allocation succeeds; regression tests cover later valid delivery after rejection. |
| Performance gate | completed | Final-source Release P99 is 26.6-31.9 us against the shared 10 ms ceiling. |

## Fresh Testing Evidence

All managed Cargo commands below ran on Windows through the coordinator-managed validation lane with
`--locked` and `Dry run: off`. Direct repetitions reused the exact binaries produced by those runs.

- `zircon_runtime_interface`: 18/18 focused tests passed: checked byte slices 2, accessibility ABI 7,
  window runtime-event adapter 7, and plugin-event carrier contracts 2.
- `zircon_runtime_host --release --lib`: 10/10 passed, including bounded graph preflight, nesting and
  total decode time, release/fuse behavior, concurrent rejection, shared policy derivation, and the
  performance acceptance test.
- `zircon_runtime --lib`: the final source compiled successfully. Eleven owned boundary tests passed:
  plugin delivery continuity 1, bounded world query 5, accessibility build budget 2, streaming event
  writer 1, bounded IME context 1, and allocation-free bounded JSON limit reporting 1.
- `zircon_app --lib`: 15/15 runtime-session tests passed, including request encoding, frame limits,
  plugin-page limits, release on malformed/truncated outputs, and diagnostic preservation.
- `zircon_editor --test runtime_foreign_output_policy`: 2/2 passed, covering protocol-fuse dispatch
  blocking and exactly-once V7 frame allocation release.
- `zircon_plugin_sound_runtime --lib dynamic_event_abi_`: 2/2 passed after the support-first
  `CoreWeak` service-construction repair.
- Plan-owned focused total: 58/58 passed. `git diff --check` and targeted `rustfmt --check` passed.
- A broader accessibility discovery run executed 108 tests and reported 103 passed / 5 failed. The
  five failures are outside this Session's write scope: three Runtime 15 source-plan/test-count
  anchors and two concurrently dirty UI behavior paths. The owned accessibility budget and structure
  gates above pass on the final source; the unrelated failures were not masked or absorbed.

## Performance

Harness: `foreign_output_decode_performance_acceptance`, Release profile, 64 warmups plus 2,000
measured decodes per round, 926 encoded bytes and 256 items. Acceptance threshold: P99 <= 10 ms.

| Round | P50 | P95 | P99 | Throughput |
| --- | ---: | ---: | ---: | ---: |
| 1 | 12.4 us | 23.1 us | 27.7 us | 71,531 payloads/s |
| 2 | 12.5 us | 26.4 us | 31.9 us | 62,656 payloads/s |
| 3 | 12.4 us | 21.1 us | 26.6 us | 75,110 payloads/s |
| 4 | 12.5 us | 26.5 us | 29.5 us | 65,726 payloads/s |
| 5 | 12.3 us | 25.8 us | 28.6 us | 72,269 payloads/s |

Aggregate P50 min/median/max: 12.3/12.4/12.5 us. Aggregate P95: 21.1/25.8/26.5 us.
Aggregate P99: 26.6/28.6/31.9 us. Throughput min/median/max:
62,656/71,531/75,110 payloads/s.

The worst P99 is 313.48x below the latency ceiling, consumes 0.319% of the 10 ms budget, and leaves
99.681% headroom. These measurements establish acceptance on the final source; they do not claim a
cross-machine speedup over a historical baseline.

## Review

Independent final reviews reported Critical 0 / Important 0 for both the runtime producer-budget
slice and the host foreign-output path. The review confirmed write-before-retention accounting,
deadline sampling plus completion checking, lossless queue continuity after rejected payloads,
accessibility shared-budget propagation, and consistent preflight/decode diagnostics.
