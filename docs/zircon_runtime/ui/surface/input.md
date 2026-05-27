---
related_code:
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/range.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/interaction_gate.rs
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
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs
  - zircon_runtime/src/ui/tests/widget_text_input_mui.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
  - zircon_runtime/src/ui/tests/popup_tooltip_state.rs
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
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
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/range.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/interaction_gate.rs
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
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs
  - zircon_runtime/src/ui/tests/widget_text_input_mui.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
  - zircon_runtime/src/ui/tests/popup_tooltip_state.rs
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
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
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
tests:
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/pointer_click_semantics.rs
  - zircon_runtime/src/ui/tests/popup_tooltip_state.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - zircon_runtime_interface/src/tests/contracts.rs
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
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/render/resolve.rs zircon_runtime/src/ui/tests/material_layout.rs (2026-05-24 TextInput placeholder measurement support fix: passed; Cargo rerun pending shared build queue)
  - 2026-05-16 text-input-keyboard-validation: cargo runtime focused test deferred while shared Cargo/rustc load remained active
  - 2026-05-20-textinput-constraints-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/text_constraints.rs zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-20-textinput-constraints-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed with existing unused-method warnings)
  - 2026-05-20-textinput-constraints-validation: cargo test -p zircon_runtime --lib widget_text_input_keyboard --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (stopped after exceeding eight minutes in lib-test compilation without reaching focused tests)
  - 2026-05-20-textinput-keyboard-selection-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-20-textinput-keyboard-selection-validation: git diff --check on TextInput keyboard runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-20-textinput-keyboard-selection-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (timed out after 10 minutes under concurrent Cargo/Rust compiler load before diagnostics)
  - 2026-05-20-textinput-grapheme-editing-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/text/grapheme.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/text/edit_state.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-20-textinput-grapheme-editing-validation: git diff --check on TextInput grapheme runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-20-textinput-grapheme-editing-validation: cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-grapheme-light-0520 --message-format short --color never (timed out after 5 minutes under shared Cargo/Rust compiler load before diagnostics)
  - 2026-05-21-textinput-word-navigation-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/text/grapheme.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-word-navigation-validation: git diff --check on TextInput word-navigation runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-word-navigation-validation: cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-word-nav-light-0521 --message-format short --color never (timed out after 5 minutes under shared Cargo/Rust compiler load before diagnostics)
  - 2026-05-21-textinput-selection-replacement-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-selection-replacement-validation: git diff --check on TextInput selection replacement test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-selection-replacement-validation: cargo test -p zircon_runtime --lib widget_text_input_keyboard --no-run --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-selection-replace-light-0521 --message-format short --color never (timed out after 5 minutes under shared Cargo/Rust compiler load before diagnostics)
  - 2026-05-21-textinput-ime-selection-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-ime-selection-validation: git diff --check on TextInput IME selection test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-ime-selection-validation: runtime cargo no-run deferred because unrelated Cargo/Rust compiler jobs were still active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21-textinput-select-all-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-select-all-validation: git diff --check on TextInput select-all runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-select-all-validation: runtime cargo no-run deferred because unrelated Cargo/Rust compiler jobs were still active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21-textinput-word-delete-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-word-delete-validation: git diff --check on TextInput word-delete runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-word-delete-validation: cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-word-delete-light-0521 --message-format short --color never (timed out after 5 minutes under shared Cargo/Rust compiler load before diagnostics)
  - 2026-05-21-textinput-clipboard-host-validation: rustfmt --edition 2021 --check on interface dispatch, runtime input, and TextInput keyboard test files (passed)
  - 2026-05-21-textinput-clipboard-host-validation: cargo test -p zircon_runtime_interface --lib ui_input --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-clipboard-interface-0521 --message-format short --color never (passed, 3 tests)
  - 2026-05-21-textinput-clipboard-host-validation: git diff --check on TextInput clipboard runtime/interface/test files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-clipboard-host-validation: cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-clipboard-runtime-0521 --message-format short --color never (timed out after 5 minutes while unrelated Cargo/Rust compiler jobs were active and produced no Rust diagnostics)
  - 2026-05-21-textinput-multiline-home-end-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/text/grapheme.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-multiline-home-end-validation: git diff --check on TextInput multiline Home/End runtime/test files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-multiline-home-end-validation: runtime cargo check deferred because unrelated Cargo/Rust compiler jobs were still active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21-textinput-readonly-selection-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/text/edit_state.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-readonly-selection-validation: git diff --check on TextInput read-only runtime/test files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-readonly-selection-validation: runtime cargo check deferred because unrelated Cargo/Rust compiler jobs were still active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21-textinput-multiline-up-down-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/text/grapheme.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-multiline-up-down-validation: git diff --check on TextInput multiline Up/Down runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-multiline-up-down-validation: runtime cargo check deferred because unrelated Cargo/Rust compiler jobs were still active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21-textinput-document-arrow-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-document-arrow-validation: git diff --check on TextInput document-arrow runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-document-arrow-validation: runtime cargo check deferred because unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21-textinput-line-boundary-edges-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-line-boundary-edges-validation: git diff --check on TextInput line-boundary edge test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-line-boundary-edges-validation: runtime cargo check deferred because unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21-textinput-enter-newline-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/text_constraints.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-enter-newline-validation: git diff --check on TextInput Enter newline runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-enter-newline-validation: runtime cargo check deferred while unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice before milestone testing
  - 2026-05-21-textinput-escape-collapse-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-escape-collapse-validation: git diff --check on TextInput Escape collapse runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-escape-collapse-validation: runtime cargo check deferred while unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice before milestone testing
  - 2026-05-21-textinput-clipboard-command-keys-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21-textinput-clipboard-command-keys-validation: git diff --check on TextInput clipboard command-key runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21-textinput-clipboard-command-keys-validation: runtime cargo check deferred while unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice before milestone testing
  - 2026-05-21-textinput-hard-line-navigation-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs zircon_runtime/src/ui/tests/mod.rs (passed)
  - 2026-05-21-textinput-hard-line-navigation-validation: tracked git diff --check on TextInput hard-line runtime/docs files and direct trailing-whitespace scan of the new hard-line test file (passed with LF/CRLF warnings only from git)
  - 2026-05-21-textinput-hard-line-navigation-validation: runtime cargo check deferred while unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice before milestone testing
  - 2026-05-21-textinput-keyboard-text-payload-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs zircon_runtime/src/ui/tests/mod.rs (passed)
  - 2026-05-21-textinput-keyboard-text-payload-validation: tracked git diff --check on TextInput keyboard-text runtime/docs files and direct trailing-whitespace scan of the new keyboard-text test file (passed with LF/CRLF warnings only from git)
  - 2026-05-21-textinput-keyboard-text-payload-validation: runtime cargo check deferred while unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice before milestone testing
  - 2026-05-22-textinput-pointer-selection-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_pointer.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/surface/render/resolve.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/tests/widget_text_input_pointer.rs zircon_runtime/src/ui/tests/mod.rs (passed)
  - 2026-05-22-textinput-pointer-selection-validation: direct trailing-whitespace scan passed for zircon_runtime/src/ui/surface/input/text_pointer.rs and zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - 2026-05-22-textinput-pointer-selection-validation: runtime Cargo validation deferred because unrelated Cargo/Rust compiler jobs remained active in the shared checkout; no Rust diagnostics were produced for this slice
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

`state.rs` stores transient per-surface input ownership that does not belong on individual widgets. The current M2 fields are the captured pointer id, high-precision owner, pointer-lock owner/policy, input-method owner, latest input-method request, active drag/drop session, popup/tooltip state, analog control values, and pointer-drag start/current points for widget-owned drags. Pointer-capture cleanup is owner-aware: the shared captured pointer id is cleared for capture loss, and high-precision ownership is cleared only when it belongs to the released or replaced captor. Pointer-drag state is keyed by the runtime owner and survives the early pointer-up capture clear long enough for TextInput, Range, and ScrollbarThumb release reducers to emit `End` metrics, then is explicitly removed by the reducer or release cleanup path.

`component_state.rs` stores retained per-node component interaction flags such as hovered, focused, and pressed. It is surface-owned rather than TOML-owned: v2 assets describe initial props/state, then runtime input reduces transient interaction state into the heap-resident `UiSurfaceComponentStateStore`. A real transition in those flags marks the owning node render-dirty only; it does not request layout, hit-test, text, input, or visible-range work. Repeated pointer movement over the same target therefore stays idle after the first hover repaint, while hover enter/leave, focus gain/loss, and press/release remain visible to style/render consumers without reloading prototype assets or mutating authored TOML.

`surface/interaction_state.rs` is the shared effective-interaction gate for retained nodes. Pointer press/release events, generic click fallback, and default widget reducers all consult the same inputs before mutating or emitting component events: retained `state_flags.enabled`, runtime component-state `disabled`/`enabled` values and disabled flags, retained `disabled` attributes, and `UiWidgetContract.disabled`. This mirrors Bevy's standard widget pattern where `InteractionDisabled` is checked inside button, checkbox, and slider pointer/keyboard reducers before `Activate` or `ValueChange` is triggered.

`validation.rs` centralizes input-owner checks used by effect application and owner-routed dispatch. A valid owner must exist, be enabled, be render-visible itself, and have an enabled plus render-visible ancestor chain. This keeps text/IME/high-precision ownership aligned with the same visibility semantics used by arranged hit/render paths and prevents disabled containers from leaving focused descendants active.

`effect.rs` consumes `UiDispatchReply` values. It applies ordered `UiDispatchEffect` entries, records applied and rejected effects with their source `effect_index`, validates target nodes before mutating surface state, marks dirty flags for dirty/redraw requests, and creates typed host requests for pointer lock, pointer unlock, high precision pointer, popup, tooltip, and input method effects. `DirtyRedraw` preserves the supplied structured dirty domains: render-only redraws do not set the legacy `state_flags.dirty` bit, because that bit still widens to hit-test/input/render in `UiSurface::dirty_flags()`. Text-edit visual metadata mutations for caret, selection, and composition ranges also stay render-only; the edited `value` property is the only text-edit mutation that drives text layout and measurement. Input-method enable requests can establish a valid owner; reset/update/disable requests require the current IME owner and are rejected before host requests when stale. Optional `UiInputMethodSurroundingText` is validated before a host request is emitted so cursor and anchor byte offsets must stay inside the UTF-8 excerpt and on character boundaries. Owner-sensitive effects now carry their owner target in the shared DTO: `ClearFocus { target, .. }`, `ReleasePointerCapture { target, pointer_id, .. }`, and `UnlockPointer { target, .. }`. Runtime application rejects stale targets before clearing focus/IME, capture/high-precision, or pointer-lock state. `DragDrop Begin`, `Update`, `Accept`, and `Reject` all validate the target through the same input-owner gate before mutating the active session; `Complete` and `Cancel` remain cleanup operations keyed by the active pointer/session so a source can be released even after the hover target disappeared. `SetFocus` and navigation clear a previous IME owner only after focus successfully changes, direct capture clears stale pointer ids, and `UseHighPrecisionPointer { enabled: true }` requires live capture for the same owner.

`dispatch.rs` is the shared entry point adapter. Pointer and navigation events keep delegating to the existing runtime dispatchers, preserving their behavior while projecting the result into `UiInputDispatchResult`. Pointer scroll results keep the original shared pointer input event, including optional precise x/y/unit wheel metadata, even while legacy dispatch consumes the scalar fallback. Keyboard input validates the focused owner before it records the focused route or enters TextInput editing, popup dismissal, and default widget activation; stale disabled or hidden focus therefore reports `owner route rejected` instead of mutating retained widget state. Text input uses a valid IME owner when present, clears stale IME ownership, then falls back to the focused node. IME input clears stale or cancelled IME ownership and reports an `owner route rejected` diagnostic when an invalid stored owner was present. Other owner-routed families validate that the stored owner still exists, is enabled, and has render-visible ancestors before reporting a handled route.

`text_constraints.rs` is the TextInput-specific constraint parser used by `dispatch.rs` before committed text and IME preedit/commit enter `UiTextEditAction`. It reads retained node attributes, not editor authoring state: `max_graphemes`, `max_chars`, or `max_length` bound the replacement by Unicode grapheme count; `input_filter` or `text_filter` can restrict inserted text to `digits`, `number`, `ascii`, or `alphanumeric`; and explicit `multiline = false` strips CR/LF from inserted or preedit text. The same retained multiline flag gates focused Enter handling: multiline fields insert `"\n"` through the committed-text path, while single-line fields leave Enter unhandled for submit/default behavior. Existing text is not rewritten, so constraints only govern the replacement payload currently entering the runtime-owned field. The constraint function is crate-visible through `surface/mod.rs` so accessibility `SetValue` can sanitize whole-field replacements and `ReplaceSelectedText` can sanitize selected-range replacements with the same runtime rules instead of bypassing TextInput filters, multiline policy, or max length.

M2 focus routing now records focused input through the runtime-owned `UiFocusState::focused_inputs` log. Keyboard, navigation, text, and IME dispatch paths record the focused node, the bubble route from focused node to root, the handler or owner that accepted the focused-input route, and whether the focused route was accepted. This mirrors Bevy's focused-input bubbling concept while keeping the route serializable in `zircon_runtime_interface::ui::focus::UiFocusedInput`.

Focus mutation lives in `surface/focus.rs` instead of growing `surface.rs` further. Programmatic focus uses `UiFocusChangeReason::Programmatic`; pointer focus uses `Input` and hides focus-visible by default unless a pointer handler explicitly requests a visible focus ring; keyboard/navigation focus uses `Navigation` and `KeyboardNavigation`. Autofocus resolves from `pending_autofocus` or the first authored `UiFocusContract::autofocus` node and records an `Autofocus` focus-change event. Re-focusing the already focused node updates the visible policy but does not append a duplicate focus-change event.

Focus reconciliation is driven by accepted tree changes only. Runtime property mutation clears focus when an accepted `disabled`, `enabled`, `visible`, `visibility`, or `focusable` mutation makes the focused node invalid, and records `Disabled` or `Hidden` according to the property that invalidated focus. The `disabled` path matters for authored/runtime attribute gates because `disabled = true` is not the inverse `enabled` state flag; it is mirrored into component state and then checked by the shared input-owner validator. Unchanged and rejected mutations do not emit focus changes. Detaching a subtree to the surface node pool clears focus, capture, press, hover, high-precision, IME, pointer-lock, and active drag/drop state for detached owners with `Despawned` focus-change reason.

The focus cleanup path rebuilds the hover-owner list from valid owners instead of retaining with a closure that also borrows the surface for validation. Editable text dispatch captures the focused-input kind before moving the shared input event into `UiInputDispatchResult`, and keyboard focused-input records an accepted focused route whenever a focused owner exists even if the current low-level keyboard reply remains unhandled pending a widget reducer. These are compile-level invariants for the runtime-owned focus/input state and do not change navigation ordering.

M3 navigation behavior consumes the M1 `UiNavigationContract` stored on `UiTreeNode`. `next_navigation_target(...)` applies tab indices, group order, modal group traps, manual directional node/group overrides, blocked edges, and spatial fallback from arranged node centers. Non-modal groups contribute ordering rather than trapping traversal; modal groups filter both tab traversal and manual directional overrides so a dialog cannot escape through an authored neighbor. Existing tree-order helpers remain available for older tests and narrow callers, but surface navigation dispatch uses the contract-aware traversal.

## Widget Behavior Contract

Headless widget behavior is now authored through `UiWidgetContract::behavior` instead of being inferred only from a component name at each runtime call site. `UiWidgetBehavior::Auto` preserves existing templates by mapping known components such as `Button`, `Checkbox`, `Slider`, `TextField`, and `MenuItem` onto behavior families. Explicit values such as `Toggle`, `Range`, or `TextInput` override that fallback, and `Passive` prevents a known component name from gaining default behavior accidentally.

Default pointer and focused-keyboard behavior read the resolved widget behavior once and then apply the behavior family. Buttons and menu items emit the same activated commit through click bindings for pointer release and focused `Enter`/`Space`; pointer double-click keeps the existing `double_activated` binding event while still being owned by the button behavior reducer. Menu item popup close is owned by the MenuItem reducer even when the menu item itself has no activation binding, so pointer, keyboard, and accessibility activation all close the nearest open popup through the same popup mutation path. Toggle widgets use `checked_property` with `checked` as the legacy fallback. Disclosure and popup widgets use `open_property` with `expanded` or `popup_open` as the current family default. Range widgets use `value_property`, `min_property`, `max_property`, and `step_property`, so custom TOML components can participate in drag, step, and value-change events without adopting the built-in `Slider` or `RangeField` component names.

Typed widget ownership remains in effect even when a widget is disabled: a disabled button, menu item, toggle, disclosure, popup, or range does not fall through to the generic click fallback. Instead, the typed reducer owns the route and no-ops because the shared interaction gate rejects it. That keeps runtime pointer, focused keyboard, and accessibility disabled semantics aligned with Bevy's `InteractionDisabled` checks in `button.rs`, `checkbox.rs`, and `slider.rs`.

Range navigation now follows the same Bevy slider keyboard contract for incremental and endpoint changes. Focused `Left`/`Down` and `Right`/`Up` navigation step through the authored `step_property`; focused `Home` and `End` navigation mutate the authored `value_property` to the current `min_property` or `max_property`. The endpoint path still goes through `UiSurface::mutate_property`, so render-only dirty behavior and runtime component-state value mirroring stay consistent with pointer drag and accessibility increment/decrement.

The Range default reducer now lives in `surface/default_interactions/range.rs`. The parent `default_interactions.rs` remains the common widget-action dispatcher for Button/MenuItem, Toggle, Disclosure, and Popup behavior, while the child module owns Range pointer drag, numeric alias lookup, clamp/step math, and navigation endpoint mutation. This keeps the Bevy slider-aligned behavior family isolated before adding further standard widgets.

Radio behavior follows the Bevy `RadioGroup`/`RadioButton` split in `dev/bevy/crates/bevy_ui_widgets/src/radio.rs`: a selected radio does not emit another change, a disabled radio or disabled nearest group is rejected by the shared interaction gate, and a radio can expose an app-specific value through `UiWidgetContract::value`. Zircon intentionally applies the group selection through `UiSurface::mutate_property` because retained templates and runtime component state are the current data-binding layer. `surface/default_interactions/radio.rs` sets the target radio checked, clears checked sibling radios in the nearest `RadioGroup`, and mutates the group's `value_property` to the selected radio's `widget.value` or stable node identity fallback. Godot's `ButtonGroup` documentation corroborates the mutual-exclusion expectation, while Slint's Material `RadioButton` documentation confirms the author-facing `checked`/`enabled` surface.

Scrollbar behavior follows Bevy `dev/bevy/crates/bevy_ui_widgets/src/scrollbar.rs`: scrollbar track and thumb are headless controls for a scrollable container, not slider-like value widgets. `UiWidgetBehavior::Scrollbar` carries `scroll_target`, optional `scroll_axis`, and `min_thumb_extent`; `ScrollbarThumb` is a passive child marker. The runtime reducer in `surface/default_interactions/scrollbar.rs` resolves a target node by `#node_id`, `control_id`, or node path, then a primary track click pages the target `ScrollableBox` by its viewport extent through `UiRuntimeTreeScrollExt::set_scroll_offset`. Thumb press captures the pointer, starts runtime pointer-drag metrics, and emits `BeginDrag`; captured move updates the target scroll offset and emits `DragDelta` with `UiDragPhase::Update`, start/current points, axis delta, and distance; release emits `EndDrag` with `UiDragPhase::End` before clearing the drag state. A thumb click still does not page the scroll container because paging belongs to the track, not the thumb.

MenuItem activation now owns the first headless menu close behavior. Bevy `dev/bevy/crates/bevy_ui_widgets/src/menu.rs` activates menu items and then sends menu close/focus-root events for the popup stack; the same file also treats Escape as a popup close request. Zircon does not yet have a full neutral menu-stack contract, so `default_interactions.rs` applies the conservative retained equivalent: after a `MenuItem` pointer click or focused Enter/Space activation, or after Escape is pressed while focus is inside a popup, the nearest `Popup` ancestor with an open `open_property` is mutated to `false` only when it passes the ancestor-aware widget interaction gate, and matching popup bindings receive `ClosePopup`. The close path no longer depends on the MenuItem itself having an activation binding; that keeps menu closing in the same runtime behavior path for pointer, keyboard, and accessibility activation without inventing editor-owned popup state.

The keyboard path handles the same default activation keys used by Bevy's headless widgets: `Enter` and `Space` on the focused control. Buttons and menu items emit an activated commit through click bindings; toggles, disclosures, and popups mutate the same properties used by pointer activation. Repeated keyboard events do not trigger this default activation path.

Focused TextInput keyboard editing now uses the same retained editable text state as text and IME input. `surface/input/text_keyboard.rs` maps Bevy-style focused edit keys from local `dev/bevy/crates/bevy_ui_widgets/src/editable_text.rs` into `UiTextEditAction`: Backspace, Delete, Escape collapse/cancel, ArrowLeft/ArrowRight, ArrowUp/ArrowDown, Home, End, select-all, keyboard text payload, and multiline Enter insertion. Arrow/Home/End actions read `UiInputEventMetadata.modifiers.shift` so Shift+movement extends the retained `selection_anchor`/`selection_focus`, while plain movement clears the selection and only moves `caret_offset`. Home and End now use shared line-boundary helpers for multiline text: plain Home/End moves to the current line start/end, while Ctrl/Super+Home/End moves to the document start/end; CRLF line endings are treated as one separator and End stops before the carriage return. Bevy also exposes hard-line shortcuts through Super+ArrowLeft/Right and Alt+Home/End; Zircon maps those to the same current hard-line start/end helpers until visual-line shaping geometry exists. Plain ArrowUp/ArrowDown use shared grapheme-column line helpers, so vertical movement targets the same logical grapheme column, clamps to shorter lines, treats CRLF as one line break, and stays on valid UTF-8 boundaries until the later shaping backend can supply visual caret x positions. Up on the first line collapses to document start and Down on the last line collapses to document end. Ctrl/Super+ArrowUp/ArrowDown follows Bevy's `TextStart`/`TextEnd` mapping and moves to the document start/end through the same retained selection path. ArrowLeft/ArrowRight also read `UiInputEventMetadata.modifiers.control` for word navigation: Ctrl+Left moves to a word start and Ctrl+Right moves to a word end, with Ctrl+Shift extending selection through the same retained selection properties. Ctrl+Backspace and Ctrl+Delete use the same word-boundary helper to select the previous or next word range and then reuse the existing Backspace/Delete selection-removal path, so word deletion reports the same value/caret/selection mutations as normal deletion. Ctrl+A and Super+A map to the existing `SetSelection` action and select the full retained text without emitting a value-change component event. Escape follows Bevy's `CollapseSelection` intent when no composition is active: it clears the retained selection by moving the caret to the current focus without changing value. If composition metadata is active, Escape keeps Zircon's IME safety behavior and cancels composition first, restoring the saved text through `CancelComposition`. Read-only TextInput still permits caret movement, selection updates, Escape collapse, and Copy, but blocks content mutation, composition writes, Cut, and Paste. Disabled or hidden focused owners are rejected before this edit path, so stale focus cannot use keyboard text payloads, editing keys, Escape collapse, popup dismissal, or default activation to mutate retained state. A plain focused Enter matches Bevy's `allow_newlines` branch: when retained `multiline` allows it, Enter inserts `"\n"` through the same committed-text path used by `UiInputEvent::Text`; when `multiline = false`, Enter is not consumed by text editing and can propagate to submit/default behavior. Printable `UiKeyboardInputEvent.text` payloads with no Control/Alt/Super modifier now also use that committed-text path, so hosts that combine key and text data do not need a second `UiInputEvent::Text`; Tab and newline payloads remain unhandled for navigation and submit/default behavior. Clipboard shortcuts now cover Ctrl/Super+C/X/V, dedicated Copy/Cut/Paste logical keys, and Shift+Delete cut. All of them emit `UiDispatchEffect::RequestClipboard` with a typed `UiDispatchHostRequestKind::Clipboard`; Copy and Paste do not invent runtime clipboard state or binding reports, while Cut removes the active selection through the same value/caret/selection mutation route before requesting the host clipboard write. Paste requests host text and expects the host to feed the pasted payload back through the normal `UiInputEvent::Text` route. Caret movement plus Backspace/Delete share `ui/text/grapheme.rs` boundaries, so combining marks and emoji clusters are treated as one editable unit before the later shaping backend supplies full glyph geometry. Active non-empty selections are replaced by text insertion, keyboard text payload, keyboard Enter newline, IME preedit, or IME commit and are removed by Backspace/Delete before caret-local insertion or deletion is considered, then `selection_anchor` and `selection_focus` collapse to the new caret offset. IME preedit writes `composition_start`, `composition_end`, `composition_text`, and `composition_restore_text` beside the temporary value; cancel restores the saved text and clears the IME owner; commit replaces the composition span, clears composition metadata, and can emit a Submit/Commit component event when authored. `dispatch.rs` applies those actions before default widget activation, so text fields do not accidentally treat editing keys as button activation; text insertion is owned by `UiInputEvent::Text`, focused keyboard text payloads, focused Enter newline, or IME commit. Custom controls that author `UiWidgetBehavior::TextInput` and `value_property` are editable even when their component name is not one of the legacy text field names. Replacement constraints now sit beside this route rather than in the editor: retained TextInput attributes filter and truncate incoming text/IME payloads against the actual replacement range, including active selections, before the shared edit action applies. This preserves one runtime truth for value, caret, selection, composition, dirty domains, binding reports, and host clipboard requests.

Focused TextInput pointer editing now uses the same editable state and layout-backed geometry. `surface/input/text_pointer.rs` consumes the `UiResolvedTextLayout` already stored on the render command, calls `ui/text/hit_test.rs`, and then applies `UiTextEditAction::MoveCaret` through `apply_editable_text_state(...)`. A primary press moves the caret, captures the pointer for that TextInput, and records `UiDragPhase::Begin`; Shift+press extends the retained selection from the existing caret; captured pointer move extends selection while recording `UiDragPhase::Update`; release records `UiDragPhase::End` before removing the owner-keyed drag state. The shared `UiInputDispatchResult.drag` field carries these metrics even when text selection changes do not emit a value-change component event. The TextInput pointer path also checks `is_valid_input_owner(...)` before caret mutation or capture, so retained disabled flags, component-state disabled values, hidden ancestors, and disabled ancestors reject pointer caret/selection just like keyboard, text, IME, accessibility, and typed widget reducers. The render resolver now treats explicit `UiWidgetBehavior::TextInput` and custom `value_property` controls as visible-value text sources. Empty editable string values still create an editable text state with an empty buffer, but render/layout text resolution falls through to `placeholder` for visual measurement when the value is empty. This keeps Material `TextField` placeholders visible and measurable without writing placeholder text into the editable buffer that keyboard, pointer, IME, and accessibility mutation own.

TextInput placeholder/value fallback evidence from 2026-05-24:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/component/catalog/material_foundation/surfaces.rs" "zircon_runtime/src/ui/surface/render/resolve.rs" "zircon_runtime/src/ui/tests/material_layout.rs"`: PASS.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked material_fields_measure_visible_value_placeholder_and_options_text --jobs 1 --message-format short --color never`: PASS, 1 matching test passed after unrelated in-flight SDF text report compile drift was corrected by its owning UI session.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked material --jobs 1 --message-format short --color never`: PASS on 2026-05-26, 90 matching lib tests plus the matching export contract test passed after the folder-backed Material catalog guard was restored by splitting editor-facing data-display descriptors.

This follows the Bevy standard-widget split: `dev/bevy/crates/bevy_ui_widgets/src/lib.rs` defines unstyled widgets that emit `Activate` and `ValueChange<T>` events, while `button.rs`, `checkbox.rs`, and `slider.rs` keep pointer and focused-keyboard behavior separate from styling. Zircon deliberately keeps the behavior contract serialized in `zircon_runtime_interface` because templates and editor-authored widgets need stable runtime behavior without depending on editor-specific styling code.

The focused regression in `zircon_runtime/src/ui/tests/pointer_click_semantics.rs` covers a custom `FavoritePill` component that opts into `Toggle` behavior and mutates a custom `selected` property from both pointer release and Space-key activation. It also covers a button widget contract that emits an activated commit from focused Enter-key activation while the existing pointer click and double-click tests continue to exercise the typed button pointer reducer. Runtime component-state `disabled = true` now blocks button pointer press/click/double-click emission, focused keyboard activation, and toggle pointer/keyboard mutation. `zircon_runtime/src/ui/tests/widget_range_navigation.rs` covers focused Range `Home`/`End` endpoint mutation through authored min/max/value aliases. The interface contract test in `zircon_runtime_interface/src/tests/ui_contract_spine.rs` covers serde round-trip behavior for the new behavior enum and property aliases.

`zircon_runtime/src/ui/tests/widget_radio_behavior.rs` covers Radio pointer selection, already-checked no-op behavior, disabled group rejection, focused keyboard activation, group value mutation, sibling uncheck, and a11y role/action projection for `RadioGroup` and `Radio`.

`zircon_runtime/src/ui/tests/widget_menu_behavior.rs` covers MenuItem pointer and focused keyboard activation closing the nearest Popup ancestor while delivering the item activation event when one is bound, focused keyboard activation still closing the Popup when the MenuItem has no item binding, focused Escape closing the popup without item activation, and disabled popup owners rejecting descendant focus, stale Escape close, and outside-click close.

MenuItem activation close follow-up evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/surface/input.md" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime/ui/binding/update_report.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0005-ui-menuitem-activation-close.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Runtime Cargo validation remains deferred because 4 unrelated `cargo` and 4 `rustc` processes remain active in the shared checkout.

`zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs` covers TextInput focused Backspace through an authored `value_property` on a custom component name and ArrowLeft caret movement without emitting a value-change component event when text content is unchanged. It also covers read-only ArrowLeft caret movement, read-only Backspace no-op mutation, grapheme-cluster ArrowLeft, Backspace, and Delete with a combining-mark cluster, Shift+ArrowLeft and Shift+Home extending the retained selection through the same binding-report path without a value-change component event, multiline Home/End current-line movement, CRLF Home/End boundaries, ArrowUp/ArrowDown same-column movement, first-line and last-line vertical edge collapse, shorter-line clamping, CRLF vertical movement, Shift+ArrowDown multiline selection, grapheme-column vertical movement, Ctrl+ArrowUp document-start movement, Ctrl+Shift+ArrowDown document-end selection extension, Shift+End current-line selection extension, Ctrl+End document-end movement, Ctrl+ArrowLeft/Ctrl+ArrowRight word navigation, Ctrl+Shift+ArrowRight word selection, Ctrl+Backspace/Ctrl+Delete word deletion, Ctrl+A select-all, Escape collapsing an active selection without value change, Escape cancelling active composition before selection collapse, Ctrl+C clipboard write for an active selection, Ctrl+X cutting that selection through the normal mutation path before a clipboard write host request, Ctrl+V clipboard read host request without local value mutation, dedicated Copy/Cut/Paste logical keys, Shift+Delete cut, Enter inserting newline in multiline fields, Enter replacing an active selection with newline, Enter remaining unhandled for explicit single-line fields, text insertion replacing an active selection, Backspace deleting an active selection, selection replacement respecting `max_chars`, IME preedit replacing an active selection and tracking composition metadata, IME cancel restoring the replacement text and clearing the owner, IME commit replacing composition with a Submit/Commit event, focused text insertion applying `input_filter = "digits"` with `max_chars`, and explicit `multiline = false` stripping CR/LF before value/caret mutation and binding report emission.

`zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs` keeps new hard-line navigation coverage out of the already-large keyboard test file. It covers Super+ArrowLeft moving to current hard-line start, Super+Shift+ArrowRight extending to hard-line end, Alt+Shift+Home extending to hard-line start, and Alt+End moving to hard-line end through the same binding-report path.

`zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs` covers `UiKeyboardInputEvent.text` printable insertion, constraints plus active selection replacement, stale disabled focused-owner rejection before text payload mutation, Tab remaining unhandled for navigation, and single-line Enter not being bypassed by a newline keyboard payload.

`zircon_runtime/src/ui/tests/widget_text_input_mui.rs` covers MUI `InputBase` component-name inference staying routed to the editable TextInput owner. The 2026-05-27 M5 runtime rerun now accepts multiple text-state synchronization binding reports for caret, selection, and composition metadata, while still requiring exactly one applied `WidgetBehavior` value update on the owner node.

`zircon_runtime/src/ui/tests/widget_text_input_pointer.rs` covers primary pointer press moving the caret and capturing the pointer with `Begin` drag metrics, Shift+press extending selection, captured pointer drag extending selection after the cursor leaves the original press point with `Update` metrics, pointer release with `End` metrics and capture cleanup, disabled TextInput pointer press refusing caret/capture mutation without drag metrics, and empty TextInput value fallback still producing usable layout-backed hit testing.

`zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs` covers accessibility `SetValue` on TextInput rejecting read-only fields before mutation, applying `input_filter`, `max_chars`, and `multiline = false` constraints before value/caret/selection binding reports are produced, clearing stale composition metadata after full-field replacement, and accessibility `ReplaceSelectedText` replacing only the selected range with the same constraint and composition-cleanup path.

`zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs` covers Scrollbar track click paging a target `ScrollableBox`, thumb click no-op behavior, thumb drag `Begin`/`Update`/`End` metrics, default a11y exclusion for headless scrollbar widgets, explicit scrollbar a11y opt-in, and accessibility `ScrollTo` mutating the scrollable container offset. `zircon_runtime/src/ui/tests/widget_range_navigation.rs` covers focused Range `Home`/`End` endpoint mutation plus pointer drag `Begin`/`Update`/`End` metrics through authored min/max/value aliases.

## Surface Integration

`UiSurface` now stores `input: UiSurfaceInputState` with serde defaults so old retained surface snapshots can deserialize without input-state fields. Public methods `UiSurface::apply_dispatch_reply(...)` and `UiSurface::dispatch_input_event(...)` keep the shared runtime seam explicit while the implementation remains in the child module.

Pointer capture release paths clear the shared pointer id in addition to clearing `UiFocusState::captured`. When the released owner also owns high precision, `clear_pointer_capture_for(owner)` clears both states; otherwise high precision is left alone so a stale release cannot clear another owner's raw-input state.

Pointer dispatch dirty requests are now first-class retained-surface input invalidation. `UiSurface::apply_pointer_dispatch_dirty(...)` maps each `UiPointerDispatchEffect::RequestDirty(...)` invocation back to the invoking node and preserves the supplied `UiDirtyFlags` without widening render-only requests through the legacy `state_flags.dirty` bit. `RuntimeUiManager` consumes that method after pointer dispatch and only calls `rebuild_dirty(root_size)` when the surface actually has dirty domains. Pointer-derived component state follows the same narrow domain: `apply_pointer_component_state(...)`, `set_node_pressed_dirty(...)`, and `surface/focus.rs` mark render dirty only when the retained hover/focus/press state actually changes. This means runtime v2 fixtures can redraw a pressed/hovered/focused visual through the render domain, or run incremental layout for real geometry changes, without reloading v2 TOML or rebuilding the entire surface tree.

Pointer primary press/release now also reduces the retained pressed state into `UiTreeNode::state_flags.pressed`. The update is intentionally render-only: it changes the accessibility/render-facing state, adds a render dirty flag to the affected node, and does not set the legacy `state_flags.dirty` bit. A second primary press on another target clears the previous pressed node and marks both nodes render-dirty, matching retained widget behavior without forcing layout or hit-grid rebuilds.

Runtime property mutation synchronizes accepted tree property changes into `UiSurfaceComponentStateStore`, including generic typed values and canonical boolean flags such as disabled, pressed, checked, expanded, popup-open, and selected. Unchanged or rejected mutations do not initialize or rewrite component state and do not mark render dirty. This preserves the public `UiPropertyMutationStatus::Unchanged` contract while still letting real accepted mutations feed runtime pseudo-state styling and accessibility state extraction through the component-state path.

Pointer hover, focus, and press also reduce into `UiSurfaceComponentStateStore`. Hover/focus state is retained for component/style consumers but does not yet widen dirty domains by itself; press still marks render dirty through the tree state path because current render and accessibility extraction already read `state_flags.pressed`. Detaching a subtree to the retained node pool clears component states for those nodes before reuse, so recycled controls cannot inherit stale hover/focus/press flags.

Focused input owner validation now rejects disabled ancestors through the shared `ui_surface_node_disabled(...)` helper used by widget interaction and accessibility extraction: `state_flags.enabled`, component-state `disabled`/inverse `enabled`, canonical runtime disabled flags, `UiWidgetContract.disabled`, and retained `disabled = true`. Accepted `disabled = true` runtime property mutations now immediately run the same reconcile path as `enabled = false`, so stale focus, capture, high-precision pointer, IME, pointer-lock, pressed, hover, and active drag/drop owners are cleared before a later input event can observe them. That means a node under a disabled popup cannot regain keyboard, IME, pointer-capture, drag/drop, or popup-close ownership merely because the child itself is enabled. Escape and outside-click popup dismissal consume the same ancestor-aware widget gate, so disabled popup owners are not closed by stale focus or pointer paths.

The 2026-05-27 M5 runtime gate also filters pointer-derived focus acquisition through the same input-owner gate. Pointer hit testing may still identify a disabled node so diagnostics and suppression remain precise, but `UiSurface` no longer tries to move focus to that disabled route target. Disabled Button and TextInput hits therefore suppress pointer/keyboard mutation without producing a stale focus owner or a `MissingNode` focus error.

Popup and tooltip input/effect DTOs now carry an optional `owner: UiNodeId`. Runtime dispatch copies that owner from `UiPopupInputEvent` and `UiTooltipTimerInputEvent` into the corresponding `UiDispatchEffect`, and `apply_dispatch_reply(...)` validates the owner with the same input-owner gate before mutating `popup_stack` or tooltip state or before emitting host requests. A missing owner remains the legacy host-only path for old serialized events and tests; an explicit stale, hidden, or disabled owner rejects the effect and leaves shared popup/tooltip state unchanged.

Pointer component events and shared input dispatch results now carry optional `UiDragMetrics` for runtime-owned drag gestures. `UiSurfaceInputState` records start/current points by owner; Range plus ScrollbarThumb reducers attach `Begin`, `Update`, and `End` metrics to their existing `BeginDrag`, `DragDelta`, and `EndDrag` component events; and TextInput pointer selection stores the same metrics on `UiInputDispatchResult.drag` because selection-only pointer edits often have no component event to decorate. `dispatch.rs` preserves legacy `UiPointerComponentEvent.drag` values when projecting pointer dispatch into `UiComponentEventReport.drag`, and `push_text_component_event_report(...)` carries TextInput drag metrics onto any text component event that is emitted. This keeps value/scroll deltas in the existing component event variants while giving input-manager consumers a neutral phase/distance DTO for drag thresholds, selection geometry, and future editor drag behavior.

## Validation Scope

Focused runtime coverage lives in `zircon_runtime/src/ui/tests/event_routing.rs`, `zircon_runtime/src/ui/tests/runtime_input_ownership.rs`, `zircon_runtime/src/ui/tests/popup_tooltip_state.rs`, `zircon_runtime/src/ui/tests/widget_range_navigation.rs`, `zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs`, `zircon_runtime/src/ui/tests/widget_text_input_pointer.rs`, `zircon_runtime/src/ui/tests/surface_dirty_domains.rs`, and the runtime-host boundary tests in `zircon_runtime/src/tests/ui_boundary/runtime_host.rs`. The ownership tests were split out because `event_routing.rs` is already above the large-file warning threshold. Together they verify focus/capture/high-precision reply effects, owner-targeted clear-focus behavior, rejected focus preserving the current IME owner, navigation/focus changes clearing previous IME owners, direct clear-focus cleanup, focus/capture rejection through hidden ancestors, disabled-property mutation on an ancestor clearing focused descendants plus transient owners and active drag/drop state, direct capture clearing stale pointer ids before high precision can enable, stale pointer-capture release rejection, stale pointer-lock unlock rejection, high-precision enable requiring live capture, stale high-precision disable rejection, capture transfer clearing the previous captor's high precision, navigation plus host-owned input effects, input-method reset/update/disable current-owner checks, invalid input-method owner rejection, drag/drop pointer/session/target owner rejection without clearing the active drag, Range/ScrollbarThumb component-event drag phase/distance metrics, TextInput pointer-selection input-result drag phase/distance metrics, explicit popup/tooltip owner rejection without state mutation or host requests, focused keyboard diagnostics, text owner routing/fallback after stale IME cleanup, stale IME owner-route rejection, hidden-ancestor owner rejection, disabled-ancestor owner rejection, IME owner cleanup, pointer scroll diagnostics plus precise scroll metadata preservation through the shared input result path, render-only `DirtyRedraw` effects staying out of hit-test/input rebuilds, runtime fixture pointer dirty requests being consumed through the persistent v2 `UiSurface`, primary press/release retained-state reduction staying render-only, component-state store updates for hover/focus/press, and node-pool cleanup for component states and drag/drop sessions.

Ancestor disabled gate evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/surface/interaction_state.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/surface/surface/interaction_state.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime/ui/surface/input.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0055-ui-ancestor-disabled-gate.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Runtime Cargo validation remains deferred because 10 unrelated `cargo` and 4 `rustc` processes remain active in the shared checkout.

Shared disabled gate helper evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/interaction_gate.rs" "zircon_runtime/src/ui/surface/mod.rs" "zircon_runtime/src/ui/surface/surface/interaction_state.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/surface/interaction_gate.rs" "zircon_runtime/src/ui/surface/mod.rs" "zircon_runtime/src/ui/surface/surface/interaction_state.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime/ui/surface/input.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0060-ui-shared-disabled-gate-helper.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. A lightweight `cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-disabled-gate-helper" --message-format short --color never` attempt was blocked before compilation because Cargo needed to update `Cargo.lock` and `--locked` forbids that; locked Cargo validation needs a separate lockfile-state pass before it can produce Rust diagnostics.

Disabled property owner-cleanup evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/tests/runtime_input_ownership.rs"`: first run reported formatting diffs in the new owner test; after applying `rustfmt` and simplifying metadata assignment, rerun PASS.
- Focused coverage now asserts `mutate_property(root, "disabled", true)` mirrors disabled component state, reports a `Disabled` focus change for a focused descendant, and clears stale focus, pointer capture, captured pointer id, pressed/hovered owners, high-precision pointer, IME owner, pointer lock owner, and pointer lock policy.
- `git diff --check` and trailing-whitespace scan cover the touched runtime files, this input module document, the UI/Text/Input/A11y plan, and the owner-cleanup session note. Runtime Cargo validation is still deferred because the shared checkout has a dirty `Cargo.lock` and an earlier focused `--locked` check stopped before compilation while requesting a lockfile update.

Drag/drop transient owner-cleanup evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/focus.rs" "zircon_runtime/src/ui/tests/runtime_input_ownership.rs" "zircon_runtime/src/ui/tests/surface_node_pool.rs"`: PASS.
- Focused coverage now asserts accepted `disabled=true` ancestor mutation clears active drag/drop state along with the drag source capture, and subtree detach clears a drag/drop session when the detached nodes contain the drag source or target.
- Runtime Cargo validation is still deferred for this slice under the same dirty lockfile constraint recorded above.

Drag/drop target owner-validation evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/input/effect.rs" "zircon_runtime/src/ui/tests/runtime_input_ownership.rs"`: PASS.
- Focused coverage now asserts a drag `Update` to a disabled target is rejected with the shared `invalid input owner` reason and leaves the active drag target/point unchanged.
- Runtime Cargo validation is still deferred for this slice under the same dirty lockfile constraint recorded above.

TextInput pointer disabled-gate evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/input/text_pointer.rs" "zircon_runtime/src/ui/tests/widget_text_input_pointer.rs"`: PASS.
- Focused coverage now asserts disabled TextInput pointer press does not enter `pointer.text_press`, does not produce binding reports, does not capture the pointer, and leaves caret/selection retained attributes unchanged.
- Runtime Cargo validation is still deferred for this slice under the same dirty lockfile constraint recorded above.

Focused keyboard owner-gate evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs"`: PASS.
- Focused coverage now asserts a stale disabled focused TextInput owner rejects keyboard text payloads with `owner route rejected`, leaves value/caret/selection unchanged, and emits no component events or binding reports.
- Runtime Cargo validation is still deferred for this slice under the same dirty lockfile constraint recorded above.

M5 disabled pointer-focus and MUI TextInput owner evidence from 2026-05-27:

- `rustfmt --edition 2021 --check zircon_runtime\src\ui\surface\surface.rs zircon_runtime\src\ui\tests\pointer_click_semantics.rs zircon_runtime\src\ui\tests\widget_text_input_mui.rs`: PASS.
- Exact focused reruns passed for disabled pointer/keyboard Button suppression, disabled TextInput pointer suppression, and `ui::tests::widget_text_input_mui::mui_input_base_component_name_is_editable_text_owner`.
- `cargo test -p zircon_runtime --lib --locked --target-dir F:\cargo-targets\zircon-platform-m5-workspace --message-format short --color never -- --format terse`: PASS with 2102 passed, 0 failed.

Popup/tooltip explicit owner-gate evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/ui/dispatch/input/effect.rs" "zircon_runtime_interface/src/tests/contracts.rs" "zircon_runtime/src/ui/surface/input/effect.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/runtime_input_ownership.rs" "zircon_runtime/src/ui/tests/popup_tooltip_state.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- Focused coverage now asserts popup open and tooltip arm events carrying a disabled explicit owner reject with the shared `invalid input owner` reason, do not mutate popup/tooltip state, and do not emit host requests. Interface contract coverage adds owner fields to popup/tooltip event/effect roundtrips and confirms legacy serialized payloads without `owner` still deserialize with `None`.
- Runtime/interface Cargo validation is still deferred for this slice because unrelated Cargo processes remain active in the shared checkout.

Drag phase/distance DTO evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/component/drag.rs" "zircon_runtime_interface/src/ui/component/mod.rs" "zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs" "zircon_runtime_interface/src/tests/contracts.rs" "zircon_runtime/src/ui/surface/input/state.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/focus.rs" "zircon_runtime/src/ui/surface/surface/default_interactions/range.rs" "zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs" "zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs" "zircon_runtime/src/ui/tests/widget_range_navigation.rs"`: PASS.
- Focused coverage now asserts `UiPointerComponentEvent.drag` roundtrips and legacy payloads default to `None`, ScrollbarThumb drag emits `Begin`/`Update`/`End` metrics with stable start/current/delta/distance, and Range pointer drag emits the same metrics while preserving existing value-change and drag-delta component events.
- Runtime/interface Cargo validation is still deferred for this slice because unrelated Cargo/Rust compiler processes remain active in the shared checkout.

Shared/TextInput drag metrics evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/dispatch/input/result.rs" "zircon_runtime_interface/src/tests/contracts.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/surface/input/effect.rs" "zircon_runtime/src/ui/surface/input/text_pointer.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/widget_text_input_pointer.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: first run reported formatting diff in `contracts.rs`; after applying rustfmt, rerun PASS.
- Focused coverage now asserts `UiInputDispatchResult.drag` and `UiComponentEventReport.drag` roundtrip and legacy payloads default to `None`; TextInput pointer press emits `Begin`, captured move emits `Update`, pointer release emits `End`, and disabled TextInput pointer press emits no drag metrics.
- Runtime/interface Cargo validation is still deferred for this slice because unrelated Cargo/Rust compiler processes remain active in the shared checkout.

The original M2 scope deliberately did not implement M6 text layout, caret, selection, shaping, or editor-native keyboard/IME translation. Current M4/M6 slices have since added retained caret/selection mutation for focused TextInput keyboard, text, and IME routes; shaping/layout geometry and editor-native keyboard/IME translation still consume this shared input state and result contract instead of adding host-owned focus, capture, or IME semantics.
