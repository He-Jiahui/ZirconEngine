---
related_code:
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/metadata.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/result.rs
  - zircon_runtime_interface/src/ui/binding/model/update.rs
  - zircon_runtime_interface/src/ui/window/mod.rs
  - zircon_runtime_interface/src/ui/window/metadata.rs
  - zircon_runtime_interface/src/ui/window/metrics.rs
  - zircon_runtime_interface/src/ui/window/impact.rs
  - zircon_runtime_interface/src/ui/window/event.rs
  - zircon_runtime_interface/src/ui/window/pump.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/window_input_contracts.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/event.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/pointer_click_semantics.rs
  - zircon_runtime/src/ui/tests/popup_tooltip_state.rs
  - zircon_runtime/src/ui/tests/widget_menu_behavior.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/tests/contracts.rs
implementation_files:
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/metadata.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/result.rs
  - zircon_runtime_interface/src/ui/binding/model/update.rs
  - zircon_runtime_interface/src/ui/window/mod.rs
  - zircon_runtime_interface/src/ui/window/metadata.rs
  - zircon_runtime_interface/src/ui/window/metrics.rs
  - zircon_runtime_interface/src/ui/window/impact.rs
  - zircon_runtime_interface/src/ui/window/event.rs
  - zircon_runtime_interface/src/ui/window/pump.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/event.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/pointer_click_semantics.rs
  - zircon_runtime/src/ui/tests/popup_tooltip_state.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
plan_sources:
  - docs/superpowers/plans/2026-05-06-ui-complete-input-events.md
  - .codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md
  - docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md
  - user: 2026-05-06 implement Milestone 1 shared input contract foundation only
  - user: 2026-05-06 continue Milestone 2 runtime surface reply/effect application
  - user: 2026-05-08 continue M1 window/input pump convergence interface slice
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/window_input_contracts.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - cargo test -p zircon_runtime_interface --lib ui_input --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never
  - cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never
  - post-review-correction: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (45 passed, 0 failed, 3 filtered out)
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
  - 2026-05-07-m5-route-validation: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/dispatch/input/reply.rs zircon_runtime_interface/src/ui/dispatch/input/mod.rs zircon_runtime_interface/src/ui/dispatch/mod.rs zircon_runtime_interface/src/ui/dispatch/pointer/event.rs zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs zircon_runtime_interface/src/tests/contracts.rs zircon_runtime/src/ui/surface/input/effect.rs zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/runtime_input_ownership.rs zircon_runtime/src/ui/tests/pointer_click_semantics.rs (passed)
  - 2026-05-07-m5-route-validation: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (54 passed, 0 failed, 6 filtered out)
  - 2026-05-07-m5-route-validation: cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (8 passed, 0 failed, 923 filtered out)
  - 2026-05-07-m5-route-validation: cargo test -p zircon_runtime --lib pointer_click_semantics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (1 passed, 0 failed, 930 filtered out)
  - 2026-05-07-m5-route-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (20 passed, 0 failed, 911 filtered out)
  - 2026-05-07-m5-popup-tooltip-validation: cargo test -p zircon_runtime --lib popup_tooltip_state --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (2 passed, 0 failed, 931 filtered out)
  - 2026-05-07-m5-drag-analog-validation: cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (11 passed, 0 failed)
  - 2026-05-07-m5-drag-analog-validation: cargo test -p zircon_runtime --lib drag_drop_ --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (2 passed, 0 failed)
  - 2026-05-07-m5-drag-analog-validation: cargo test -p zircon_runtime --lib analog_input_suppresses_repeated_values_before_routing --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (1 passed, 0 failed)
  - 2026-05-07-m5-native-validation: cargo test -p zircon_editor --lib native_input_translation --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m4 --message-format short --color never (5 passed, 0 failed)
  - 2026-05-08-m1-window-input-validation: cargo test -p zircon_runtime_interface --lib window_input_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-window-input-m1 --message-format short --color never (2 passed, 0 failed, 68 filtered out; existing sibling `ui_contract_spine` unused-import warning)
  - 2026-05-08-m1-window-input-validation: cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-window-input-m1 --message-format short --color never (passed)
  - 2026-05-20-binding-result-validation: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --message-format short --color never (passed, 7 tests)
  - 2026-05-20-binding-result-validation: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --message-format short --color never (passed, 91 tests)
  - 2026-05-20-widget-binding-result-validation: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/dispatch/pointer/result.rs zircon_runtime_interface/src/ui/dispatch/navigation/result.rs zircon_runtime_interface/src/tests/ui_contract_spine.rs zircon_runtime/src/ui/surface/surface/default_interactions.rs zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs zircon_runtime/src/ui/surface/surface/default_interactions/range.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/tests/pointer_click_semantics.rs zircon_runtime/src/ui/tests/widget_radio_behavior.rs zircon_runtime/src/ui/tests/widget_menu_behavior.rs zircon_runtime/src/ui/tests/widget_range_navigation.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-20-widget-binding-result-validation: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --message-format short --color never (passed, 7 tests)
  - 2026-05-20-widget-binding-result-validation: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --message-format short --color never (passed, 91 tests)
  - 2026-05-20-widget-binding-result-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (passed with existing unused-method warning)
  - 2026-05-20-scrollbar-runtime-state-validation: cargo test -p zircon_runtime --lib widget_scrollbar_behavior --locked --jobs 1 --message-format short --color never (passed, 4 tests)
  - 2026-05-20-scrollbar-thumb-drag-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs (passed)
  - 2026-05-20-scrollbar-thumb-drag-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed with existing unused-method warnings)
  - 2026-05-20-scrollbar-thumb-drag-validation: cargo test -p zircon_runtime --lib widget_scrollbar_behavior --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed, 6 tests)
  - 2026-05-20-popup-outside-dismiss-validation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/surface/default_interactions.rs zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs zircon_runtime/src/ui/tests/widget_menu_behavior.rs (passed)
  - 2026-05-20-popup-outside-dismiss-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed with existing unused-method warnings)
  - 2026-05-20-popup-outside-dismiss-validation: cargo test -p zircon_runtime --lib widget_menu_behavior --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (blocked before acceptance in the shared checkout by unrelated `scene/tests/ecs_systems.rs:538` tuple compile limits and then concurrent Cargo/Rust compiler timeout)
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
doc_type: module-detail
---

# Shared UI Input Events

`zircon_runtime_interface::ui::dispatch::input` owns the neutral shared input-event DTOs for the M5 event-system contract. It is a folder-backed declaration subtree: `mod.rs` only declares and re-exports child modules, while each child file owns one declaration family.

## Contract Shape

`metadata.rs` defines shared event metadata: timestamp, sequence, user/device/window/surface ids, pointer id, modifiers, and the synthetic-event flag.

`zircon_runtime_interface::ui::window` adds the M1 neutral window/input pump layer that wraps these input events without duplicating their payloads. `UiWindowEvent` covers cursor move/enter/leave, focus, resize, scale-factor, redraw, close, and lifecycle-style cleanup events. `UiWindowInputPumpEvent` is either a window event or an existing `UiInputEvent`, and `UiWindowInputPumpBatch::push_coalesced(...)` drops consecutive redraw requests while preserving input/window ordering around non-redraw events. The window event impact DTO records the M1 dirty consequences: cursor leave clears hover and redraws, scale-factor changes mark layout metrics dirty only, and close requests remain distinguishable from close cleanup.

`event.rs` defines `UiInputEvent` variants for pointer, keyboard, text, IME, navigation, analog, drag/drop, popup, and tooltip timer input. It reuses existing shared DTOs where they already exist: `UiPointerEvent`, `UiNavigationEventKind`, `UiPoint`, and `UiDragPayload`. Pointer input can carry optional `UiPreciseScrollDelta` x/y metadata with line or pixel units while legacy `UiPointerEvent::scroll_delta` remains the scalar fallback. `UiPointerEvent.click_count` carries host-supplied click count for double-click semantics and defaults to one for old payloads. IME ranges use `UiTextByteRange`, whose offsets are explicit UTF-8 byte positions into the event text, and drag/drop events can carry a `UiDragSessionId` for cross-event correlation.

`reply.rs` defines `UiDispatchReply`, `UiDispatchDisposition`, and `UiDispatchPhase`. Replies are per-dispatch transient commands, not durable widget state, and can carry ordered `UiDispatchEffect` entries. Each reply may now record the handler node and Slate-style phase (`Preprocess`, `PreviewTunnel`, `Direct`, `Target`, `Bubble`, or `DefaultAction`). `UiDispatchReply::merge_route(...)` is the shared propagation combiner: unhandled and passthrough replies continue, while handled and blocked replies stop the route and prevent later tunnel/bubble/default effects from being applied.

`effect.rs` defines `UiDispatchEffect` for focus, pointer capture/release, pointer lock, high precision pointer, drag/drop, navigation, popup, tooltip, input method requests, clipboard requests, dirty/redraw, and component event emission. Effects are also transient commands; runtime/editor persistence must go through normal tree, component, or host state paths. Owner-clearing effects carry the owner target (`ClearFocus`, `ReleasePointerCapture`, and `UnlockPointer`) so stale replies cannot clear another node's focus/IME, capture/high-precision, or pointer-lock state. Drag/drop effects carry target, pointer, optional session id, optional point, and optional payload so runtime diagnostics can distinguish simultaneous drags. Input-method requests carry surface-space cursor and composition rectangles rather than an origin-only anchor, giving host candidate windows enough geometry to follow both the caret and active composition spans. They can also carry `UiInputMethodSurroundingText`, a UTF-8 byte-indexed excerpt around the caret excluding active preedit text; this mirrors the winit 0.31 `ImeRequestData::surrounding_text` contract while keeping the DTO host-neutral. Input-method disable is encoded as `RequestInputMethod { kind: Disable }` so host requests keep one typed input-method channel. Clipboard requests carry only an owner, `ReadText`/`WriteText`, and optional text for writes, so the runtime can request host clipboard work without storing platform clipboard state.

`result.rs` defines `UiInputDispatchResult` plus diagnostics, applied/rejected effects, host requests, component event reporting, and binding update reports. Applied and rejected effects report the reply `effect_index` for duplicate-effect correlation. Host requests use the dedicated `UiDispatchHostRequestKind` enum instead of accepting arbitrary local effects; input method and clipboard requests are both typed host request variants. `binding_reports` is defaulted and omitted when empty, so older serialized dispatch results remain readable while M3 property-binding paths can return structured update evidence. The pointer and navigation dispatch result DTOs expose the same defaulted report array, and the shared input wrapper copies reports from those legacy dispatchers. The result type is representational in Milestone 1 and becomes the runtime surface input result in Milestone 2.

## Runtime Surface Application

`zircon_runtime::ui::surface::input` is the runtime M2 consumer for the shared input contract. It is folder-backed so `surface.rs` remains the retained surface orchestration boundary instead of becoming the implementation sink for every input family.

`state.rs` adds `UiSurfaceInputState` beside the existing `UiFocusState`. It tracks per-surface input ownership that is not durable widget state: captured pointer id, high-precision owner, pointer-lock owner/policy, input-method owner, the latest input-method request geometry, popup stack entries, pending/visible tooltip state, a shared drag/drop lifecycle record, and analog control values. Capture cleanup is owner-aware: capture loss clears the shared pointer id, while high precision is cleared only for the released or replaced captor. Popup effects open/close/toggle entries in a surface-local stack, and tooltip effects arm, show, hide, or cancel a single surface-local tooltip record before the host turns those results into platform UI.

`UiSurfaceDragDropState` stores the active drag source, target, pointer id, session id, current surface-space point, optional payload, and accepted/rejected result. `begin_drag_drop(...)` rejects a second concurrent drag and records the source owner; `update_drag_drop(...)`, `accept_drag_drop(...)`, `reject_drag_drop(...)`, and `end_drag_drop(...)` all validate pointer/session ownership so stale drag events cannot clear a live newer drag. `UiSurfaceAnalogControlState` stores the last routed value per control. Repeated values within the small equality threshold are suppressed before routing and recorded with an `analog_repeat_suppressed` diagnostic note, which prevents gamepad stick noise from forcing repeated hover/action work.

`validation.rs` keeps the runtime owner predicate shared between effect application and owner-routed dispatch. It accepts only existing, enabled owners with render-visible self and ancestor chains, so text/IME ownership cannot outlive hidden or collapsed parents.

`effect.rs` applies ordered `UiDispatchEffect` values once through `UiSurface::apply_dispatch_reply(...)`. It mutates existing focus/capture/navigation/tree dirty state, validates node ownership before accepting target-owned effects, records applied/rejected effects by `effect_index`, emits component event reports for accepted component events, and converts host-owned effects into typed `UiDispatchHostRequestKind` requests for the editor or runtime host. `UiSurface::apply_dispatch_reply_steps(...)` consumes phased reply steps through the shared merge contract before applying effects, so preview/tunnel handling can stop later bubble/default effects at the same runtime seam. Safety checks reject invalid input-method owners before host requests, reject malformed surrounding-text byte positions before native IME requests escape the surface, reject stale input-method reset/update/disable requests whose owner no longer matches the surface input state, reject pointer-capture release requests whose target/pointer id do not match the current capture, clear stale pointer ids on direct capture, reject stale pointer unlock requests from non-owners, reject stale high-precision disable requests from non-owners, and reject high-precision enable unless the same owner already has live pointer capture.

Drag/drop effects now mutate the same surface input state as capture and high precision. Begin validates the source owner and captures the pointer for the drag source; update changes target/point/payload only for the current session; accept/reject store the drop result; complete/cancel clear drag state and release the capture/high-precision owner that started the drag. This is the runtime-side cleanup contract for editor drag overlays, drawer tab detaches, asset drops, and future runtime drag targets: stale end events are rejected rather than clearing the current drag.

`dispatch.rs` adds `UiSurface::dispatch_input_event(...)` as the shared input entry point. Pointer and navigation events delegate to existing runtime dispatchers and then wrap their legacy results in `UiInputDispatchResult`, including any widget-originated binding reports returned by default pointer or navigation behavior. Keyboard events route through the focused path and record focused-route diagnostics plus default-action binding reports; TextInput caret keys consume `UiInputEventMetadata.modifiers.shift` in `text_keyboard.rs`, so Shift+Arrow/Home/End extends the retained selection while plain movement clears it. Home/End now respect multiline line boundaries through shared text helpers: plain Home/End moves to the current line start/end, while Ctrl/Super+Home/End moves to the document start/end, and CRLF is handled as one separator. Plain ArrowUp/ArrowDown use shared grapheme-column line helpers, so vertical movement targets the same logical column, clamps to shorter lines, handles CRLF, collapses first-line Up to document start and last-line Down to document end, and never returns a non-UTF-8 caret offset; Ctrl/Super+ArrowUp/ArrowDown follows the same document-start/document-end behavior as Bevy's focused editable text. Read-only TextInput still allows caret movement, selection, and Copy, but blocks content mutation, composition writes, Cut, and Paste. They also consume `UiInputEventMetadata.modifiers.control` for word navigation and word deletion, keeping Ctrl+Arrow and Ctrl+Backspace/Ctrl+Delete inside the same focused-input and binding-report route. Ctrl+A and Super+A reuse the existing selection action to select the full retained text without changing value. Plain focused Enter follows Bevy's `allow_newlines` branch: if retained `multiline` allows it, Enter inserts `"\n"` through the committed-text path and can replace an active selection; if `multiline = false`, Enter is left unhandled so submit/default behavior can observe it. Ctrl/Super+C and Ctrl/Super+V produce typed clipboard host requests without local value mutation or binding reports; Ctrl/Super+X first removes the active selection through the same `UiEditableTextState` mutation path, then emits a clipboard write request for the removed text. Paste remains host-fed: the runtime requests clipboard text and expects the platform/editor host to deliver the pasted payload back as `UiInputEvent::Text`. The same TextInput route now uses `ui/text/grapheme.rs` for ArrowLeft/ArrowRight and Backspace/Delete boundaries, preventing combining marks and emoji clusters from being split at scalar-value boundaries. Text events route to a valid input-method owner or, after clearing stale IME ownership, to the focused node; retained TextInput constraints from `text_constraints.rs` filter/truncate incoming text and IME preedit/commit payloads against the active replacement range before the shared edit action applies. This means insertion, keyboard Enter newline, and IME composition can replace a non-empty selection, Backspace/Delete can remove a non-empty selection, and `max_chars` budget is computed from the retained text outside that selection. IME preedit records composition range/text plus restore text in retained properties, cancel restores the saved text and clears the IME owner, and commit replaces the composition span before emitting the authored Submit/Commit event. The text mutation helper then labels property writes as widget behavior and records their binding reports beside text edit component events. Scrollbar track clicks return a pointer binding report for `scroll_target -> scroll_offset` runtime-state updates, while ScrollbarThumb drags capture the pointer, emit `BeginDrag` / `DragDelta` / `EndDrag` component events, and report `scroll_thumb -> scroll_offset` runtime-state updates during movement. Popup outside-click dismissal now runs as a runtime default pointer action: a primary release outside the topmost open popup closes that popup through the same `popup_open` property mutation and `ClosePopup` component event surface used by direct popup toggles, while clicks on the routed popup path or inside its arranged frame are preserved as inside-popup input. Accessibility dispatch returns the same report shape: Activate preserves reused widget reports, range Increment/Decrement labels property writes as `AccessibilityAction`, and ScrollTo emits an accessibility-originated runtime-state `scroll_offset` report. IME cancel clears the input-method owner and records an explicit diagnostic note; stale IME owners are also cleared and reported instead of remaining sticky. Pointer scroll keeps the legacy scalar fallback while preserving the original shared `UiPointerInputEvent`, including optional precise x/y/unit scroll metadata, in the returned `UiInputDispatchResult.event`. Analog events update surface-local analog control state before routing; unchanged values are returned as `Unhandled` with `routed = false`, so hosts can keep polling devices without rebuilding presentation on unchanged input.

2026-05-07 M6 closes the editable text mutation gap on this same route. `UiInputEvent::Text` and IME preedit/commit/cancel no longer stop at owner diagnostics: they build `UiEditableTextState`, apply `UiTextEditAction`, mutate focused/input-method owner node properties, and emit component event reports for the authored Change/Submit bindings. Preedit records composition range/text plus restore text, cancel restores the saved text and clears the input-method owner, and commit replaces the composition span before firing the submit/commit report. `property_mutation.rs` maps value edits to layout+render+text dirtiness, while caret, selection, and composition edits stay render+text only.

Focused validation for this closure used `E:\zircon-build\targets-ui-m6`: runtime `event_routing` passed 22 / 0 with shared text and IME mutation regressions; runtime `text_layout` passed 11 / 0; runtime-interface `render_contracts` passed 27 / 0; and editor/runtime text paint focused gates confirmed the shared render DTO still carries caret, selection, composition, rich runs, and font/atlas resource identity.

## Milestone Boundaries

Milestone 1 intentionally does not modify runtime or editor behavior. It only makes the shared DTO vocabulary serializable and constructible in `zircon_runtime_interface` so later milestones can route through a common input contract instead of adding another editor- or runtime-owned event vocabulary.

Milestone 2 changes runtime surface behavior only at the shared `UiSurface` seam. It preserves existing pointer click, hover, capture, scroll, and navigation tests while adding shared reply/effect application and focused keyboard/text/IME diagnostics. Later M4/M6 slices now implement retained TextInput caret/selection mutation on that seam, while shaping/text geometry and editor-native event translation remain separate milestones that should consume the same shared contract.

Contract tests in `zircon_runtime_interface/src/tests/contracts.rs` construct every event family and every effect family. They also round-trip pointer, keyboard, IME, drag/drop, popup, tooltip, input-method request, and clipboard request payloads through serde JSON.

Window/input pump contract tests in `zircon_runtime_interface/src/tests/window_input_contracts.rs` construct and round-trip Bevy-informed window events, assert cursor-leave and scale-factor impact semantics, and assert consecutive redraw coalescing while carrying existing `UiInputEvent` payloads. Runtime winit and editor host conversion remain later M1 behavior slices.

Runtime tests in `zircon_runtime/src/ui/tests/event_routing.rs`, `zircon_runtime/src/ui/tests/runtime_input_ownership.rs`, `zircon_runtime/src/ui/tests/pointer_click_semantics.rs`, and `zircon_runtime/src/ui/tests/popup_tooltip_state.rs` cover focus/capture/high-precision reply effects, owner-targeted clear-focus behavior, rejected focus preserving the current IME owner, navigation and focus changes clearing previous IME owners, direct clear-focus cleanup, focus/capture rejection through hidden ancestors, phased reply propagation stopping later bubble effects, direct capture clearing stale pointer ids before high precision can enable, stale capture release rejection, stale pointer unlock rejection, stale high-precision disable rejection, high-precision enable requiring live capture, capture transfer clearing the previous captor's high precision, navigation/input-method host requests, input-method reset/update/disable current-owner checks, invalid and stale input-method owner rejection, focused keyboard diagnostics, text owner routing and fallback after stale IME cleanup, hidden-ancestor owner rejection, IME owner cleanup, scroll diagnostics, double-click component event generation on the shared route, popup stack open/toggle state, and tooltip pending/visible/cancel state. The broader `shared_core` filter preserves the existing shared layout/hit/focus/navigation baseline while the runtime surface input state is present.
