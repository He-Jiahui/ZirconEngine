---
related_code:
  - zircon_editor/src/core/gateway/capabilities.rs
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/core/gateway/in_process.rs
  - zircon_editor/src/core/gateway/session.rs
  - zircon_editor/src/core/gateway/detached.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/scene/level_system.rs
implementation_files:
  - zircon_editor/src/core/gateway/capabilities.rs
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/core/gateway/in_process.rs
  - zircon_editor/src/core/gateway/session.rs
  - zircon_editor/src/core/gateway/detached.rs
  - zircon_editor/src/core/context/editor_context.rs
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
  - zircon_editor/src/tests/gateway/session.rs
  - zircon_editor/src/tests/gateway/handle.rs
  - zircon_editor/src/tests/runtime_event_consumer.rs
  - zircon_app/src/entry/runtime_library/tests.rs
doc_type: module-detail
---

# Editor Runtime Gateway

`EditorRuntimeGateway` is the editor-owned boundary for runtime access. UI and editor-domain consumers depend on the gateway contract instead of retaining `CoreHandle`, `LevelSystem`, or `World` as private alternate runtime entry points.

## Implementations

`InProcessGateway` owns the runtime `CoreHandle` and the active `LevelSystem`. Its borrowed `with_world` and `with_world_mut` methods call the level system directly, so GUI/editor code can inspect or mutate the same world instance without serializing it or cloning scene state.

`SessionGateway` is the serialized transport over a normalized `ZrRuntimeApiV2` function table and one valid runtime session handle. It calls the ABI entries for frame ticks, events, frame capture, profiling, plugin-event mirroring, and asynchronous operations. The optional `profile_control` entry returns `Ok(None)` when the provider does not expose it; missing required entries still return `CapabilityMissing`. Borrowed world access is permanently rejected with `RequiresSerializedAccess`. The gateway also retains an opaque `Arc<dyn Send + Sync>` provider owner; its unsafe constructor requires that owner to keep every copied function pointer loaded until the gateway is dropped. In `zircon_app`, that owner is the `Arc<RuntimeSession>`, so session destruction and dynamic-library unload cannot occur while editor calls remain possible.

Every `ZrOwnedByteBuffer` returned by the session table is wrapped immediately. The wrapper validates `len <= capacity`, rejects null or callback-less owned storage, parses only validated initialized bytes, and invokes the provider's `free` callback exactly once. Empty plugin-event batches, invalid JSON, malformed buffers, and runtime error statuses all use the same cleanup path; decoding errors cannot leak provider-owned storage. Frame capture is also a hard ownership boundary: `SessionGateway` copies validated provider RGBA bytes into `EditorRuntimeFrame` and frees the ABI buffer before returning. No editor-visible frame retains a provider function pointer, so replacing a gateway transport cannot leave a frame whose destructor jumps into an unloaded runtime library.

`RuntimeCapabilities` materializes the five-state session profile, the canonical sorted/deduplicated `EditorCoreProfile` capability set, and a canonical plugin activation summary. Registration diagnostics distinguish active, disabled, and rejected plugins. Plugin summaries are sorted by ID, version, and activation state; only exact duplicates are removed, so contradictory registrations remain visible in deterministic order. Gateway calls return this object by value as an immutable snapshot. Returning a borrowed reference would be unsound for `EditorRuntimeGatewayHandle`, because transport replacement can retire the previous gateway while existing callers still hold the stable handle.

`DetachedEditorRuntimeGateway` represents a transport without a live in-process runtime. Borrowed world access returns `GatewayError::RequiresSerializedAccess`. This rejection is the permanent contract for detached and future session-backed transports; consumers that must work in both modes need a serialized query or command API.

`EditorRuntimeGatewayHandle` is the stable, replaceable gateway reference. Every call snapshots the current `Arc<dyn EditorRuntimeGateway>` and forwards to that transport. Replacing a transport therefore changes later calls without invalidating handles already held by editor services.

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

The first managed gateway-matrix job `a4ff529bd2d64b4984fd09783e31af68`, run `5636833c80374faf974920f821287952`, was blocked before tests by a Performance01-owned `SharedString` conversion E0282 in retained text input. That cross-plan failure was returned canonically and closed by managed commit `43a1957e929739e229fcd34ab0ef1c36f0f156c3`. The current-source retry used job `13392907003549dbac31e080da7ab7aa`, run `ff0b13f008754335abe011470ad59f75`, and passed `cargo test -p zircon_editor --lib gateway:: --locked --jobs 1 -- --test-threads=1` with 24 passed, 0 failed, 3334 filtered, and exit 0. M2.2 remains partially verified only because the `zircon_app` runtime-library upstream gate is still queued under exact reservation `1b02c0fbbda6495c9385c057654310a6`.

The former Runtime15 screen-space UI text font-id report blocker has been fixed and returned with the managed `text_font` gate at 47/47 and independent review 0/0/0. That historical blocker does not replace the current M2.2 validation requirements.
