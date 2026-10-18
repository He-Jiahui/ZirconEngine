# Editor01 HighlightSet V4 Hard-Cut Manifest

Scope: forward repair for `failure-2026-07-31-highlight-set-gateway-contract.md`.
This is an isolated implementation manifest, not a formal plan record or a failure return.

## Boundary Decision

- Retire `ZrRuntimeApiV3`, `ZrRuntimeGetApiFnV3`, `ZR_RUNTIME_GET_API_SYMBOL_V3`, and
  `zircon_runtime_get_api_v3` together. The replacement is V4 only; there is no loader fallback,
  table alias, or tail-field compatibility path.
- Introduce the runtime-owned, serialized DTOs in
  `zircon_runtime_interface/src/runtime_api/editor_overlay.rs`:
  `ZrRuntimeHighlightSetV1` and `ZrRuntimeEditorOverlayRequestV1`.
- `ZrRuntimeHighlightSetV1` contains an ordered `Vec<world_sync::EntityId>`, `outline`, and an
  optional RGBA tint. It contains no editor-authoring vocabulary.
- `ZrRuntimeEditorOverlayRequestV1` carries ABI version, viewport handle, frame generation, and
  exactly one HighlightSet. The V4 entry point receives it as JSON through `ZrByteSlice`.
- Runtime stores one value per viewport. A request with a generation below the stored generation
  succeeds without replacing it; equal or newer generations replace it. No queue is allocated.
- The V4 capability identifier is `runtime.editor_overlay.submit`. Missing dynamic API entries,
  detached gateways, and unsupported gateways return `GatewayError::CapabilityMissing`.

## Exact Production Scope

- `zircon_runtime_interface/src/runtime_api.rs`
- `zircon_runtime_interface/src/runtime_api/api_table.rs`
- `zircon_runtime_interface/src/runtime_api/editor_overlay.rs` (new)
- `zircon_runtime_interface/src/lib.rs`
- `zircon_runtime_interface/src/version.rs`
- `zircon_runtime/src/dynamic_api/exports.rs`
- `zircon_runtime/src/dynamic_api/mod.rs`
- `zircon_runtime/src/dynamic_api/session.rs`
- `zircon_runtime/src/dynamic_api/session/ffi.rs`
- `zircon_runtime/src/dynamic_api/session/state.rs`
- `zircon_runtime/src/dynamic_api/session/editor_overlay.rs` (new)
- `zircon_runtime/src/dynamic_api/session/construction.rs`
- `zircon_runtime/src/plugin/core_profiles.rs`
- `zircon_editor/src/core/gateway/contract.rs`
- `zircon_editor/src/core/gateway/handle.rs`
- `zircon_editor/src/core/gateway/in_process.rs`
- `zircon_editor/src/core/gateway/session.rs`
- `zircon_app/src/entry/runtime_library/loaded_runtime.rs`

All remaining V3 consumers are compile-driven mechanical V4 replacements and must be included in
the same atomic hard-cut commit. No Editor05 `SelectionHighlightExtract` adapter is included in
this base contract: Editor05 consumes this contract in its own follow-on migration and must remove
the legacy extract rather than dual-write it.

## Required Tests

- Interface DTO JSON round trip preserves ordered entities, visual attributes, viewport, and
  generation; malformed/unknown fields are rejected.
- ABI inventory asserts V4 table shape, V4 export symbol, and absence of every V3 table/symbol
  across interface, runtime, editor, app loader, and test fixtures.
- Dynamic runtime FFI rejects invalid ABI, invalid viewport, and malformed JSON with a typed ABI
  status; two viewports remain isolated; stale generation does not replace newer state.
- Session gateway serializes the V4 request and reports a missing function pointer as
  `GatewayError::CapabilityMissing`.
- In-process and detached gateway tests prove the same ordered input and typed unsupported error.
- The Editor05 consumer migration test remains a separate successor: it proves multi-entity
  delivery and stale-entity removal only after the V4 base contract has an accepted return.

## Prohibited Shapes

- No `SelectionHighlightExtract`, `RenderOverlayExtract.selection`, selection aliases, or direct
  `Scene` bridge belongs in this base contract.
- No V3 table, V3 exported symbol, V3 loader branch, `size_bytes` tail extension, global overlay
  singleton, per-frame world clone, or unbounded overlay queue is permitted.
