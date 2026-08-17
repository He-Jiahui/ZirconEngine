---
related_code:
  - zircon_editor/src/core/gateway/capabilities.rs
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/error.rs
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/core/gateway/in_process.rs
  - zircon_editor/src/core/gateway/mod.rs
  - zircon_editor/src/core/gateway/session/mod.rs
  - zircon_editor/src/core/gateway/session/gateway.rs
  - zircon_editor/src/core/gateway/session/frame.rs
  - zircon_editor/src/core/gateway/session/output.rs
  - zircon_editor/src/core/gateway/detached.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/editor/tests/runtime_loading.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/scene/level_system.rs
implementation_files:
  - zircon_editor/src/core/gateway/capabilities.rs
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/error.rs
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/core/gateway/in_process.rs
  - zircon_editor/src/core/gateway/mod.rs
  - zircon_editor/src/core/gateway/session/
  - zircon_editor/src/core/gateway/detached.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/entry_runner/editor.rs
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/tests/gateway/in_process.rs
  - zircon_editor/src/tests/gateway/session/
  - zircon_editor/src/tests/gateway/handle.rs
  - zircon_editor/src/tests/runtime_event_consumer.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/entry_runner/editor/tests/runtime_loading.rs
doc_type: module-detail
---

# Editor Runtime Gateway

`EditorRuntimeGateway` is the editor-owned boundary for runtime access. UI and editor-domain consumers depend on the gateway contract instead of retaining `CoreHandle`, `LevelSystem`, or `World` as private alternate runtime entry points.

## Implementations

`InProcessGateway` owns the runtime `CoreHandle` and the active `LevelSystem`. Its borrowed `with_world` and `with_world_mut` methods call the level system directly, so GUI/editor code can inspect or mutate the same world instance without serializing it or cloning scene state.

`SessionGateway` is the serialized transport over the frozen `ZrRuntimeApiV7` function table and one valid runtime session handle. It requires the single `release_allocation` entry together with the session operation group, then calls the ABI entries for frame ticks, events, frame capture, viewport-surface bind/unbind/present, profiling, plugin-event mirroring, and asynchronous operations. Viewport-surface methods preserve the ABI request and runtime viewport handle exactly; consumers on this gateway path do not resolve `RenderFramework` or retain its manager handle. The retained UI migration to that path remains the M2.4 hard-cut. Tick output is caller-owned and initialized to `ZrRuntimeFrameDemandV1::idle()` before entering the runtime. The gateway validates ABI version, raw kind, and kind-specific delay, then maps the result once to the editor-owned `EditorRuntimeFrameDemand::{OnDemand, SleepUntil, Continuous}` facade; malformed demand is a protocol error instead of being silently ignored. `AFTER` demand is clamped to a 60-second host-safe wake, so a provider cannot turn an extreme ABI delay into an unbounded or lost native wait. The retained host window independently applies the same bound to every `SleepUntil` facade value and falls back to an immediate wake if native instant arithmetic cannot represent it. The retained host consumes that facade without importing ABI types: `OnDemand` clears a prior wake, `SleepUntil` owns one native `WaitUntil` deadline, and `Continuous` queues the next frame through the external redraw path. Missing required entries return `CapabilityMissing`. Borrowed world access is permanently rejected with `RequiresSerializedAccess`. The gateway also retains an opaque `Arc<dyn Send + Sync>` provider owner; its unsafe constructor requires that owner to keep every copied function pointer loaded until the gateway is dropped. In `zircon_app`, that owner is the `Arc<RuntimeSession>`, so session destruction and dynamic-library unload cannot occur while editor calls remain possible.

The embedding host injects its shared viewport-surface lifecycle marker through `SessionGateway::with_viewport_surface_lifecycle_state`. A successful gateway bind sets that marker only after the ABI call succeeds; a successful unbind clears it only after the ABI call succeeds. `RuntimeSession` uses the same marker during teardown, so the session remains responsible for a final native-surface unbind even though the editor gateway invoked the normalized V6 entry. Hosts must publish bind, unbind, and present as one table capability; exposing only part of the trio is a contract error rather than a fallback rendering path.

Every `ZrOwnedResultV2` returned by the session table is wrapped immediately. The wrapper validates the null/length/allocation-id shape and `len <= isize::MAX`, parses only validated immutable bytes, and uses one private `RuntimeOwnedOutputReleaser` that binds the gateway session to V7 `release_allocation`. Explicit cleanup, protocol-error cleanup, and Drop therefore send the originating session plus opaque id exactly once; a crossed-session id cannot release another gateway's live storage. The wrapper never accepts allocator capacity, owner tokens, or per-result free callbacks from the provider. Empty plugin-event batches, invalid JSON, malformed results, and runtime error statuses all use the same cleanup path; decoding errors cannot leak provider-owned storage. Frame capture is also a hard ownership boundary: after exact `width * height * 4` shape validation, `SessionGateway` moves the provider result and provider `Arc` into `EditorRuntimeFrame`'s private pixel owner without a full-frame `Vec` copy. A rejected frame releases immediately; a successful frame releases only through explicit `release()` or Drop, after its last borrowed pixel access. Plugin deliveries must retain the requested subscription identity, and operation poll/harvest responses must retain the requested handle. No public editor-facing value exposes a provider function pointer; the private frame owner keeps its provider alive until allocation release, so replacing a gateway transport cannot leave a destructor that jumps into an unloaded runtime library.

`RuntimeCapabilities` materializes the five-state session profile, the canonical sorted/deduplicated `EditorCoreProfile` capability set, and a canonical plugin activation summary. Registration diagnostics distinguish active, disabled, and rejected plugins. Plugin summaries are sorted by ID, version, and activation state; only exact duplicates are removed, so contradictory registrations remain visible in deterministic order. Each gateway returns an `Arc<RuntimeCapabilities>`; `EditorRuntimeGatewayHandle` captures that Arc once while constructing a generation. Stable capability queries therefore clone only the Arc and never rebuild or deep-clone the string collections.

`DetachedEditorRuntimeGateway` represents a transport without a live in-process runtime. Borrowed world access returns `GatewayError::RequiresSerializedAccess`. This rejection is the permanent contract for detached and future session-backed transports; consumers that must work in both modes need a serialized query or command API.

`EditorRuntimeGatewayHandle` is the stable, replaceable gateway reference. Its owner publishes an immutable `{generation, gateway, capabilities}` record through `ArcSwap`. Tick, event, capture, viewport-surface, world access, subscription, and operation calls hold an atomic generation guard for the duration of the call; they do not enter a shared `RwLock` or clone the transport Arc. Replacement is the only serialized control-plane path. It builds the next capability snapshot before publishing, keeps the previous generation alive until all in-flight guards retire, and recovers the writer mutex after a failed snapshot construction without replacing the last valid generation.

The app-side `RuntimeSession` remains the general client/runtime lifecycle owner, but no longer implements the editor gateway trait. Editor startup builds a `SessionGateway` from its validated normalized API table and capability projection. This is a hard cut: there is one editor ABI transport implementation, with no compatibility trait implementation left on `RuntimeSession`.

## Borrowed Callback Contract

Borrowed world callbacks execute while the `LevelSystem` world mutex is held. They are leaf operations: entering `with_world` or `with_world_mut` again from the same thread returns `GatewayError::ReentrantBorrowedWorldAccess` before attempting to lock the world. The RAII guard clears on normal return and panic unwind; independent threads retain their own guard state and continue to serialize through the level mutex.

Keep callbacks bounded to direct inspection or one atomic mutation. Asset IO, project scans, task waits, full hierarchy projection, and frame capture must happen outside the callback. This prevents editor UI and render frames from being stalled by work that does not require exclusive world access.

`InProcessGateway` does not expose its raw `CoreHandle` or `LevelSystem`. Consumers that need a new runtime capability must extend the gateway contract instead of adding an accessor and caching the underlying owner.

## Current Boundary

The M2.1 foundation provides direct borrowed world reads and writes, detached rejection, and stable-handle forwarding. `InProcessGateway::session_handle` is invalid until an in-process runtime session identity is wired. Operation submission, polling, and harvesting on this transport continue to return typed capability-missing results.

The M2.2 source now contains the serialized session transport and runtime capability materialization, and the app entry has been cut over from the former direct `RuntimeSession` trait implementation. This does not complete M2: UI deep imports and world consumers still need to move to the gateway boundary, serialized inspection and editor-overlay input remain to be added with their owning plans, and M2.4 import guards remain pending until those consumers move. Runtime10 has removed `RuntimeDynamicSession.selected_node` and the pointer/scroll selection-sync helper from current source while preserving a construction-only neutral orbit target; its canonical failure return remains open until the upstream `dynamic_api` gate is clean.

## Validation

The initial focused gateway binary proved 3/3 for same-level mutation, stable-handle forwarding, and detached rejection. Review hardening expanded the current source to seven tests covering repeated read/write reentry rejection, panic recovery, and thread-local isolation. The final source review is Critical/Important/Minor 0/0/0. Managed Windows job `37b0965d5e7647bb8952c3adb523145d`, run `6b173cb849884a49b827961fdfcb6667`, executed `cargo test -p zircon_editor --lib gateway::in_process --locked --jobs 1 -- --test-threads=1` against the shared current source and passed 7/7 with exit 0.

M2.2 has separate evidence and does not reuse the M2.1 7/7 result. Its initial current-source review was Critical/Important/Minor 0/2/0: the owned-buffer, optional-profile, frame-copy, provider-lifetime, and RuntimeSession hard-cut paths were accepted, while the editor-normalized API table still carried create/destroy lifecycle pointers and the child-plan record had not yet been advanced. The plan record is now current. A test-first app gate was added for lifecycle stripping; managed job `5134b3bff56c478da5607b04d8bd4495`, run `c6085185488f45b4ae825127483a5238`, was blocked before reaching the assertion by the active Runtime12 input-event-buffer owner, so it is not RED evidence. The source has since removed both lifecycle assignments from `editor_gateway_api_table`; `RuntimeSession` remains the only create/destroy caller, and static source/rustfmt/diff guards pass. Final current-source review is 0/0/0. After the foreign Render01 E0425 was fixed, managed job `18d3e80c10094fe09357ae25892bc2b8`, run `2feb43d1b0944a389cbc7cc4b3a7a0e7`, executed the focused app lifecycle gate and passed 1/1 with 175 filtered and exit 0.

The first managed gateway-matrix job `a4ff529bd2d64b4984fd09783e31af68`, run `5636833c80374faf974920f821287952`, was blocked before tests by a Performance01-owned `SharedString` conversion E0282 in retained text input. That cross-plan failure was returned canonically and closed by managed commit `43a1957e929739e229fcd34ab0ef1c36f0f156c3`. The retry used job `13392907003549dbac31e080da7ab7aa`, run `ff0b13f008754335abe011470ad59f75`, and passed `cargo test -p zircon_editor --lib gateway:: --locked --jobs 1 -- --test-threads=1` with 24 passed, 0 failed, 3334 filtered, and exit 0. A later independent review returned 0/2/4 and caused the buffer, response-identity, normalized-table, and app test-layout hardening described above. Those changes supersede the prior source fingerprint: exact gateway and app runtime-library managed revalidation is pending, and the old app reservation `1b02c0fbbda6495c9385c057654310a6` was released before job creation.

The former Runtime15 screen-space UI text font-id report blocker has been fixed and returned with the managed `text_font` gate at 47/47 and independent review 0/0/0. That historical blocker does not replace the current M2.2 validation requirements.

The generation-bound ArcSwap hard cut has passed file formatting, a 6/6 static gateway contract, a 3/3 dependency guard, a 4/4 test inventory guard, and scoped diff validation. Its former exact-11 reservation `a604598586b74e0e8e6b4d63fe948347` was released before job creation after the source leases expired and the coordinator failed to advance absolute-expired FIFO heads. A fresh source-bound reservation is required after the Coordinator01 failure is returned fixed; these static checks are not a managed Cargo acceptance result.
