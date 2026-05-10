---
related_code:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime_interface/src/tests/accessibility_contracts.rs
  - zircon_runtime_interface/src/tests/mod.rs
implementation_files:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime_interface/src/tests/accessibility_contracts.rs
  - zircon_runtime_interface/src/tests/mod.rs
plan_sources:
  - docs/superpowers/plans/2026-05-09-accesskit-bridge.md
  - user: 2026-05-09 implement AccessKit bridge Milestone 0 neutral interface contracts
tests:
  - zircon_runtime_interface/src/tests/accessibility_contracts.rs
  - pass: coordination scan with LookbackHours 4
  - pass: cargo metadata --locked --format-version 1
  - pass: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/accessibility.rs zircon_runtime_interface/src/ui/dispatch/input/event.rs zircon_runtime_interface/src/runtime_api.rs zircon_runtime_interface/src/tests/mod.rs zircon_runtime_interface/src/tests/accessibility_contracts.rs
  - pass: cargo test -p zircon_runtime_interface --lib accessibility_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-accesskit-bridge --message-format short --color never (rerun after initial compile timeout; 7 passed, 0 failed, 83 filtered)
  - pass: cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-accesskit-bridge --message-format short --color never
  - pass: git diff --check on Milestone 0 interface/doc files with LF/CRLF warnings only
doc_type: module-detail
---

# Accessibility Interface Contracts

`zircon_runtime_interface::ui::accessibility` owns the AccessKit-free accessibility DTO layer. The interface crate records serializable snapshots, actions, action results, diagnostics, and ABI event payloads only; it does not extract runtime UI trees, validate runtime surfaces, convert snapshots to AccessKit, or host platform adapters.

## Milestone 0 Scope

Milestone 0 of `docs/superpowers/plans/2026-05-09-accesskit-bridge.md` lands the neutral contract baseline needed by later runtime and app milestones. Runtime accessibility behavior and app host behavior remain untouched in this milestone, so no runtime extraction, app-host adapter behavior, screen-reader integration, or AccessKit type exposure is claimed here. The only runtime crate touch is the compile-compatibility initializer in `zircon_runtime/src/dynamic_api/exports.rs`, which sets the appended optional `capture_accessibility_tree` API slot to `None`.

The current dependency decision is to keep `zircon_runtime_interface` free of `accesskit` and `accesskit_winit`. Repository metadata currently locks `winit = 0.31.0-beta.2`; Bevy's reference `accesskit = "0.24"` and `accesskit_winit = "0.32"` remain the starting AccessKit pair, but `accesskit_winit 0.32.0` declares `winit ^0.30.5`. Later runtime/app milestones must re-check whether a newer `accesskit_winit` supports the beta `winit` line or whether the app host needs a dependency-version adjustment before adding optional AccessKit crates.

## Action DTOs

`UiAccessibilityActionRequest` describes a neutral accessibility action routed at a `UiNodeId`. It carries the requested `UiAccessibilityAction`, a `UiAccessibilityActionSource`, an optional string value, and an optional numeric value. Defaults intentionally represent an assistive-technology `Activate` request against the default node id so legacy/default deserialization is stable.

`UiAccessibilityActionSource` distinguishes assistive technology, keyboard, pointer, and programmatic action origins. Runtime action dispatch can use this source later to pick focus-visible and widget-event semantics without importing AccessKit concepts into the interface crate.

`UiAccessibilityActionResult` records the target, action, `UiAccessibilityActionStatus`, and optional rejection reason. The status vocabulary is `accepted`, `rejected`, `unsupported`, and `stale_target`, which gives later runtime behavior a stable neutral result contract for stale nodes and unsupported role/action pairs.

## Diagnostics

`UiAccessibilityDiagnosticCode` keeps the original baseline diagnostics and adds invalid-tree diagnostics required by the AccessKit bridge plan: `duplicate_node_id`, `missing_bounds`, `invalid_focus`, `dangling_label`, `dangling_description`, `relation_cycle`, `unsupported_role_action`, and `excluded_focused_node`.

These codes are serialized with snake_case names. They are data-only in this crate; runtime snapshot validation in later milestones decides when each diagnostic is emitted.

## Input And ABI Payloads

`UiInputEvent::Accessibility(UiAccessibilityInputEvent)` lets shared UI dispatch streams carry neutral accessibility action requests with the same `UiInputEventMetadata` used by pointer, keyboard, text, IME, navigation, analog, drag/drop, popup, and tooltip-timer inputs.

The runtime ABI adds `ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1` and `ZrRuntimeEventV1::accessibility_action(...)`. The constructor stores serialized `UiAccessibilityActionRequest` bytes in `payload`; ownership and lifetime remain the caller's responsibility, matching the existing `ZrByteSlice` ABI contract.

`ZrRuntimeAccessibilityTreeRequestV1` captures the ABI version, viewport handle, requested viewport size, and a `generation_hint` for later app-host snapshot caching. `ZrRuntimeCaptureAccessibilityTreeFnV1` returns a serialized neutral `UiAccessibilityTreeSnapshot` through `ZrOwnedByteBuffer`, and `ZrRuntimeApiV1::empty(...)` leaves the appended `capture_accessibility_tree` function pointer as `None` by default.

## Tests

`zircon_runtime_interface/src/tests/accessibility_contracts.rs` covers action DTO roundtrips and defaults, snake_case diagnostic serialization, accessibility input-event payload roundtrip, runtime ABI request construction, capture function pointer shape, default optional API table field state, and accessibility action event payload bytes.

The API table test locks `capture_accessibility_tree` as the immediate appended field after `capture_frame` using `core::mem::offset_of!`, so future field reordering or insertion between those slots is caught by the focused interface test.

Milestone 0 validation evidence:

- `./.codex/skills/zircon-project-skills/cross-session-coordination/scripts/Get-RecentCoordinationContext.ps1 -RepoRoot E:\Git\ZirconEngine -LookbackHours 4`: PASS.
- `cargo metadata --locked --format-version 1`: PASS.
- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/runtime_api.rs" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs"`: PASS.
- `cargo test -p zircon_runtime_interface --lib accessibility_contracts --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: first run timed out during initial compile; rerun PASS with 7 passed, 0 failed, 83 filtered.
- `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/runtime_api.rs" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "docs/zircon_runtime_interface/ui/accessibility.md"`: PASS with LF/CRLF warnings only.
