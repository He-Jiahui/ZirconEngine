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
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
  - user: 2026-05-09 implement AccessKit bridge Milestone 0 neutral interface contracts
tests:
  - zircon_runtime_interface/src/tests/accessibility_contracts.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
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

`UiAccessibilityActionRequest` describes a neutral accessibility action routed at a `UiNodeId`. It carries the requested `UiAccessibilityAction`, a `UiAccessibilityActionSource`, an optional string value, an optional numeric value, an optional TextInput `text_selection` payload, and an optional scroll `scroll_offset` point. Defaults intentionally represent an assistive-technology `Activate` request against the default node id so legacy/default deserialization is stable.

The current action vocabulary distinguishes whole-field value replacement, selected-text replacement, selection-only movement, expandable state movement, popup/dialog/tooltip dismissal, and scroll movement. `SetValue` carries a complete replacement value for mutable value controls such as sliders and text inputs; `ReplaceSelectedText` carries a text payload that runtime TextInput dispatch applies only to the active neutral `UiA11yTextSelection` range; `SetTextSelection` carries a `UiA11yTextSelection` payload so platform bridges can move caret/selection without changing text; `Expand` and `Collapse` are neutral open-state actions for controls such as disclosures, popups, and future tree/menu items; `Dismiss` is the neutral close/cancel request used by platform actions such as tooltip hide or blur when runtime has an open popup owner or active tooltip state; `ScrollTo` can carry a numeric value, parseable string value, or `scroll_offset` point so platform bridges can preserve native scroll-offset action payloads without importing AccessKit types.

`UiA11yTextSelection` offsets are always UTF-8 byte offsets in the exposed text value. Platform bridges whose native action payloads use character, text-run, UTF-16, or platform-local positions must convert those positions to byte offsets before populating the neutral DTO; they must not copy native character indexes directly into the neutral fields.

`UiAccessibilityActionSource` distinguishes assistive technology, keyboard, pointer, and programmatic action origins. Runtime action dispatch can use this source later to pick focus-visible and widget-event semantics without importing AccessKit concepts into the interface crate.

`UiAccessibilityActionResult` records the target, action, `UiAccessibilityActionStatus`, and optional rejection reason. The status vocabulary is `accepted`, `rejected`, `unsupported`, and `stale_target`, which gives later runtime behavior a stable neutral result contract for stale nodes and unsupported role/action pairs.

## Diagnostics

`UiAccessibilityDiagnosticCode` keeps the original baseline diagnostics and adds invalid-tree diagnostics required by the AccessKit bridge plan: `duplicate_node_id`, `missing_bounds`, `invalid_focus`, `dangling_label`, `dangling_description`, `relation_cycle`, `unsupported_role_action`, and `excluded_focused_node`.

These codes are serialized with snake_case names. They are data-only in this crate; runtime snapshot validation in later milestones decides when each diagnostic is emitted.

## Snapshot State DTOs

`UiA11yState` is the neutral state envelope for runtime accessibility nodes. It carries generic hidden/disabled/focused/selected state, widget state such as expanded/checked/pressed/value, and optional TextInput caret/selection state through `UiA11yTextSelection`. The text-selection DTO records caret, anchor, and focus as byte offsets in the same exposed string used by `state.value`; runtime extraction is responsible for clamping offsets to valid UTF-8 boundaries before a snapshot leaves `zircon_runtime`.

`UiA11yTextSelection` remains AccessKit-free. It gives Zircon runtime, editor hosts, and future platform adapters a stable serialized contract without forcing AccessKit `TextPosition` or `TextRun` concepts into the interface crate.

## Input And ABI Payloads

`UiInputEvent::Accessibility(UiAccessibilityInputEvent)` lets shared UI dispatch streams carry neutral accessibility action requests with the same `UiInputEventMetadata` used by pointer, keyboard, text, IME, navigation, analog, drag/drop, popup, and tooltip-timer inputs.

The runtime ABI adds `ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1` and `ZrRuntimeEventV1::accessibility_action(...)`. The constructor stores serialized `UiAccessibilityActionRequest` bytes in `payload`; ownership and lifetime remain the caller's responsibility, matching the existing `ZrByteSlice` ABI contract.

`ZrRuntimeAccessibilityTreeRequestV1` captures the ABI version, viewport handle, requested viewport size, and a `generation_hint` for later app-host snapshot caching. `ZrRuntimeCaptureAccessibilityTreeFnV1` returns a serialized neutral `UiAccessibilityTreeSnapshot` through `ZrOwnedByteBuffer`, and `ZrRuntimeApiV1::empty(...)` leaves the appended `capture_accessibility_tree` function pointer as `None` by default.

## Tests

`zircon_runtime_interface/src/tests/accessibility_contracts.rs` covers action DTO roundtrips and defaults, snake_case diagnostic serialization, accessibility input-event payload roundtrip, runtime ABI request construction, capture function pointer shape, default optional API table field state, and accessibility action event payload bytes.

`zircon_runtime_interface/src/tests/ui_contract_spine.rs` covers `UiA11yTextSelection` serialization, its collapsed constructor, backward-compatible default deserialization when legacy `UiA11yState` JSON omits `text_selection`, snake_case `replace_selected_text`, `set_text_selection`, `expand`, and `collapse` action spellings, and `UiAccessibilityActionRequest.scroll_offset` roundtrip/default behavior.

The API table test locks `capture_accessibility_tree` as the immediate appended field after `capture_frame` using `core::mem::offset_of!`, so future field reordering or insertion between those slots is caught by the focused interface test.

Text-selection DTO follow-up evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs"`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0050-ui-a11y-text-selection.md"`: PASS with LF/CRLF warnings only.

ReplaceSelectedText action follow-up evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime/ui/surface/input.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for the touched Rust files, docs, plan, and session note. Runtime Cargo validation remains deferred because unrelated `cargo`/`rustc` processes are active in the shared checkout.

SetTextSelection action follow-up evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, and plan. Runtime/interface Cargo validation remains deferred because 6 unrelated `cargo` processes and 3 unrelated `rustc` processes are active in the shared checkout. The AccessKit bridge now treats `SetTextSelection` as a native-character-index payload that must be converted through snapshot text before producing neutral byte offsets.

ScrollOffset action follow-up evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs"`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0145-ui-a11y-scroll-offset.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Runtime/interface Cargo validation remains deferred because 14 unrelated `cargo` processes and 5 unrelated `rustc` processes are active in the shared checkout.

Expand/Collapse action follow-up evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0155-ui-a11y-expand-collapse.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Runtime/interface Cargo validation remains deferred because unrelated `cargo`/`rustc` processes remain active in the shared checkout.

Popup Dismiss action follow-up evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0205-ui-a11y-popup-dismiss.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Runtime Cargo validation remains deferred because unrelated `cargo`/`rustc` processes remain active in the shared checkout.

Tooltip Dismiss action follow-up evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0220-ui-a11y-tooltip-dismiss.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Runtime Cargo validation remains deferred because unrelated `cargo`/`rustc` processes remain active in the shared checkout.

Milestone 0 validation evidence:

- `./.codex/skills/zircon-project-skills/cross-session-coordination/scripts/Get-RecentCoordinationContext.ps1 -RepoRoot E:\Git\ZirconEngine -LookbackHours 4`: PASS.
- `cargo metadata --locked --format-version 1`: PASS.
- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/runtime_api.rs" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs"`: PASS.
- `cargo test -p zircon_runtime_interface --lib accessibility_contracts --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: first run timed out during initial compile; rerun PASS with 7 passed, 0 failed, 83 filtered.
- `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/runtime_api.rs" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "docs/zircon_runtime_interface/ui/accessibility.md"`: PASS with LF/CRLF warnings only.
