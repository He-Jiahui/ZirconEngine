# Runtime 10 status diagnostics ABI hard-cut pre-review

## Scope and current source

- Runtime owner: `zircon_runtime/src/dynamic_api/session/status.rs`.
- Interface owners: `zircon_runtime_interface/src/status.rs` and `buffer.rs`.
- Host consumers: Runtime App and Editor gateway status decoders, including create/destroy and plugin-event paths.
- Canonical failure: `docs/plans/zircon_runtime/runtime/10/failure-2026-07-19-dynamic-status-diagnostic-ownership.md`.

This is an architecture pre-review, not an implementation or performance result. No ABI source was changed in this slice.

## Current-source correction

The historical permanent `Box::leak` path is gone. Dynamic errors now write into a bounded 4 KiB thread-local diagnostics buffer and return a borrowed `ZrByteSlice`. This bounds the per-thread storage but does not establish a valid ABI ownership contract:

- `ZrStatus` is `Copy` and carries only a borrowed pointer/length pair.
- A later error on the same thread can overwrite the bytes while an earlier copied status still exists.
- Cross-thread retention and runtime DLL unload have no enforceable lifetime rule.
- Status can fail before a session exists, so the session-specific `release_allocation` API cannot be the sole diagnostics release route.
- App and Editor currently decode diagnostics immediately, but that implementation habit is not an ABI guarantee.

Replacing the buffer with a larger TLS slot, retaining multiple TLS generations, or documenting immediate consumption would preserve the same ownership defect and is rejected.

## Required atomic design

The next implementation must select and freeze one explicit status-diagnostics owner in a new atomically versioned API table. The preferred landing shape is an owned status payload that reuses the interface's existing buffer-level free callback contract, because it can represent errors before session creation:

- the status type is not `Copy` or `Clone`;
- dynamic diagnostics carry data, length/capacity, owner token, and an explicit runtime free callback;
- empty/static-code-only status carries no allocation;
- host wrappers move the status into an RAII decoder and release diagnostics exactly once before a runtime library can unload;
- API negotiation, session creation/destruction, normal calls, panic boundaries, and nested plugin-event status all use the same versioned type;
- the old borrowed status return and TLS diagnostics owner are deleted in the same hard cut, with no V7/V8 per-call fallback.

A caller-provided output buffer remains a viable alternative only if every affected function signature and table slot changes in the same migration. It must report required length deterministically and cannot fall back to TLS or a hidden global last-error registry.

## Implementation order

1. Freeze the new interface type, exact layout, free semantics, empty/error invariants, and API table version.
2. Add interface layout/ownership tests and Runtime constructors, including pre-session and panic paths.
3. Convert every Runtime table slot and nested status carrier atomically.
4. Convert App and Editor decoders to move-only RAII ownership; ensure destroy/unload paths drain diagnostics before unloading the library.
5. Delete the borrowed dynamic status path and thread-local diagnostics buffer.
6. Run error/reload/concurrency matrices before any further optimization.

The current App/Editor/API-table call chain is under broad shared modification, so implementing only steps 1-2 would leave an unusable parallel ABI. This pre-review therefore records `design_review_complete_atomic_cutover_deferred_due_shared_owner_changes` rather than creating a compatibility layer.

## Measurement and acceptance plan

After the atomic cut, measure 1/1k/1M sequential errors, concurrent errors across the Runtime worker/thread matrix, pre-session failures, repeated load/unload, and panic-boundary failures. Record allocation/free counts, outstanding owned bytes, peak working set, wall time, and use-after-free/double-free detection. Acceptance requires:

- allocated diagnostics count equals released diagnostics count;
- outstanding diagnostic bytes return to zero before unload;
- no status points into TLS or unloaded module memory;
- peak resident diagnostics memory is bounded by live host-owned statuses;
- exact diagnostics/code parity for Runtime, App, and Editor consumers.

No CPU, memory, power, or reference-engine comparison is claimed from this source review. Those conclusions require the exact post-cut binary and recorded measurements.
