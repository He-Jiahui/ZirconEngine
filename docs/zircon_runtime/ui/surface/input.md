---
related_code:
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/component_state.rs
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
  - zircon_runtime_interface/src/ui/dispatch/input/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/tests/contracts.rs
implementation_files:
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/component_state.rs
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
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/tests/contracts.rs
plan_sources:
  - docs/superpowers/specs/2026-05-06-ui-complete-input-events-design.md
  - docs/superpowers/plans/2026-05-06-ui-complete-input-events.md
  - user: 2026-05-06 continue Milestone 2 runtime surface reply/effect application
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
  - user: 2026-05-08 include M3 tab/directional navigation in M2 focus milestone
tests:
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
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
doc_type: module-detail
---

# Runtime UI Surface Input

`zircon_runtime::ui::surface::input` applies the shared M5 input contract to retained runtime UI surfaces. The module is intentionally below `surface/` because `UiSurface` owns focus, capture, navigation, tree dirty state, and retained frame data; hosts only translate native input into shared DTOs and consume host requests returned by dispatch.

## Module Shape

`mod.rs` is structural. It exposes `UiSurfaceInputState` publicly through `zircon_runtime::ui::surface` and keeps `apply_dispatch_reply(...)` plus `dispatch_input_event(...)` crate-private implementation details called by `UiSurface` methods.

`state.rs` stores transient per-surface input ownership that does not belong on individual widgets. The current M2 fields are the captured pointer id, high-precision owner, pointer-lock owner/policy, input-method owner, and latest input-method request. Pointer-capture cleanup is owner-aware: the shared captured pointer id is cleared for capture loss, and high-precision ownership is cleared only when it belongs to the released or replaced captor.

`component_state.rs` stores retained per-node component interaction flags such as hovered, focused, and pressed. It is surface-owned rather than TOML-owned: v2 assets describe initial props/state, then runtime input reduces transient interaction state into the heap-resident `UiSurfaceComponentStateStore`. A real transition in those flags marks the owning node render-dirty only; it does not request layout, hit-test, text, input, or visible-range work. Repeated pointer movement over the same target therefore stays idle after the first hover repaint, while hover enter/leave, focus gain/loss, and press/release remain visible to style/render consumers without reloading prototype assets or mutating authored TOML.

`validation.rs` centralizes input-owner checks used by effect application and owner-routed dispatch. A valid owner must exist, be enabled, be render-visible itself, and have an enabled plus render-visible ancestor chain. This keeps text/IME/high-precision ownership aligned with the same visibility semantics used by arranged hit/render paths and prevents disabled containers from leaving focused descendants active.

`effect.rs` consumes `UiDispatchReply` values. It applies ordered `UiDispatchEffect` entries, records applied and rejected effects with their source `effect_index`, validates target nodes before mutating surface state, marks dirty flags for dirty/redraw requests, and creates typed host requests for pointer lock, pointer unlock, high precision pointer, popup, tooltip, and input method effects. `DirtyRedraw` preserves the supplied structured dirty domains: render-only redraws do not set the legacy `state_flags.dirty` bit, because that bit still widens to hit-test/input/render in `UiSurface::dirty_flags()`. Text-edit visual metadata mutations for caret, selection, and composition ranges also stay render-only; the edited `value` property is the only text-edit mutation that drives text layout and measurement. Input-method enable requests can establish a valid owner; reset/update/disable requests require the current IME owner and are rejected before host requests when stale. Owner-sensitive effects now carry their owner target in the shared DTO: `ClearFocus { target, .. }`, `ReleasePointerCapture { target, pointer_id, .. }`, and `UnlockPointer { target, .. }`. Runtime application rejects stale targets before clearing focus/IME, capture/high-precision, or pointer-lock state. `SetFocus` and navigation clear a previous IME owner only after focus successfully changes, direct capture clears stale pointer ids, and `UseHighPrecisionPointer { enabled: true }` requires live capture for the same owner.

`dispatch.rs` is the shared entry point adapter. Pointer and navigation events keep delegating to the existing runtime dispatchers, preserving their behavior while projecting the result into `UiInputDispatchResult`. Pointer scroll results keep the original shared pointer input event, including optional precise x/y/unit wheel metadata, even while legacy dispatch consumes the scalar fallback. Keyboard input records the focused route. Text input uses a valid IME owner when present, clears stale IME ownership, then falls back to the focused node. IME input clears stale or cancelled IME ownership and reports an `owner route rejected` diagnostic when an invalid stored owner was present. Other owner-routed families validate that the stored owner still exists, is enabled, and has render-visible ancestors before reporting a handled route.

M2 focus routing now records focused input through the runtime-owned `UiFocusState::focused_inputs` log. Keyboard, navigation, text, and IME dispatch paths record the focused node, the bubble route from focused node to root, the handler or owner that accepted the focused-input route, and whether the focused route was accepted. This mirrors Bevy's focused-input bubbling concept while keeping the route serializable in `zircon_runtime_interface::ui::focus::UiFocusedInput`.

Focus mutation lives in `surface/focus.rs` instead of growing `surface.rs` further. Programmatic focus uses `UiFocusChangeReason::Programmatic`; pointer focus uses `Input` and hides focus-visible by default unless a pointer handler explicitly requests a visible focus ring; keyboard/navigation focus uses `Navigation` and `KeyboardNavigation`. Autofocus resolves from `pending_autofocus` or the first authored `UiFocusContract::autofocus` node and records an `Autofocus` focus-change event. Re-focusing the already focused node updates the visible policy but does not append a duplicate focus-change event.

Focus reconciliation is driven by accepted tree changes only. Runtime property mutation clears focus when an accepted `enabled`, `visible`, `visibility`, or `focusable` mutation makes the focused node invalid, and records `Disabled` or `Hidden` according to the property that invalidated focus. Unchanged and rejected mutations do not emit focus changes. Detaching a subtree to the surface node pool clears focus, capture, press, hover, high-precision, IME, and pointer-lock state for detached owners with `Despawned` focus-change reason.

The focus cleanup path rebuilds the hover-owner list from valid owners instead of retaining with a closure that also borrows the surface for validation. Editable text dispatch captures the focused-input kind before moving the shared input event into `UiInputDispatchResult`, and keyboard focused-input records an accepted focused route whenever a focused owner exists even if the current low-level keyboard reply remains unhandled pending a widget reducer. These are compile-level invariants for the runtime-owned focus/input state and do not change navigation ordering.

M3 navigation behavior consumes the M1 `UiNavigationContract` stored on `UiTreeNode`. `next_navigation_target(...)` applies tab indices, group order, modal group traps, manual directional node/group overrides, blocked edges, and spatial fallback from arranged node centers. Non-modal groups contribute ordering rather than trapping traversal; modal groups filter both tab traversal and manual directional overrides so a dialog cannot escape through an authored neighbor. Existing tree-order helpers remain available for older tests and narrow callers, but surface navigation dispatch uses the contract-aware traversal.

## Surface Integration

`UiSurface` now stores `input: UiSurfaceInputState` with serde defaults so old retained surface snapshots can deserialize without input-state fields. Public methods `UiSurface::apply_dispatch_reply(...)` and `UiSurface::dispatch_input_event(...)` keep the shared runtime seam explicit while the implementation remains in the child module.

Pointer capture release paths clear the shared pointer id in addition to clearing `UiFocusState::captured`. When the released owner also owns high precision, `clear_pointer_capture_for(owner)` clears both states; otherwise high precision is left alone so a stale release cannot clear another owner's raw-input state.

Pointer dispatch dirty requests are now first-class retained-surface input invalidation. `UiSurface::apply_pointer_dispatch_dirty(...)` maps each `UiPointerDispatchEffect::RequestDirty(...)` invocation back to the invoking node and preserves the supplied `UiDirtyFlags` without widening render-only requests through the legacy `state_flags.dirty` bit. `RuntimeUiManager` consumes that method after pointer dispatch and only calls `rebuild_dirty(root_size)` when the surface actually has dirty domains. Pointer-derived component state follows the same narrow domain: `apply_pointer_component_state(...)`, `set_node_pressed_dirty(...)`, and `surface/focus.rs` mark render dirty only when the retained hover/focus/press state actually changes. This means runtime v2 fixtures can redraw a pressed/hovered/focused visual through the render domain, or run incremental layout for real geometry changes, without reloading v2 TOML or rebuilding the entire surface tree.

Pointer primary press/release now also reduces the retained pressed state into `UiTreeNode::state_flags.pressed`. The update is intentionally render-only: it changes the accessibility/render-facing state, adds a render dirty flag to the affected node, and does not set the legacy `state_flags.dirty` bit. A second primary press on another target clears the previous pressed node and marks both nodes render-dirty, matching retained widget behavior without forcing layout or hit-grid rebuilds.

Pointer hover, focus, and press also reduce into `UiSurfaceComponentStateStore`. Hover/focus state is retained for component/style consumers but does not yet widen dirty domains by itself; press still marks render dirty through the tree state path because current render and accessibility extraction already read `state_flags.pressed`. Detaching a subtree to the retained node pool clears component states for those nodes before reuse, so recycled controls cannot inherit stale hover/focus/press flags.

## Validation Scope

Focused runtime coverage lives in `zircon_runtime/src/ui/tests/event_routing.rs`, `zircon_runtime/src/ui/tests/runtime_input_ownership.rs`, `zircon_runtime/src/ui/tests/surface_dirty_domains.rs`, and the runtime-host boundary tests in `zircon_runtime/src/tests/ui_boundary/runtime_host.rs`. The ownership tests were split out because `event_routing.rs` is already above the large-file warning threshold. Together they verify focus/capture/high-precision reply effects, owner-targeted clear-focus behavior, rejected focus preserving the current IME owner, navigation/focus changes clearing previous IME owners, direct clear-focus cleanup, focus/capture rejection through hidden ancestors, direct capture clearing stale pointer ids before high precision can enable, stale pointer-capture release rejection, stale pointer-lock unlock rejection, high-precision enable requiring live capture, stale high-precision disable rejection, capture transfer clearing the previous captor's high precision, navigation plus host-owned input effects, input-method reset/update/disable current-owner checks, invalid input-method owner rejection, focused keyboard diagnostics, text owner routing/fallback after stale IME cleanup, stale IME owner-route rejection, hidden-ancestor owner rejection, IME owner cleanup, pointer scroll diagnostics plus precise scroll metadata preservation through the shared input result path, render-only `DirtyRedraw` effects staying out of hit-test/input rebuilds, runtime fixture pointer dirty requests being consumed through the persistent v2 `UiSurface`, primary press/release retained-state reduction staying render-only, component-state store updates for hover/focus/press, and node-pool cleanup for component states.

The M2 scope deliberately does not implement M6 text layout, caret, selection, shaping, or editor-native keyboard/IME translation. Those systems should consume this shared input state and result contract instead of adding host-owned focus, capture, or IME semantics.
