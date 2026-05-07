# UI Complete Input Events Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first complete M5 shared input-event slice so Zircon UI has serializable input DTOs, unified reply/effect contracts, runtime route/effect application, and editor-native keyboard/IME translation points based on Unreal Slate input semantics.

**Architecture:** `zircon_runtime_interface::ui::dispatch` owns neutral input DTOs, dispatch phases, replies, effects, and diagnostics. `zircon_runtime::ui::surface` applies reply/effects centrally and preserves current pointer/navigation behavior. `zircon_editor` translates native host events into shared DTOs without taking over shared focus/capture/IME semantics.

**Tech Stack:** Rust, Serde DTOs, Cargo workspace, `zircon_runtime_interface`, `zircon_runtime`, `zircon_editor`, PowerShell validation with `--locked`.

---

## File Structure

- Create `zircon_runtime_interface/src/ui/dispatch/input/mod.rs`: structural module exports for the new input-event contract subtree.
- Create `zircon_runtime_interface/src/ui/dispatch/input/event.rs`: `UiInputEvent` and per-family payload declarations.
- Create `zircon_runtime_interface/src/ui/dispatch/input/metadata.rs`: common event metadata, ids, user/device/window/surface ids, modifiers, timestamp, and pointer ids.
- Create `zircon_runtime_interface/src/ui/dispatch/input/reply.rs`: `UiDispatchReply`, `UiDispatchDisposition`, and helper constructors.
- Create `zircon_runtime_interface/src/ui/dispatch/input/effect.rs`: `UiDispatchEffect` and supporting reason/policy enums.
- Create `zircon_runtime_interface/src/ui/dispatch/input/result.rs`: `UiInputDispatchResult`, diagnostics, and host request reporting.
- Modify `zircon_runtime_interface/src/ui/dispatch/mod.rs`: re-export `input` subtree without adding behavior to the root file.
- Modify `zircon_runtime_interface/src/tests/contracts.rs`: add DTO construction and serialization coverage for the new input contracts.
- Modify `zircon_runtime/src/ui/surface/surface.rs`: add narrow helpers that apply generic dispatch effects to existing focus/capture/navigation state and dispatch keyboard/text/IME events through focused or direct routes.
- Modify `zircon_runtime/src/ui/tests/event_routing.rs`: add focused shared runtime tests for double click metadata, precise scroll preservation, reply/effect capture/high precision, keyboard focus bubbling, text commit, and IME owner lifecycle.
- Create `zircon_runtime/src/ui/tests/runtime_input_ownership.rs`: split focused owner-safety regressions out of the oversized event routing test module.
- Modify `zircon_editor/src/ui/slint_host/host_contract/window.rs`: add pure native-to-shared input translation helpers and keep the actual event loop forwarding path narrow.
- Modify `zircon_editor/src/tests/host/slint_window/native_host_contract.rs`: add translation-focused tests for native keyboard, IME preedit/commit, and pixel wheel x/y preservation where the current host test harness can exercise pure helpers without requiring a real OS event loop.
- Update `docs/superpowers/specs/2026-05-06-ui-complete-input-events-design.md`: record implementation files and actual validation evidence.
- Update `docs/ui-and-layout/shared-ui-core-foundation.md` or create `docs/ui-and-layout/shared-ui-input-events.md`: document the shared input-event contract, routing semantics, and validation commands.

## Milestone 1: Shared Input Contract Foundation

Goal: `zircon_runtime_interface::ui::dispatch` can represent the full M5 event family and Slate-like reply/effect contracts without runtime/editor behavior changes.

In-scope behaviors:
- `UiInputEvent` variants for pointer, keyboard, text, IME, navigation, analog, drag/drop, popup, and tooltip timer.
- Common metadata for timestamp/sequence, surface id, window id, user id, device id, pointer id, modifiers, and synthetic flag.
- `UiDispatchReply` with unhandled, handled, blocked, and passthrough dispositions plus ordered effects.
- `UiDispatchEffect` entries for focus, pointer capture/release, pointer lock, high precision pointer, drag/drop, navigation, popup, tooltip, input method requests, dirty/redraw, and component event emission.
- `UiInputDispatchResult` diagnostics that can report handled phase, route target, effects applied/rejected, host requests, and component events.

Dependencies:
- Existing pointer DTOs in `zircon_runtime_interface/src/ui/dispatch/pointer/**`.
- Existing surface/focus/node ids in `zircon_runtime_interface/src/ui/surface/**` and `event_ui`.
- Existing component event envelopes in `zircon_runtime_interface/src/ui/component/**`.

Implementation slices:
- [x] Add `dispatch/input` folder-backed subtree with one declaration family per file.
- [x] Wire only structural re-exports in `zircon_runtime_interface/src/ui/dispatch/mod.rs`.
- [x] Add contract tests in `zircon_runtime_interface/src/tests/contracts.rs` that construct every event family and every effect family.
- [x] Add serde round-trip tests for at least pointer, keyboard, IME, drag/drop, popup, tooltip, and input-method request payloads.
- [x] Add concise comments to `UiDispatchReply` and `UiDispatchEffect` explaining they are transient commands, not durable widget state.

Testing stage:
- Run `cargo test -p zircon_runtime_interface --lib ui_input --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture`.
- Run `cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture`.
- Run `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never`.
- Debug/correction loop: if DTO tests fail, fix the interface declaration or serde default first; do not patch runtime/editor behavior to make a contract test pass.

Lightweight checks:
- `rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/dispatch/input/*.rs zircon_runtime_interface/src/ui/dispatch/mod.rs zircon_runtime_interface/src/tests/contracts.rs`

Exit evidence:
- New DTOs compile in `zircon_runtime_interface`.
- Contract tests prove every M5 event/effect family is representable and serializable.

Milestone 1 evidence recorded:
- `rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/dispatch/input/*.rs zircon_runtime_interface/src/ui/dispatch/mod.rs zircon_runtime_interface/src/tests/contracts.rs` passed after the post-review contract-shape fixes.
- `cargo test -p zircon_runtime_interface --lib ui_input --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture` passed: 2 passed, 0 failed.
- `cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture` passed: 35 passed, 0 failed.
- `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never` passed.

## Milestone 2: Runtime Surface Reply/Effect Application

Goal: `UiSurface` can apply the generic reply/effect contract to existing focus/capture/navigation state and route keyboard/text/IME events through shared state.

In-scope behaviors:
- Apply `SetFocus`, `ClearFocus`, `CapturePointer`, `ReleasePointerCapture`, `UseHighPrecisionPointer`, `RequestInputMethod` including `UiInputMethodRequestKind::Disable`, `RequestNavigation`, and `EmitComponentEvent` effects.
- Preserve existing pointer click, hover, capture, and scroll behavior.
- Preserve precise scroll delta metadata on the new shared event result even while legacy pointer dispatch keeps scalar scroll fallback.
- Add keyboard focused-route dispatch diagnostics.
- Add text commit and IME direct-owner lifecycle diagnostics without implementing M6 caret/selection rendering.

Dependencies:
- Milestone 1 DTOs.
- Existing `UiSurface::focus_node`, `clear_focus`, `capture_pointer`, `release_pointer_capture`, `dispatch_pointer_event`, and `dispatch_navigation_event`.

Implementation slices:
- [x] Add `UiSurface::apply_dispatch_reply(...)` or equivalent narrow helper that consumes `UiDispatchReply` and returns `UiInputDispatchResult`.
- [x] Add `UiSurface::dispatch_input_event(...)` as the shared input entry point, delegating pointer/navigation to existing behavior and handling keyboard/text/IME via focus or direct owner routes.
- [x] Add runtime tests for focus effect, capture release effect, high precision release-on-capture-release, keyboard focused route diagnostics, text commit owner routing, and IME owner cleanup.
- [x] Keep `surface.rs` from becoming a broad implementation sink by extracting new helper modules if the new logic starts mixing event families.

Testing stage:
- Run `cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture`.
- Run `cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture`.
- Run `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never`.
- Debug/correction loop: when a high-level event test fails, inspect the shared effect application and focus/capture state first before changing individual test expectations.

Lightweight checks:
- `rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/tests/event_routing.rs zircon_runtime/src/ui/tests/shared_core.rs`

Exit evidence:
- Existing pointer behavior tests still pass.
- New runtime tests prove shared effects mutate surface input state centrally.

Milestone 2 evidence recorded:
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/mod.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/input/mod.rs" "zircon_runtime/src/ui/surface/input/state.rs" "zircon_runtime/src/ui/surface/input/effect.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/tests/event_routing.rs" "zircon_runtime/src/ui/tests/shared_core.rs"` passed.
- `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 16 passed, 0 failed, 860 filtered out. Existing runtime warning noise remained.
- `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 36 passed, 0 failed, 840 filtered out. Existing interface/runtime warning noise remained.
- `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never` passed with existing runtime warning noise.
- Review correction added optional `UiPreciseScrollDelta` x/y/unit metadata to shared pointer input events, preserved it on the returned `UiInputDispatchResult.event`, and added runtime coverage for input-method disable encoded through `RequestInputMethod { kind: Disable }`.
- Post-review-correction validation from 2026-05-06 10:51: `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/ui/dispatch/input/mod.rs" "zircon_runtime_interface/src/ui/dispatch/mod.rs" "zircon_runtime_interface/src/tests/contracts.rs" "zircon_runtime/src/ui/tests/event_routing.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/surface/input/effect.rs" "zircon_runtime/src/ui/surface/input/state.rs" "zircon_runtime/src/ui/surface/input/mod.rs" "zircon_runtime/src/ui/surface/surface.rs"` passed.
- Post-review-correction validation from 2026-05-06 10:51: `cargo test -p zircon_runtime_interface --lib ui_input --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 2 passed, 0 failed.
- Post-review-correction validation from 2026-05-06 10:51: `cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 45 passed, 0 failed, 3 filtered out.
- Post-review-correction validation from 2026-05-06 10:51: `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never` passed.
- Post-review-correction validation from 2026-05-06 10:51: `cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 16 passed, 0 failed, 860 filtered out.
- Post-review-correction validation from 2026-05-06 10:51: `cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 36 passed, 0 failed, 841 filtered out.
- Post-review-correction validation from 2026-05-06 10:51: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never` passed with existing runtime warning noise.
- Quality-review fixes tightened stale-state handling: `ReleasePointerCapture` rejects mismatched owner/pointer ids, `RequestInputMethod { kind: Disable }` rejects owner mismatches without emitting a host request, and owner-routed text/IME-like dispatch rejects missing, disabled, or non-render-visible owners with `owner route rejected` diagnostics.
- Quality-fix validation from 2026-05-06 13:11: `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/ui/dispatch/input/mod.rs" "zircon_runtime_interface/src/ui/dispatch/mod.rs" "zircon_runtime_interface/src/tests/contracts.rs" "zircon_runtime/src/ui/tests/event_routing.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/surface/input/effect.rs" "zircon_runtime/src/ui/surface/input/state.rs" "zircon_runtime/src/ui/surface/input/mod.rs" "zircon_runtime/src/ui/surface/surface.rs"` passed.
- Quality-fix validation from 2026-05-06 13:11: focused scroll/capture filters passed before the full runtime gate: `release_capture_effect_clears_only_current_captor` passed, `scroll_fallback` passed 3 tests, and `template_tree_builder_preserves_parent_owned_slot_contracts` passed after a transient template compile blocker in an overlapping session-owned area.
- Quality-fix validation from 2026-05-06 13:11: `cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 18 passed, 0 failed, 869 filtered out.
- Quality-fix validation from 2026-05-06 13:11: `cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 36 passed, 0 failed, 851 filtered out.
- Quality-fix validation from 2026-05-06 13:11: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never` passed with existing runtime warning noise.
- Post-review fix from 2026-05-06 13:35 added `zircon_runtime/src/ui/surface/input/validation.rs` so effect application and owner-routed dispatch share the same valid-owner predicate: owner exists, is enabled, is render-visible, and all ancestors are render-visible. Text dispatch now clears stale IME ownership and falls back to focus; IME dispatch clears stale owners; input-method enable validates owners before host requests; high-precision disable validates the current owner. The later 20:18 owner-safety fix tightened input-method reset/update to require the current IME owner.
- Post-review-fix validation from 2026-05-06 13:35: `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/input/mod.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/surface/input/effect.rs" "zircon_runtime/src/ui/tests/event_routing.rs"` passed.
- Post-review-fix validation from 2026-05-06 13:35: focused filters passed before the full runtime gate: `shared_input_dispatch` passed 2 tests and `dispatch_reply_applies` passed 2 tests.
- Post-review-fix validation from 2026-05-06 13:35: `cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 19 passed, 0 failed, 869 filtered out.
- Post-review-fix validation from 2026-05-06 13:35: `cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 36 passed, 0 failed, 852 filtered out.
- Post-review-fix validation from 2026-05-06 13:35: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never` passed with existing runtime warning noise.
- Owner-cutover fix from 2026-05-06 14:05 made owner-clearing effects carry explicit targets in the shared DTO: `ClearFocus { target, reason }`, `ReleasePointerCapture { target, pointer_id, reason }`, and `UnlockPointer { target, policy }`. Runtime application now rejects stale clear-focus, capture-release, pointer-unlock, high-precision, pointer-lock, pointer-capture, and input-method owner effects before mutating shared state or emitting host requests.
- Owner-cutover validation from 2026-05-06 14:05: `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/dispatch/input/effect.rs" "zircon_runtime_interface/src/tests/contracts.rs" "zircon_runtime/src/ui/surface/input/mod.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/surface/input/effect.rs" "zircon_runtime/src/ui/tests/event_routing.rs"` passed after formatting.
- Owner-cutover validation from 2026-05-06 14:05: focused filters passed: `cargo test -p zircon_runtime_interface --lib ui_input ...` passed 2 tests; `dispatch_reply_applies` passed 2 tests; `focus_effects_clear_only_their_current_input_owner` passed 1 test.
- Owner-cutover validation from 2026-05-06 14:05: `cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 49 passed, 0 failed, 3 filtered out.
- Owner-cutover validation from 2026-05-06 14:05: `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never` passed.
- Owner-cutover validation from 2026-05-06 14:05: `cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 20 passed, 0 failed, 869 filtered out.
- Owner-cutover validation from 2026-05-06 14:05: `cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 36 passed, 0 failed, 853 filtered out.
- Owner-cutover validation from 2026-05-06 14:05: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never` passed with existing runtime warning noise.
- `surface.rs` line count after validation extraction: 806 lines; `event_routing.rs` line count after owner-cutover regressions: 1051 lines. Next runtime input scenario work should split `event_routing.rs` before adding more tests.
- Owner-safety final fixes from 2026-05-06 20:41 moved follow-up regressions into `zircon_runtime/src/ui/tests/runtime_input_ownership.rs` instead of growing `event_routing.rs`. Runtime focus cleanup now clears a previous IME owner only after a successful focus move, focus/capture reject hidden-ancestor owners through the shared valid-owner predicate, navigation uses the same path, direct `clear_focus()` clears only the focused IME owner, direct capture clears stale pointer ids before high precision can enable, input-method reset/update require the current IME owner, high-precision enable requires live capture for the same owner, capture transfer clears the previous captor's high precision, and capture release clears high precision only for the released owner.
- Owner-safety final validation from 2026-05-06 20:41: `rustfmt --edition 2021 --config skip_children=true --check "zircon_runtime/src/ui/surface/input/mod.rs" "zircon_runtime/src/ui/surface/input/state.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/surface/input/effect.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_runtime/src/ui/tests/event_routing.rs" "zircon_runtime/src/ui/tests/runtime_input_ownership.rs"` passed.
- Owner-safety final validation from 2026-05-06 20:41: `cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 7 passed, 0 failed, 897 filtered out.
- Owner-safety final validation from 2026-05-06 20:41: `cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 20 passed, 0 failed, 884 filtered out.
- Owner-safety final validation from 2026-05-06 20:41: `cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never -- --nocapture` passed: 38 passed, 0 failed, 866 filtered out.
- Owner-safety final validation from 2026-05-06 20:41: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-complete-input-events" --message-format short --color never` passed with existing runtime warning noise.

## Milestone 3: Editor Native Translation Slice

Goal: editor native host code can translate keyboard, IME, and precise pointer metadata into shared `UiInputEvent` DTOs without owning routing semantics.

In-scope behaviors:
- Pure helper translation for keyboard text/modifiers/repeat into `UiInputEvent::Keyboard`.
- Pure helper translation for IME preedit/commit into `UiInputEvent::Ime` or `UiInputEvent::Text` as appropriate.
- Preserve pixel wheel x/y delta in shared pointer event metadata.
- Keep actual native host event loop changes minimal and avoid editor painter/text/input regression session areas unless required by compile errors.

Dependencies:
- Milestone 1 DTOs.
- Current `zircon_editor/src/ui/slint_host/host_contract/window.rs` native event loop.

Implementation slices:
- [ ] Add pure helper functions in `window.rs` or a focused child module under `host_contract/window/` if the file becomes mixed-responsibility.
- [ ] Add tests in `native_host_contract.rs` that exercise helper translation without requiring a real OS event loop.
- [ ] Do not route text into editor command callbacks in this milestone; shared runtime dispatch owns the semantics and editor callback integration remains a later host cutover slice.

Testing stage:
- Run `cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture`.
- Run `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never`.
- Debug/correction loop: if failures touch existing native text/input regression code, read `.codex/sessions/20260505-1106-editor-native-text-input-regression.md` before changing those paths.

Lightweight checks:
- `rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/host_contract/window.rs zircon_editor/src/tests/host/slint_window/native_host_contract.rs`

Exit evidence:
- Editor tests prove keyboard/IME/wheel native data can become shared input DTOs.
- Editor host does not gain independent focus/capture/IME semantics.

## Milestone 4: Docs And Acceptance

Goal: code-facing docs reflect the new shared input contract and actual validation evidence.

In-scope behaviors:
- Update the approved design spec with implementation status and validation results.
- Create or update `docs/ui-and-layout/shared-ui-input-events.md` with machine-readable header and module details.
- Update active session note with exact files, tests, blockers, and remaining risk.

Dependencies:
- Milestones 1-3 implementation slices.

Implementation slices:
- [ ] Update docs header `related_code`, `implementation_files`, `plan_sources`, and `tests` to include all touched files.
- [ ] Record Unreal and Slint reference evidence used by the implementation.
- [ ] Record deferred M6 text rendering and M7 debug-tool boundaries.

Testing stage:
- Run the same scoped validation commands from Milestones 1-3 after docs changes if source files changed during documentation updates.
- If docs-only changes follow a completed source validation run, record that no additional Cargo validation was required for docs-only edits.

Lightweight checks:
- Verify frontmatter starts with `related_code` and uses repository-relative paths.

Exit evidence:
- Docs can be used to map shared input-event code back to this plan and spec.
- Remaining risks are explicit rather than hidden in implementation notes.

## Plan Self-Review

- Spec coverage: every approved event family has a contract milestone; runtime application covers focus/capture/navigation/text/IME and preserves pointer behavior; editor host covers native keyboard/IME/wheel translation; docs and validation are explicit.
- Scope boundary: full M6 text shaping/caret/selection and M7 debug tooling are excluded, matching the spec.
- Placeholder scan: the plan intentionally contains no `TBD`, `TODO`, or unspecified test commands.
- Type consistency: all new API names use the `UiInputEvent`, `UiDispatchReply`, `UiDispatchEffect`, and `UiInputDispatchResult` names from the approved spec.
