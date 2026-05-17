---
related_code:
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/range.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/surface/interaction_state.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/surface_node_pool.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/pointer_click_semantics.rs
  - zircon_runtime/src/ui/tests/widget_range_navigation.rs
  - zircon_runtime/src/ui/tests/widget_radio_behavior.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
  - zircon_runtime_interface/src/ui/dispatch/input/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
implementation_files:
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/range.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/surface/interaction_state.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/surface_node_pool.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/pointer_click_semantics.rs
  - zircon_runtime/src/ui/tests/widget_range_navigation.rs
  - zircon_runtime/src/ui/tests/widget_radio_behavior.rs
  - zircon_runtime/src/ui/tests/widget_menu_behavior.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
plan_sources:
  - docs/superpowers/specs/2026-05-06-ui-complete-input-events-design.md
  - docs/superpowers/plans/2026-05-06-ui-complete-input-events.md
  - user: 2026-05-06 continue Milestone 2 runtime surface reply/effect application
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
  - user: 2026-05-08 include M3 tab/directional navigation in M2 focus milestone
  - user: 2026-05-16 continue Bevy-level UI/Text/Widgets/Focus/A11y completion plan
tests:
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/pointer_click_semantics.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never
  - post-review-correction: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (16 passed, 0 failed, 860 filtered out)
  - quality-fix-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (18 passed, 0 failed, 869 filtered out)
  - quality-fix-validation: cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (36 passed, 0 failed, 851 filtered out)
  - quality-fix-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed with existing warnings)
  - post-review-fix-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (19 passed, 0 failed, 869 filtered out)
  - post-review-fix-validation: cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (36 passed, 0 failed, 852 filtered out)
  - post-review-fix-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed with existing warnings)
  - owner-cutover-validation: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (49 passed, 0 failed, 3 filtered out)
  - owner-cutover-validation: cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed)
  - owner-cutover-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (20 passed, 0 failed, 869 filtered out)
  - owner-cutover-validation: cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (36 passed, 0 failed, 853 filtered out)
  - owner-cutover-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed with existing warnings)
  - owner-safety-final-validation: rustfmt --edition 2021 --config skip_children=true --check zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/surface/input/state.rs zircon_runtime/src/ui/surface/input/validation.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/effect.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/event_routing.rs zircon_runtime/src/ui/tests/runtime_input_ownership.rs (passed)
  - owner-safety-final-validation: cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (7 passed, 0 failed, 897 filtered out)
  - owner-safety-final-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (20 passed, 0 failed, 884 filtered out)
  - owner-safety-final-validation: cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (38 passed, 0 failed, 866 filtered out)
  - owner-safety-final-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed with existing warnings)
  - target: cargo test -p zircon_runtime --lib focus_navigation --locked
  - 2026-05-08 cross-lane compile unblock: cargo test -p zircon_runtime --lib scene::tests::ecs_schedule::render_extract_prepare_flushes_parent_reorder_and_active_changes --locked --message-format short (passed, 1 test, after UI focus/input compile fixes)
  - 2026-05-13 dirty-domain-validation: cargo test -p zircon_runtime --lib surface_dirty_text_edit_visual_metadata_stays_render_only --jobs 1 -- --nocapture --test-threads=1 (passed, 1 test)
  - 2026-05-13 dirty-domain-validation: cargo test -p zircon_runtime --lib surface_dirty_domains --jobs 1 -- --nocapture --test-threads=1 (passed, 10 tests)
  - 2026-05-13 dirty-domain-validation: cargo check -p zircon_runtime --lib --jobs 1 (passed)
  - 2026-05-13 runtime-v2-manager-validation: cargo test -p zircon_runtime --lib runtime_ui_manager_applies_pointer_render_dirty_to_persistent_surface --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 runtime-v2-manager-validation: cargo test -p zircon_runtime --lib runtime_ui_manager_routes_pointer_layout_dirty_through_incremental_surface_rebuild --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 runtime-v2-manager-validation: cargo test -p zircon_runtime --lib runtime_ui_manager_loads_fixture_documents_from_asset_files --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 runtime-v2-manager-validation: cargo test -p zircon_runtime --lib runtime_ui --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 20 tests)
  - 2026-05-13 runtime-v2-manager-validation: cargo check -p zircon_runtime --lib --jobs 1 --target-dir target\codex-ui-v2-guard (passed)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib pointer_dispatch_syncs_pressed_state_as_render_only_dirty --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib pointer_dispatch_clears_previous_pressed_state_when_primary_press_moves_target --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib runtime_ui_manager_applies_pointer_render_dirty_to_persistent_surface --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib runtime_ui_manager_routes_pointer_layout_dirty_through_incremental_surface_rebuild --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib pointer_dispatch_reduces_hover_focus_and_press_into_component_state_store --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib surface_node_pool_reuses_detached_template_node_and_resets_transient_state --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib focus_component_state_changes_mark_render_only_dirty --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib focus_navigation --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 13 tests)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib event_routing --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 26 tests)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib surface_node_pool --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 2 tests)
  - 2026-05-13 component-state-validation: cargo test -p zircon_runtime --lib runtime_ui --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 20 tests)
  - 2026-05-13 component-state-validation: cargo check -p zircon_runtime --lib --jobs 1 --target-dir target\codex-ui-v2-guard (passed)
  - 2026-05-16 widget-behavior-contract-validation: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-widget-behavior --message-format short --color never (passed, 5 tests)
  - 2026-05-16 widget-behavior-runtime-validation: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/widget.rs zircon_runtime_interface/src/tests/ui_contract_spine.rs zircon_runtime/src/ui/surface/surface/default_interactions.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/accessibility/extract.rs zircon_runtime/src/ui/tests/pointer_click_semantics.rs zircon_runtime/src/ui/tests/accessibility.rs (passed)
  - 2026-05-16 widget-behavior-runtime-validation: git diff --check on the same touched Rust/docs files (passed with LF/CRLF warnings only)
  - 2026-05-16 widget-behavior-runtime-validation: cargo test -p zircon_runtime --lib pointer_click_semantics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-widget-behavior --message-format short --color never (inconclusive; did not reach focused tests before package-cache lock waits/dependency compilation interruption in active shared workspace)
  - 2026-05-16 range-default-interactions-module-split: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/surface/default_interactions.rs zircon_runtime/src/ui/surface/surface/default_interactions/range.rs (passed)
  - 2026-05-16 range-default-interactions-module-split: git diff --check on the same touched Rust files (passed with LF/CRLF warnings only)
  - 2026-05-16 radio-widget-contract-validation: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-radio-contract --message-format short --color never (passed, 5 tests)
  - 2026-05-16 radio-widget-runtime-validation: rustfmt --edition 2021 --check on radio/interface/a11y/runtime touched Rust files (passed)
  - 2026-05-16 radio-widget-runtime-validation: git diff --check on radio/interface/a11y/runtime touched Rust files (passed with LF/CRLF warnings only)
  - 2026-05-16 radio-widget-runtime-validation: cargo test -p zircon_runtime --lib widget_radio_behavior --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-radio-runtime --message-format short --color never (inconclusive; timed out after 360 seconds without usable Rust diagnostics while shared Cargo/rustc load remained active)
  - 2026-05-16 menu-widget-runtime-validation: rustfmt --edition 2021 --check on MenuItem runtime/test files (passed)
  - 2026-05-16 menu-widget-runtime-validation: cargo runtime focused test deferred while shared Cargo/rustc load remained active
  - 2026-05-16 text-input-keyboard-validation: rustfmt --edition 2021 --check on TextInput keyboard runtime/test files (passed)
  - 2026-05-16 text-input-keyboard-validation: cargo runtime focused test deferred while shared Cargo/rustc load remained active
  - 2026-05-17 scrollbar-widget-contract-validation: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-scrollbar-contract --message-format short --color never (passed, 6 tests)
  - 2026-05-17 scrollbar-widget-runtime-validation: rustfmt --edition 2021 --check on Scrollbar runtime/interface/a11y/test touched Rust files (passed)
  - 2026-05-17 scrollbar-widget-runtime-validation: cargo test -p zircon_runtime --lib widget_scrollbar_behavior --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-scrollbar-runtime --message-format short --color never (blocked before focused tests by third-party wgpu-hal 29.0.3 DX12/windows type mismatch)
  - 2026-05-16 mutation-status-validation: cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib surface_property_mutation_marks_dirty_only_when_values_change -- --nocapture (passed, 1 test)
  - 2026-05-16 material-runtime-state-validation: cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib editor_material_ -- --nocapture (passed, 2 tests)
doc_type: module-detail
---

# Runtime UI Surface Input

`zircon_runtime::ui::surface::input` applies the shared M5 input contract to retained runtime UI surfaces. The module is intentionally below `surface/` because `UiSurface` owns focus, capture, navigation, tree dirty state, and retained frame data; hosts only translate native input into shared DTOs and consume host requests returned by dispatch.

## Module Shape

`mod.rs` is structural. It exposes `UiSurfaceInputState` publicly through `zircon_runtime::ui::surface` and keeps `apply_dispatch_reply(...)` plus `dispatch_input_event(...)` crate-private implementation details called by `UiSurface` methods.

`state.rs` stores transient per-surface input ownership that does not belong on individual widgets. The current M2 fields are the captured pointer id, high-precision owner, pointer-lock owner/policy, input-method owner, and latest input-method request, including cursor/composition rectangles plus optional surrounding text for native IME context. Pointer-capture cleanup is owner-aware: the shared captured pointer id is cleared for capture loss, and high-precision ownership is cleared only when it belongs to the released or replaced captor.

`component_state.rs` stores retained per-node component interaction flags such as hovered, focused, and pressed. It is surface-owned rather than TOML-owned: v2 assets describe initial props/state, then runtime input reduces transient interaction state into the heap-resident `UiSurfaceComponentStateStore`. A real transition in those flags marks the owning node render-dirty only; it does not request layout, hit-test, text, input, or visible-range work. Repeated pointer movement over the same target therefore stays idle after the first hover repaint, while hover enter/leave, focus gain/loss, and press/release remain visible to style/render consumers without reloading prototype assets or mutating authored TOML.

`surface/interaction_state.rs` is the shared effective-interaction gate for retained nodes. Pointer press/release events, generic click fallback, and default widget reducers all consult the same inputs before mutating or emitting component events: retained `state_flags.enabled`, runtime component-state `disabled`/`enabled` values and disabled flags, retained `disabled` attributes, and `UiWidgetContract.disabled`. This mirrors Bevy's standard widget pattern where `InteractionDisabled` is checked inside button, checkbox, and slider pointer/keyboard reducers before `Activate` or `ValueChange` is triggered.

`validation.rs` centralizes input-owner checks used by effect application and owner-routed dispatch. A valid owner must exist, be enabled, be render-visible itself, and have an enabled plus render-visible ancestor chain. This keeps text/IME/high-precision ownership aligned with the same visibility semantics used by arranged hit/render paths and prevents disabled containers from leaving focused descendants active.

`effect.rs` consumes `UiDispatchReply` values. It applies ordered `UiDispatchEffect` entries, records applied and rejected effects with their source `effect_index`, validates target nodes before mutating surface state, marks dirty flags for dirty/redraw requests, and creates typed host requests for pointer lock, pointer unlock, high precision pointer, popup, tooltip, and input method effects. `DirtyRedraw` preserves the supplied structured dirty domains: render-only redraws do not set the legacy `state_flags.dirty` bit, because that bit still widens to hit-test/input/render in `UiSurface::dirty_flags()`. Text-edit visual metadata mutations for caret, selection, and composition ranges also stay render-only; the edited `value` property is the only text-edit mutation that drives text layout and measurement. Input-method enable requests can establish a valid owner; reset/update/disable requests require the current IME owner and are rejected before host requests when stale. Optional `UiInputMethodSurroundingText` is validated before a host request is emitted so cursor and anchor byte offsets must stay inside the UTF-8 excerpt and on character boundaries. Owner-sensitive effects now carry their owner target in the shared DTO: `ClearFocus { target, .. }`, `ReleasePointerCapture { target, pointer_id, .. }`, and `UnlockPointer { target, .. }`. Runtime application rejects stale targets before clearing focus/IME, capture/high-precision, or pointer-lock state. `SetFocus` and navigation clear a previous IME owner only after focus successfully changes, direct capture clears stale pointer ids, and `UseHighPrecisionPointer { enabled: true }` requires live capture for the same owner.

`dispatch.rs` is the shared entry point adapter. Pointer and navigation events keep delegating to the existing runtime dispatchers, preserving their behavior while projecting the result into `UiInputDispatchResult`. Pointer scroll results keep the original shared pointer input event, including optional precise x/y/unit wheel metadata, even while legacy dispatch consumes the scalar fallback. Keyboard input records the focused route. Text input uses a valid IME owner when present, clears stale IME ownership, then falls back to the focused node. IME input clears stale or cancelled IME ownership and reports an `owner route rejected` diagnostic when an invalid stored owner was present. Other owner-routed families validate that the stored owner still exists, is enabled, and has render-visible ancestors before reporting a handled route.

M2 focus routing now records focused input through the runtime-owned `UiFocusState::focused_inputs` log. Keyboard, navigation, text, and IME dispatch paths record the focused node, the bubble route from focused node to root, the handler or owner that accepted the focused-input route, and whether the focused route was accepted. This mirrors Bevy's focused-input bubbling concept while keeping the route serializable in `zircon_runtime_interface::ui::focus::UiFocusedInput`.

Focus mutation lives in `surface/focus.rs` instead of growing `surface.rs` further. Programmatic focus uses `UiFocusChangeReason::Programmatic`; pointer focus uses `Input` and hides focus-visible by default unless a pointer handler explicitly requests a visible focus ring; keyboard/navigation focus uses `Navigation` and `KeyboardNavigation`. Autofocus resolves from `pending_autofocus` or the first authored `UiFocusContract::autofocus` node and records an `Autofocus` focus-change event. Re-focusing the already focused node updates the visible policy but does not append a duplicate focus-change event.

Focus reconciliation is driven by accepted tree changes only. Runtime property mutation clears focus when an accepted `enabled`, `visible`, `visibility`, or `focusable` mutation makes the focused node invalid, and records `Disabled` or `Hidden` according to the property that invalidated focus. Unchanged and rejected mutations do not emit focus changes. Detaching a subtree to the surface node pool clears focus, capture, press, hover, high-precision, IME, and pointer-lock state for detached owners with `Despawned` focus-change reason.

The focus cleanup path rebuilds the hover-owner list from valid owners instead of retaining with a closure that also borrows the surface for validation. Editable text dispatch captures the focused-input kind before moving the shared input event into `UiInputDispatchResult`, and keyboard focused-input records an accepted focused route whenever a focused owner exists even if the current low-level keyboard reply remains unhandled pending a widget reducer. These are compile-level invariants for the runtime-owned focus/input state and do not change navigation ordering.

M3 navigation behavior consumes the M1 `UiNavigationContract` stored on `UiTreeNode`. `next_navigation_target(...)` applies tab indices, group order, modal group traps, manual directional node/group overrides, blocked edges, and spatial fallback from arranged node centers. Non-modal groups contribute ordering rather than trapping traversal; modal groups filter both tab traversal and manual directional overrides so a dialog cannot escape through an authored neighbor. Existing tree-order helpers remain available for older tests and narrow callers, but surface navigation dispatch uses the contract-aware traversal.

## Widget Behavior Contract

Headless widget behavior is now authored through `UiWidgetContract::behavior` instead of being inferred only from a component name at each runtime call site. `UiWidgetBehavior::Auto` preserves existing templates by mapping known components such as `Button`, `Checkbox`, `Slider`, `TextField`, and `MenuItem` onto behavior families. Explicit values such as `Toggle`, `Range`, or `TextInput` override that fallback, and `Passive` prevents a known component name from gaining default behavior accidentally.

Default pointer and focused-keyboard behavior read the resolved widget behavior once and then apply the behavior family. Buttons and menu items emit the same activated commit through click bindings for pointer release and focused `Enter`/`Space`; pointer double-click keeps the existing `double_activated` binding event while still being owned by the button behavior reducer. Toggle widgets use `checked_property` with `checked` as the legacy fallback. Disclosure and popup widgets use `open_property` with `expanded` or `popup_open` as the current family default. Range widgets use `value_property`, `min_property`, `max_property`, and `step_property`, so custom TOML components can participate in drag, step, and value-change events without adopting the built-in `Slider` or `RangeField` component names.

Typed widget ownership remains in effect even when a widget is disabled: a disabled button, menu item, toggle, disclosure, popup, or range does not fall through to the generic click fallback. Instead, the typed reducer owns the route and no-ops because the shared interaction gate rejects it. That keeps runtime pointer, focused keyboard, and accessibility disabled semantics aligned with Bevy's `InteractionDisabled` checks in `button.rs`, `checkbox.rs`, and `slider.rs`.

Range navigation now follows the same Bevy slider keyboard contract for incremental and endpoint changes. Focused `Left`/`Down` and `Right`/`Up` navigation step through the authored `step_property`; focused `Home` and `End` navigation mutate the authored `value_property` to the current `min_property` or `max_property`. The endpoint path still goes through `UiSurface::mutate_property`, so render-only dirty behavior and runtime component-state value mirroring stay consistent with pointer drag and accessibility increment/decrement.

The Range default reducer now lives in `surface/default_interactions/range.rs`. The parent `default_interactions.rs` remains the common widget-action dispatcher for Button/MenuItem, Toggle, Disclosure, and Popup behavior, while the child module owns Range pointer drag, numeric alias lookup, clamp/step math, and navigation endpoint mutation. This keeps the Bevy slider-aligned behavior family isolated before adding further standard widgets.

Radio behavior follows the Bevy `RadioGroup`/`RadioButton` split in `dev/bevy/crates/bevy_ui_widgets/src/radio.rs`: a selected radio does not emit another change, a disabled radio or disabled nearest group is rejected by the shared interaction gate, and a radio can expose an app-specific value through `UiWidgetContract::value`. Zircon intentionally applies the group selection through `UiSurface::mutate_property` because retained templates and runtime component state are the current data-binding layer. `surface/default_interactions/radio.rs` sets the target radio checked, clears checked sibling radios in the nearest `RadioGroup`, and mutates the group's `value_property` to the selected radio's `widget.value` or stable node identity fallback. Godot's `ButtonGroup` documentation corroborates the mutual-exclusion expectation, while Slint's Material `RadioButton` documentation confirms the author-facing `checked`/`enabled` surface.

Scrollbar behavior follows Bevy `dev/bevy/crates/bevy_ui_widgets/src/scrollbar.rs`: scrollbar track and thumb are headless controls for a scrollable container, not slider-like value widgets. `UiWidgetBehavior::Scrollbar` carries `scroll_target`, optional `scroll_axis`, and `min_thumb_extent`; `ScrollbarThumb` is a passive child marker. The runtime reducer in `surface/default_interactions/scrollbar.rs` resolves a target node by `#node_id`, `control_id`, or node path, then a primary track click pages the target `ScrollableBox` by its viewport extent through `UiRuntimeTreeScrollExt::set_scroll_offset`. Thumb click is intentionally a no-op in this slice because Zircon's shared pointer event DTO does not yet expose Bevy-style drag distance for thumb dragging.

MenuItem activation now owns the first headless menu close behavior. Bevy `dev/bevy/crates/bevy_ui_widgets/src/menu.rs` activates menu items and then sends menu close/focus-root events for the popup stack; the same file also treats Escape as a popup close request. Zircon does not yet have a full neutral menu-stack contract, so `default_interactions.rs` applies the conservative retained equivalent: after a `MenuItem` pointer click or focused Enter/Space activation succeeds, or after Escape is pressed while focus is inside a popup, the nearest enabled `Popup` ancestor with an open `open_property` is mutated to `false`, and matching popup bindings receive `ClosePopup`. This keeps menu closing in the same runtime behavior path as the existing MenuItem `activated` commit without inventing editor-owned popup state.

The keyboard path handles the same default activation keys used by Bevy's headless widgets: `Enter` and `Space` on the focused control. Buttons and menu items emit an activated commit through click bindings; toggles, disclosures, and popups mutate the same properties used by pointer activation. Repeated keyboard events do not trigger this default activation path.

Focused TextInput keyboard editing now uses the same retained editable text state as text and IME input. `surface/input/text_keyboard.rs` maps Bevy-style focused edit keys from local `dev/bevy/crates/bevy_ui_widgets/src/editable_text.rs` into `UiTextEditAction`: Backspace, Delete, Escape composition cancel, ArrowLeft/ArrowRight, Home, and End. `dispatch.rs` applies those actions before default widget activation, so text fields do not accidentally treat editing keys as button activation; text insertion remains owned by `UiInputEvent::Text` or IME commit. Custom controls that author `UiWidgetBehavior::TextInput` and `value_property` are editable even when their component name is not one of the legacy text field names.

This follows the Bevy standard-widget split: `dev/bevy/crates/bevy_ui_widgets/src/lib.rs` defines unstyled widgets that emit `Activate` and `ValueChange<T>` events, while `button.rs`, `checkbox.rs`, and `slider.rs` keep pointer and focused-keyboard behavior separate from styling. Zircon deliberately keeps the behavior contract serialized in `zircon_runtime_interface` because templates and editor-authored widgets need stable runtime behavior without depending on editor-specific styling code.

The focused regression in `zircon_runtime/src/ui/tests/pointer_click_semantics.rs` covers a custom `FavoritePill` component that opts into `Toggle` behavior and mutates a custom `selected` property from both pointer release and Space-key activation. It also covers a button widget contract that emits an activated commit from focused Enter-key activation while the existing pointer click and double-click tests continue to exercise the typed button pointer reducer. Runtime component-state `disabled = true` now blocks button pointer press/click/double-click emission, focused keyboard activation, and toggle pointer/keyboard mutation. `zircon_runtime/src/ui/tests/widget_range_navigation.rs` covers focused Range `Home`/`End` endpoint mutation through authored min/max/value aliases. The interface contract test in `zircon_runtime_interface/src/tests/ui_contract_spine.rs` covers serde round-trip behavior for the new behavior enum and property aliases.

`zircon_runtime/src/ui/tests/widget_radio_behavior.rs` covers Radio pointer selection, already-checked no-op behavior, disabled group rejection, focused keyboard activation, group value mutation, sibling uncheck, and a11y role/action projection for `RadioGroup` and `Radio`.

`zircon_runtime/src/ui/tests/widget_menu_behavior.rs` covers MenuItem pointer and focused keyboard activation closing the nearest Popup ancestor while still delivering the item activation event and popup `ClosePopup` event, plus focused Escape closing the popup without item activation.

`zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs` covers TextInput focused Backspace through an authored `value_property` on a custom component name and ArrowLeft caret movement without emitting a value-change component event when text content is unchanged.

`zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs` covers Scrollbar track click paging a target `ScrollableBox`, thumb click no-op behavior, default a11y exclusion for headless scrollbar widgets, explicit scrollbar a11y opt-in, and accessibility `ScrollTo` mutating the scrollable container offset.

## Surface Integration

`UiSurface` now stores `input: UiSurfaceInputState` with serde defaults so old retained surface snapshots can deserialize without input-state fields. Public methods `UiSurface::apply_dispatch_reply(...)` and `UiSurface::dispatch_input_event(...)` keep the shared runtime seam explicit while the implementation remains in the child module.

Pointer capture release paths clear the shared pointer id in addition to clearing `UiFocusState::captured`. When the released owner also owns high precision, `clear_pointer_capture_for(owner)` clears both states; otherwise high precision is left alone so a stale release cannot clear another owner's raw-input state.

Pointer dispatch dirty requests are now first-class retained-surface input invalidation. `UiSurface::apply_pointer_dispatch_dirty(...)` maps each `UiPointerDispatchEffect::RequestDirty(...)` invocation back to the invoking node and preserves the supplied `UiDirtyFlags` without widening render-only requests through the legacy `state_flags.dirty` bit. `RuntimeUiManager` consumes that method after pointer dispatch and only calls `rebuild_dirty(root_size)` when the surface actually has dirty domains. Pointer-derived component state follows the same narrow domain: `apply_pointer_component_state(...)`, `set_node_pressed_dirty(...)`, and `surface/focus.rs` mark render dirty only when the retained hover/focus/press state actually changes. This means runtime v2 fixtures can redraw a pressed/hovered/focused visual through the render domain, or run incremental layout for real geometry changes, without reloading v2 TOML or rebuilding the entire surface tree.

Pointer primary press/release now also reduces the retained pressed state into `UiTreeNode::state_flags.pressed`. The update is intentionally render-only: it changes the accessibility/render-facing state, adds a render dirty flag to the affected node, and does not set the legacy `state_flags.dirty` bit. A second primary press on another target clears the previous pressed node and marks both nodes render-dirty, matching retained widget behavior without forcing layout or hit-grid rebuilds.

Runtime property mutation synchronizes accepted tree property changes into `UiSurfaceComponentStateStore`, including generic typed values and canonical boolean flags such as disabled, pressed, checked, expanded, popup-open, and selected. Unchanged or rejected mutations do not initialize or rewrite component state and do not mark render dirty. This preserves the public `UiPropertyMutationStatus::Unchanged` contract while still letting real accepted mutations feed runtime pseudo-state styling and accessibility state extraction through the component-state path.

Pointer hover, focus, and press also reduce into `UiSurfaceComponentStateStore`. Hover/focus state is retained for component/style consumers but does not yet widen dirty domains by itself; press still marks render dirty through the tree state path because current render and accessibility extraction already read `state_flags.pressed`. Detaching a subtree to the retained node pool clears component states for those nodes before reuse, so recycled controls cannot inherit stale hover/focus/press flags.

## Validation Scope

Focused runtime coverage lives in `zircon_runtime/src/ui/tests/event_routing.rs`, `zircon_runtime/src/ui/tests/runtime_input_ownership.rs`, `zircon_runtime/src/ui/tests/surface_dirty_domains.rs`, and the runtime-host boundary tests in `zircon_runtime/src/tests/ui_boundary/runtime_host.rs`. The ownership tests were split out because `event_routing.rs` is already above the large-file warning threshold. Together they verify focus/capture/high-precision reply effects, owner-targeted clear-focus behavior, rejected focus preserving the current IME owner, navigation/focus changes clearing previous IME owners, direct clear-focus cleanup, focus/capture rejection through hidden ancestors, direct capture clearing stale pointer ids before high precision can enable, stale pointer-capture release rejection, stale pointer-lock unlock rejection, high-precision enable requiring live capture, stale high-precision disable rejection, capture transfer clearing the previous captor's high precision, navigation plus host-owned input effects, input-method reset/update/disable current-owner checks, invalid input-method owner rejection, focused keyboard diagnostics, text owner routing/fallback after stale IME cleanup, stale IME owner-route rejection, hidden-ancestor owner rejection, IME owner cleanup, pointer scroll diagnostics plus precise scroll metadata preservation through the shared input result path, render-only `DirtyRedraw` effects staying out of hit-test/input rebuilds, runtime fixture pointer dirty requests being consumed through the persistent v2 `UiSurface`, primary press/release retained-state reduction staying render-only, component-state store updates for hover/focus/press, and node-pool cleanup for component states.

The M2 scope deliberately does not implement M6 text layout, caret, selection, shaping, or editor-native keyboard/IME translation. Those systems should consume this shared input state and result contract instead of adding host-owned focus, capture, or IME semantics.
