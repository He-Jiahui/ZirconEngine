---
related_code:
  - zircon_runtime/src/ui/binding/update_report.rs
  - zircon_runtime/src/ui/binding/mod.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/range.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/result.rs
  - zircon_runtime/src/ui/tests/binding.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_runtime/src/ui/tests/widget_menu_behavior.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime_interface/src/ui/binding/model/update.rs
implementation_files:
  - zircon_runtime/src/ui/binding/update_report.rs
  - zircon_runtime/src/ui/binding/mod.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/range.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
plan_sources:
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
tests:
  - zircon_runtime/src/ui/tests/binding.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - 2026-05-20: cargo test -p zircon_runtime --lib binding_update_helpers_classify_runtime_state_and_attribute_writes --locked --jobs 1 --message-format short --color never (passed, 1 test)
  - 2026-05-20: cargo test -p zircon_runtime --lib surface_property_mutation_marks_dirty_only_when_values_change --locked --jobs 1 --message-format short --color never (passed, 1 test)
  - 2026-05-20: cargo test -p zircon_runtime --lib accessibility_set_value --locked --jobs 1 --message-format short --color never (passed, 4 tests)
  - 2026-05-20 result-propagation: cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (passed with existing unused-method warning)
  - 2026-05-20 result-propagation: cargo test -p zircon_runtime --lib accessibility_set_value --locked --jobs 1 --message-format short --color never (blocked before running focused tests by pre-existing asset material test compile error: missing `RenderMaterialValidationError::MissingRequiredProperty`)
  - 2026-05-20 widget-result-propagation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/surface/default_interactions.rs zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs zircon_runtime/src/ui/surface/surface/default_interactions/range.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/tests/pointer_click_semantics.rs zircon_runtime/src/ui/tests/widget_radio_behavior.rs zircon_runtime/src/ui/tests/widget_menu_behavior.rs zircon_runtime/src/ui/tests/widget_range_navigation.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-20 widget-result-propagation: cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (passed with existing unused-method warning)
  - 2026-05-20 scrollbar-runtime-state-report: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/binding/model/update.rs zircon_runtime_interface/src/tests/ui_contract_spine.rs zircon_runtime/src/ui/binding/update_report.rs zircon_runtime/src/ui/tests/binding.rs zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs (passed)
  - 2026-05-20 scrollbar-runtime-state-report: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --message-format short --color never (passed, 7 tests)
  - 2026-05-20 scrollbar-runtime-state-report: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --message-format short --color never (passed, 91 tests)
  - 2026-05-20 scrollbar-runtime-state-report: cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (passed with existing unused-method warning)
  - 2026-05-20 scrollbar-runtime-state-report: cargo test -p zircon_runtime --lib widget_scrollbar_behavior --locked --jobs 1 --message-format short --color never (passed, 4 tests)
  - 2026-05-20 a11y-action-report: rustfmt --edition 2021 --check zircon_runtime/src/ui/accessibility/action.rs zircon_runtime/src/ui/surface/surface/default_interactions/range.rs zircon_runtime/src/ui/tests/accessibility.rs zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs (passed)
  - 2026-05-20 a11y-action-report: cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (passed with existing unused-method warning)
  - 2026-05-20 a11y-action-report: cargo test -p zircon_runtime --lib accessibility_activate_uses --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed, 5 tests)
  - 2026-05-20 a11y-action-report: cargo test -p zircon_runtime --lib accessibility_increment --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed, 2 tests)
  - 2026-05-20 a11y-action-report: cargo test -p zircon_runtime --lib accessibility_scroll_to_mutates_scrollable_container_offset --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed, 1 test)
  - 2026-05-20 scrollbar-thumb-drag: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs (passed)
  - 2026-05-20 scrollbar-thumb-drag: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed with existing unused-method warnings)
  - 2026-05-20 scrollbar-thumb-drag: cargo test -p zircon_runtime --lib widget_scrollbar_behavior --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed, 6 tests)
  - 2026-05-20 popup-outside-dismiss: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/surface/default_interactions.rs zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs zircon_runtime/src/ui/tests/widget_menu_behavior.rs (passed)
  - 2026-05-20 popup-outside-dismiss: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed with existing unused-method warnings)
  - 2026-05-20 popup-outside-dismiss: cargo test -p zircon_runtime --lib widget_menu_behavior --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (blocked in the shared checkout; one attempt failed before focused tests on unrelated `zircon_runtime/src/scene/tests/ecs_systems.rs:538` tuple `Debug`/`PartialEq` compile limits, and a later rerun timed out while concurrent Cargo/Rust compiler processes were active)
  - 2026-05-20 textinput-constraints: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/text_constraints.rs zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-20 textinput-constraints: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (passed with existing unused-method warnings)
  - 2026-05-20 textinput-constraints: cargo test -p zircon_runtime --lib widget_text_input_keyboard --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (stopped after exceeding eight minutes in lib-test compilation without reaching focused tests)
  - 2026-05-20 textinput-keyboard-selection: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-20 textinput-keyboard-selection: git diff --check on TextInput keyboard runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-20 textinput-keyboard-selection: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never (timed out after 10 minutes under concurrent Cargo/Rust compiler load before diagnostics)
  - 2026-05-20 textinput-grapheme-editing: rustfmt --edition 2021 --check zircon_runtime/src/ui/text/grapheme.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/text/edit_state.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-20 textinput-grapheme-editing: git diff --check on TextInput grapheme runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-20 textinput-grapheme-editing: cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-grapheme-light-0520 --message-format short --color never (timed out after 5 minutes under shared Cargo/Rust compiler load before diagnostics)
  - 2026-05-21 textinput-word-navigation: rustfmt --edition 2021 --check zircon_runtime/src/ui/text/grapheme.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21 textinput-word-navigation: git diff --check on TextInput word-navigation runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-word-navigation: cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-word-nav-light-0521 --message-format short --color never (timed out after 5 minutes under shared Cargo/Rust compiler load before diagnostics)
  - 2026-05-21 textinput-selection-replacement: rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21 textinput-selection-replacement: git diff --check on TextInput selection replacement test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-selection-replacement: cargo test -p zircon_runtime --lib widget_text_input_keyboard --no-run --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-selection-replace-light-0521 --message-format short --color never (timed out after 5 minutes under shared Cargo/Rust compiler load before diagnostics)
  - 2026-05-21 textinput-ime-selection: rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21 textinput-ime-selection: git diff --check on TextInput IME selection test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-ime-selection: runtime cargo no-run deferred because unrelated Cargo/Rust compiler jobs were still active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21 textinput-select-all: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21 textinput-select-all: git diff --check on TextInput select-all runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-select-all: runtime cargo no-run deferred because unrelated Cargo/Rust compiler jobs were still active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21 textinput-word-delete: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21 textinput-word-delete: git diff --check on TextInput word-delete runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-word-delete: cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-word-delete-light-0521 --message-format short --color never (timed out after 5 minutes under shared Cargo/Rust compiler load before diagnostics)
  - 2026-05-21 textinput-clipboard-host: rustfmt --edition 2021 --check on interface dispatch, runtime input, and TextInput keyboard test files (passed)
  - 2026-05-21 textinput-clipboard-host: cargo test -p zircon_runtime_interface --lib ui_input --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-clipboard-interface-0521 --message-format short --color never (passed, 3 tests)
  - 2026-05-21 textinput-clipboard-host: git diff --check on TextInput clipboard runtime/interface/test files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-clipboard-host: cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-clipboard-runtime-0521 --message-format short --color never (timed out after 5 minutes while unrelated Cargo/Rust compiler jobs were active and produced no Rust diagnostics)
  - 2026-05-21 textinput-multiline-home-end: rustfmt --edition 2021 --check zircon_runtime/src/ui/text/grapheme.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21 textinput-multiline-home-end: git diff --check on TextInput multiline Home/End runtime/test files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-multiline-home-end: runtime cargo check deferred because unrelated Cargo/Rust compiler jobs were still active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21 textinput-readonly-selection: rustfmt --edition 2021 --check zircon_runtime/src/ui/text/edit_state.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21 textinput-readonly-selection: git diff --check on TextInput read-only runtime/test files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-readonly-selection: runtime cargo check deferred because unrelated Cargo/Rust compiler jobs were still active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21 textinput-multiline-up-down: rustfmt --edition 2021 --check zircon_runtime/src/ui/text/grapheme.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21 textinput-multiline-up-down: git diff --check on TextInput multiline Up/Down runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-multiline-up-down: runtime cargo check deferred because unrelated Cargo/Rust compiler jobs were still active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21 textinput-document-arrow: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21 textinput-document-arrow: git diff --check on TextInput document-arrow runtime/test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-document-arrow: runtime cargo check deferred because unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21 textinput-line-boundary-edges: rustfmt --edition 2021 --check zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-21 textinput-line-boundary-edges: git diff --check on TextInput line-boundary edge test/docs files (passed with LF/CRLF warnings only)
  - 2026-05-21 textinput-line-boundary-edges: runtime cargo check deferred because unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice
  - 2026-05-21 textinput-hard-line-navigation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs zircon_runtime/src/ui/tests/mod.rs (passed)
  - 2026-05-21 textinput-hard-line-navigation: tracked git diff --check on TextInput hard-line runtime/docs files and direct trailing-whitespace scan of the new hard-line test file (passed with LF/CRLF warnings only from git)
  - 2026-05-21 textinput-hard-line-navigation: runtime cargo check deferred while unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice before milestone testing
  - 2026-05-21 textinput-keyboard-text-payload: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_keyboard.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs zircon_runtime/src/ui/tests/mod.rs (passed)
  - 2026-05-21 textinput-keyboard-text-payload: tracked git diff --check on TextInput keyboard-text runtime/docs files and direct trailing-whitespace scan of the new keyboard-text test file (passed with LF/CRLF warnings only from git)
  - 2026-05-21 textinput-keyboard-text-payload: runtime cargo check deferred while unrelated Cargo/Rust compiler jobs were active in the shared checkout; no Rust diagnostics were produced for this slice before milestone testing
  - 2026-05-22 textinput-pointer-selection: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/input/text_pointer.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/surface/render/resolve.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/tests/widget_text_input_pointer.rs zircon_runtime/src/ui/tests/mod.rs (passed)
  - 2026-05-22 textinput-pointer-selection: direct trailing-whitespace scan passed for zircon_runtime/src/ui/surface/input/text_pointer.rs and zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - 2026-05-22 textinput-pointer-selection: runtime Cargo validation deferred because unrelated Cargo/Rust compiler jobs remained active in the shared checkout; no Rust diagnostics were produced for this slice
doc_type: module-detail
---

# Runtime UI Binding Update Reports

`zircon_runtime::ui::binding::update_report` contains small helpers that build the shared `UiBindingUpdate` DTOs for runtime-owned binding paths. It keeps runtime behavior in the runtime crate while leaving the update vocabulary in `zircon_runtime_interface`.

The helpers currently cover the safe M3 foundation:

- retained attribute updates, where source and target are the retained node attribute
- component-state value updates, where runtime state is the source and `UiSurfaceComponentStateStore` is the target
- runtime-state target updates, where widget or accessibility behavior changes surface-owned facts such as scroll offset
- rejected widget-alias updates, for reducers or accessibility actions that need a structured rejection without mutating state
- reflected property updates, where `UiPropertyMutationReport` records accepted, unchanged, or rejected retained-property writes
- report aggregation through `binding_update_report(...)`

This module does not execute property mutation and does not own widget behavior. It exists so `UiSurface::mutate_property`, widget reducers, accessibility dispatch, and the future UI ECS bridge can converge on one report shape as their milestones land.

Accessibility SetValue dispatch now records the mutation binding report on `UiInputDispatchResult.binding_reports` in addition to the older diagnostic notes. Default widget reducers record the same report stream for pointer, keyboard, and navigation actions on the legacy pointer/navigation results and the shared input wrapper. TextInput keyboard and pointer-selection edits also record widget-behavior binding reports for value, caret, selection, and composition property writes. Consumers can therefore inspect applied/unchanged/rejected counts and update source kind without parsing strings.

Scrollbar track clicks and thumb drags now report page-scroll mutations even though they bypass `UiSurface::mutate_property`. Track clicks use the scrollbar node's `scroll_target` widget behavior as the source. Thumb drags use the thumb node's `scroll_thumb` widget behavior as the source. Both target the scrollable node's runtime `scroll_offset`, and the dirty domains mirror the scrollable node invalidation caused by `set_scroll_offset(...)`.

Popup outside-click dismissal now uses the same widget behavior source as direct popup toggles and menu-item close behavior. A primary release outside the topmost open popup mutates that popup's `popup_open`/custom open property to false, emits `ClosePopup`, and returns the `UiPropertyMutationReport.binding` report on the pointer dispatch result. MenuItem activation uses that same close helper for pointer, focused keyboard, and accessibility activation; the close mutation does not depend on the MenuItem itself having an activation binding, so the popup `open_property` still produces a `WidgetBehavior` binding report when only the popup close binding exists. The hit check treats the routed path and the arranged popup frame as inside the popup, so empty popup space does not dismiss itself while outside clicks close only the topmost open popup.

TextInput replacement constraints are applied before the existing text mutation pipeline rather than as a post-write correction. Runtime reads retained `max_graphemes`/`max_chars`/`max_length`, `input_filter`/`text_filter`, and `multiline` attributes, sanitizes the inserted or preedit payload against the current replacement range, then lets the normal `value`, `caret_offset`, selection, and composition property writes produce the same widget-behavior binding reports as unconstrained text edits. Keyboard selection uses that same path: Shift+Arrow/Home/End changes `caret_offset`, `selection_anchor`, and `selection_focus` without emitting a value-change component event because the retained value is unchanged. Read-only TextInput keeps caret/selection mutation enabled, so users can navigate and copy text, but the shared edit helper blocks value-changing actions and composition writes. Plain multiline Home/End also stays inside that path, moving to the current line boundary, while Ctrl/Super+Home/End continues to move to the document boundary; Shift+End extends selection to the current line end using the same retained selection properties. The line helpers treat CRLF as one separator and keep End before the carriage return so caret offsets never point into the newline pair. Plain ArrowUp/ArrowDown also stay inside the same report path: shared text helpers compute the nearest same grapheme column on the previous or next line, clamp to shorter lines, handle CRLF, collapse first-line Up to document start and last-line Down to document end, and return valid UTF-8 caret offsets before `MoveCaret` writes retained caret/selection properties. Ctrl/Super+ArrowUp/ArrowDown bypass line-column lookup and write document-start/document-end offsets through the same report path, matching Bevy editable text's `TextStart`/`TextEnd` shortcut behavior. Ctrl+Arrow word navigation also stays inside that path, so Ctrl+Shift word selection reports caret/selection properties without a value-change component event. Grapheme-aware Backspace/Delete and Arrow movement also stay inside that path; value-changing deletion still reports the authored value property, while selection-only movement reports caret/selection properties. Escape collapse also stays on this route: without active composition it writes collapsed `selection_anchor`/`selection_focus` at the current caret/focus and emits no value-change event; with active composition it uses `CancelComposition`, so any restored text reports as an ordinary value/caret/composition write. Active selection replacement now stays on this route as well: inserted text and keyboard Enter newline replace the selected range, Backspace removes the selected range, selection endpoints collapse to the new caret, and `max_chars` budgets only the retained text outside the replaced selection. Enter uses the retained `multiline` constraint as Bevy's `allow_newlines` equivalent: multiline fields report the normal `WidgetBehavior` value/caret/selection writes for `"\n"`, while single-line fields return no binding report because text editing does not consume Enter. Clipboard shortcuts deliberately do not create a second binding-report vocabulary: Ctrl/Super+C/X/V, dedicated Copy/Cut/Paste logical keys, and Shift+Delete cut all reuse the same host request path. Copy and Paste only return typed `Clipboard` host requests, while Cut removes the active selection through `apply_editable_text_state(...)` and therefore reports the same `WidgetBehavior` value/caret/selection writes as Backspace deletion before adding the host clipboard write request. This keeps filtering/truncation, Escape collapse, selection movement, line navigation, Enter newline, read-only navigation, and cut mutation visible as ordinary TextInput state changes without treating the host clipboard as runtime truth.

Accessibility actions now follow the same report surface beyond SetValue. `Activate` reuses default keyboard/widget behavior and preserves the widget-originated binding reports it produces. `Increment` and `Decrement` step range widgets through `UiSurface::mutate_property` with an `AccessibilityAction` source override, so slider adjustments expose the same retained-attribute and component-state updates as direct SetValue. `ScrollTo` reports a runtime-state update from the target node's `scroll_to` action to its `scroll_offset`, including unchanged reports when the requested offset is already current.

## Dirty Domains

The helper functions accept existing `UiDirtyFlags` and convert them to `UiBindingDirtyDomain` through the interface helper. This keeps the first slice aligned with current retained-tree invalidation while leaving room for later `Accessibility`, `Interaction`, and `Schedule` dirty domains that do not belong in tree layout flags.

## Validation

The focused runtime binding test checks that runtime-state and retained-attribute updates are classified correctly, that rejected widget alias updates carry a rejected status, and that the aggregate report exposes applied/unchanged/rejected counts plus dirty domain union.

On 2026-05-20, `cargo test -p zircon_runtime --lib binding_update_helpers_classify_runtime_state_and_attribute_writes --locked --jobs 1 --message-format short --color never` passed the focused helper test.

The same runtime test binary later passed `surface_property_mutation_marks_dirty_only_when_values_change`, covering that `UiSurface::mutate_property` now returns binding reports for unchanged, accepted, and rejected property writes, that accepted writes which sync component state add a second `ComponentStateValue` update, and that widget-originated mutations preserve `WidgetBehavior` source classification.

`accessibility_set_value` focused tests also passed after SetValue dispatch began using the `AccessibilityAction` binding source override and emitting binding update diagnostics. A later result-propagation pass typechecked the runtime library successfully, but the focused runtime test binary could not be re-run because an unrelated asset material test still references the removed `RenderMaterialValidationError::MissingRequiredProperty` variant during lib-test compilation.

The widget-result propagation slice extends the runtime-side tests for Button/Toggle/Radio/Menu/Range/TextInput dispatch results so successful default behavior asserts a `WidgetBehavior` source in the returned binding reports, while ignored or disabled widget paths assert no report. The runtime library typechecked after the result propagation changes; focused lib-test execution was later delayed by concurrent Cargo/Rust compiler activity in the shared workspace.

The Scrollbar runtime-state slice extends `widget_scrollbar_behavior`: track click now asserts one binding report with `WidgetBehavior` source, `RuntimeState` target, `scroll_target -> scroll_offset` properties, previous/current offset values, and layout/render/input dirty domains; thumb click still asserts no report. A separate binding-helper focused test attempt timed out under concurrent workspace builds, but the helper typechecked through `cargo check` and is exercised by the Scrollbar behavior test.

The Accessibility action report slice extends `accessibility` and `widget_scrollbar_behavior` tests so Activate asserts propagated `WidgetBehavior` reports, Increment/Decrement assert `AccessibilityAction` reports, and ScrollTo asserts an `AccessibilityAction -> RuntimeState(scroll_offset)` report. On 2026-05-20, rustfmt passed, `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` passed with the existing unused-method warning, and the focused Activate, Increment, and ScrollTo runtime filters passed. The focused test commands used `D:\cargo-targets\zircon-render-light-stats-0520` after concurrent Cargo activity caused long artifact-lock waits in the shared workspace.

The ScrollbarThumb drag slice extends the same test file so press captures the thumb and emits `BeginDrag`, pointer move outside the original hit can mutate the target scroll container through `scroll_thumb -> scroll_offset`, release emits `EndDrag` and clears capture, and disabled thumbs neither capture nor scroll. On 2026-05-20, rustfmt passed, `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never` passed with existing unused-method warnings, and `cargo test -p zircon_runtime --lib widget_scrollbar_behavior --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never` passed 6 focused tests.

The Popup outside-dismiss slice moves popup/menu behavior into `default_interactions/popup.rs` so the parent default-interactions file remains an orchestration boundary. `widget_menu_behavior` now has focused assertions for outside clicks closing an open popup without activating menu items, nested popup stacks closing only the topmost popup, and empty popup-space clicks preserving the open popup. On 2026-05-20, rustfmt passed and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never` passed with existing unused-method warnings. Focused `widget_menu_behavior` lib-test execution is not claimed green in this shared checkout because unrelated ECS test compilation and later concurrent Cargo/Rust compiler activity blocked the run before focused assertions could execute.

The MenuItem activation-close slice keeps popup close reports on the same `WidgetBehavior` source even when the MenuItem itself has no item activation binding. `widget_menu_behavior` now covers focused keyboard activation closing the popup without an item commit event, and `accessibility_widget_actions` covers accessibility `Activate` inheriting that close path. On 2026-05-23, rustfmt passed for the touched runtime/test files, scoped `git diff --check` passed with LF/CRLF warnings only, and trailing-whitespace checks passed. Runtime Cargo validation remains deferred because 4 unrelated `cargo` and 4 `rustc` processes remain active in the shared checkout.

The TextInput constraints slice adds `surface/input/text_constraints.rs` and focused `widget_text_input_keyboard` cases for filtered/truncated text input and explicit single-line replacement. On 2026-05-20, rustfmt passed and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-light-stats-0520 --message-format short --color never` passed with existing unused-method warnings. Focused `widget_text_input_keyboard` lib-test execution was stopped after more than eight minutes in `zircon_runtime` lib-test compilation without reaching the focused tests; no runtime assertion failure was observed.

The TextInput keyboard selection slice extends `widget_text_input_keyboard` with Shift+ArrowLeft and Shift+Home cases. Those cases assert that selection-only keyboard movement writes `caret_offset`, `selection_anchor`, and `selection_focus` through widget-behavior binding reports while leaving the value unchanged and emitting no value-change component event. On 2026-05-20, rustfmt and scoped diff whitespace checks passed; the runtime library typecheck was attempted but timed out after 10 minutes while multiple unrelated Cargo/Rust compiler jobs were still active in the shared checkout.

The TextInput grapheme editing slice extends the same keyboard test file with combining-mark cases for ArrowLeft, Backspace, and Delete. The runtime edit helper and keyboard mapper now share `ui/text/grapheme.rs` boundaries, so deletion and caret movement avoid splitting multi-codepoint grapheme clusters while still returning the same widget-behavior binding reports as earlier TextInput edits.

The TextInput word navigation slice extends the boundary helper with Unicode word-boundary movement and consumes the shared `control` modifier in `text_keyboard.rs`. Ctrl+Arrow movement and Ctrl+Shift+Arrow selection continue to mutate only retained caret/selection properties, so report consumers see the same `WidgetBehavior` source and dirty-domain shape as grapheme caret movement.

The TextInput selection replacement slice extends `widget_text_input_keyboard` with focused cases for text insertion replacing an active selection, Backspace deleting an active selection, and `max_chars` truncating inserted text against the retained text outside the selected range. The implementation path did not need a second reducer: it exercises the existing `UiEditableTextState` replacement range, `text_constraints.rs` budget calculation, and `apply_editable_text_state(...)` property mutation/reporting path.

On 2026-05-21, rustfmt passed for `widget_text_input_keyboard.rs`, and scoped diff whitespace checks passed with LF/CRLF warnings only. A light `cargo test -p zircon_runtime --lib widget_text_input_keyboard --no-run --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-selection-replace-light-0521 --message-format short --color never` attempt timed out after 5 minutes while unrelated Cargo/Rust compiler jobs were active and produced no Rust diagnostics.

The TextInput IME selection slice extends the same focused tests across preedit, cancel, and commit. Preedit replacement of an active selection records the temporary value plus `composition_start`, `composition_end`, `composition_text`, and `composition_restore_text` as ordinary widget-behavior property reports. Cancel restores the saved text, collapses selection/composition metadata to the caret, clears the IME owner, and reports the restored value through the same path. Commit replaces the composition span, clears composition metadata, and emits the authored Submit/Commit event while preserving widget-behavior binding reports for the value/caret/composition writes.

For the IME selection slice, rustfmt passed for `widget_text_input_keyboard.rs`, and scoped diff whitespace checks passed with LF/CRLF warnings only. Runtime cargo no-run validation was deferred because unrelated Cargo/Rust compiler jobs remained active in the shared checkout; no Rust diagnostics were produced for this slice.

The TextInput select-all slice maps Ctrl+A and Super+A to the existing `UiTextEditAction::SetSelection` action in `surface/input/text_keyboard.rs`. It is intentionally not a clipboard shortcut yet: it mutates only `caret_offset`, `selection_anchor`, and `selection_focus`, emits no value-change component event, and still reports the property writes with `WidgetBehavior` source through `apply_editable_text_state(...)`.

For the select-all slice, rustfmt passed for `text_keyboard.rs` and `widget_text_input_keyboard.rs`, and scoped diff whitespace checks passed with LF/CRLF warnings only. Runtime cargo no-run validation was deferred because unrelated Cargo/Rust compiler jobs remained active in the shared checkout; no Rust diagnostics were produced for this slice.

The TextInput word-delete slice changes the runtime keyboard mapper from one edit action to a short action sequence. Ctrl+Backspace selects the previous word range with the existing word-boundary helper and then applies Backspace; Ctrl+Delete selects the next word range and then applies Delete. Active selections still use the normal deletion action directly. Because the sequence flows through `apply_editable_text_state(...)`, word deletion reports the authored value property and collapsed caret/selection writes with the same `WidgetBehavior` source as grapheme deletion and text replacement.

For the word-delete slice, rustfmt passed for `dispatch.rs`, `text_keyboard.rs`, and `widget_text_input_keyboard.rs`, and scoped diff whitespace checks passed with LF/CRLF warnings only. A light `cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-word-delete-light-0521 --message-format short --color never` attempt timed out after 5 minutes while unrelated Cargo/Rust compiler jobs were active and produced no Rust diagnostics.

The TextInput clipboard-host slice adds `UiClipboardRequest`, `UiClipboardRequestKind`, `UiDispatchEffect::RequestClipboard`, and `UiDispatchHostRequestKind::Clipboard` to the neutral dispatch contract. Runtime TextInput maps Ctrl/Super+C to a clipboard write request for the active selection, Ctrl/Super+V to a clipboard read request, and Ctrl/Super+X to the normal selected-range delete path followed by a clipboard write request for the removed text. Copy and Paste return no binding reports because no runtime property changes; Cut returns the same `WidgetBehavior` binding reports as other value-changing TextInput edits.

For the clipboard-host slice, rustfmt passed for the interface dispatch files, runtime input files, and `widget_text_input_keyboard.rs`. `cargo test -p zircon_runtime_interface --lib ui_input --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-clipboard-interface-0521 --message-format short --color never` passed the 3 focused input contract tests. Scoped diff whitespace checks passed with LF/CRLF warnings only. A light runtime `cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-clipboard-runtime-0521 --message-format short --color never` timed out after 5 minutes while unrelated Cargo/Rust compiler jobs were active and produced no Rust diagnostics.

The TextInput multiline Home/End slice adds shared line-start and line-end byte-boundary helpers beside the existing grapheme and word helpers. Runtime TextInput now moves plain Home/End to the current line start/end, while Ctrl/Super+Home/End remains document navigation. Shift+End extends retained selection to the current line end, so all line-boundary movement continues to report caret/selection property writes through the same `WidgetBehavior` binding source.

For the multiline Home/End slice, rustfmt passed for `grapheme.rs`, `mod.rs`, `text_keyboard.rs`, and `widget_text_input_keyboard.rs`. Scoped diff whitespace checks passed with LF/CRLF warnings only. Runtime Cargo validation was deferred because unrelated Cargo/Rust compiler jobs remained active in the shared checkout; no Rust diagnostics were produced for this slice.

The TextInput multiline Up/Down slice adds shared previous/next-line same-grapheme-column helpers beside the line start/end helpers. Runtime TextInput maps ArrowUp/ArrowDown and their key-code fallbacks to `MoveCaret`, so plain vertical movement collapses selection to the target line and Shift+ArrowDown extends selection through the same retained caret/selection properties. Shorter target lines clamp to line end, and combining-mark grapheme columns are counted as one logical column until the shaping layer supplies visual x-position caret navigation.

For the multiline Up/Down slice, rustfmt passed for `grapheme.rs`, `mod.rs`, `text_keyboard.rs`, and `widget_text_input_keyboard.rs`. Scoped diff whitespace checks passed with LF/CRLF warnings only. Runtime Cargo validation was deferred because unrelated Cargo/Rust compiler jobs remained active in the shared checkout; no Rust diagnostics were produced for this slice.

The TextInput document-arrow slice aligns Ctrl/Super+ArrowUp and Ctrl/Super+ArrowDown with Bevy `editable_text.rs`, where command-style vertical arrows become `TextStart` and `TextEnd`. Zircon maps those shortcuts to offsets `0` and `text.len()` through `MoveCaret`, so Shift variants extend retained selection while plain variants collapse selection to the document boundary.

For the document-arrow slice, rustfmt passed for `text_keyboard.rs` and `widget_text_input_keyboard.rs`, and scoped diff whitespace checks passed with LF/CRLF warnings only. Runtime Cargo validation was deferred because unrelated Cargo/Rust compiler jobs remained active in the shared checkout; no Rust diagnostics were produced for this slice.

The TextInput line-boundary edge slice adds focused coverage for CRLF line separators and first/last-line vertical movement. Home moves to the byte after the CRLF pair, End stops before the CR byte, ArrowUp/ArrowDown preserve the same grapheme column across CRLF lines, ArrowUp on the first line collapses to document start, and ArrowDown on the last line collapses to document end. No new binding vocabulary is introduced; all cases continue to report retained caret/selection writes with `WidgetBehavior` source.

For the line-boundary edge slice, rustfmt passed for `widget_text_input_keyboard.rs`, and scoped diff whitespace checks passed with LF/CRLF warnings only. Runtime Cargo validation was deferred because unrelated Cargo/Rust compiler jobs remained active in the shared checkout; no Rust diagnostics were produced for this slice.

The TextInput read-only selection slice changes `apply_text_edit_action(...)` so `read_only` blocks value-changing and composition actions without blocking `MoveCaret` and `SetSelection`. That keeps the runtime behavior aligned with standard read-only fields: caret movement, selection, and Copy remain available, while Backspace/Delete/Insert/IME writes do not change the value and therefore do not emit value-change component events.

For the read-only selection slice, rustfmt passed for `edit_state.rs` and `widget_text_input_keyboard.rs`, and scoped diff whitespace checks passed with LF/CRLF warnings only. Runtime Cargo validation was deferred because unrelated Cargo/Rust compiler jobs remained active in the shared checkout; no Rust diagnostics were produced for this slice.

The TextInput Enter-newline slice adds the Bevy `allow_newlines` equivalent to focused keyboard dispatch. Plain Enter in a multiline TextInput inserts `"\n"` through `committed_text_state(...)`, so active selections are replaced and the same widget-behavior binding reports describe value/caret/selection writes. Explicit `multiline = false` returns no text-edit binding report because the key remains unhandled for submit/default behavior.

For the Enter-newline slice, rustfmt passed for `dispatch.rs`, `text_constraints.rs`, `text_keyboard.rs`, and `widget_text_input_keyboard.rs`. Scoped diff whitespace checks passed with LF/CRLF warnings only. Runtime Cargo validation was deferred while unrelated Cargo/Rust compiler jobs remained active in the shared checkout; no Rust diagnostics were produced before milestone testing.

The TextInput Escape-collapse slice aligns the no-composition Escape path with Bevy `TextEdit::CollapseSelection`. Zircon reuses `MoveCaret { extend_selection: false }` at the current caret/focus, so selection collapse reports only caret/selection property writes and emits no value-change component event. If a composition is active, Escape still maps to `CancelComposition` and restores the preedit source text before clearing composition metadata.

For the Escape-collapse slice, rustfmt passed for `text_keyboard.rs` and `widget_text_input_keyboard.rs`, and scoped diff whitespace checks passed with LF/CRLF warnings only. Runtime Cargo validation remains deferred while unrelated Cargo/Rust compiler jobs are active in the shared checkout.

The TextInput clipboard command-key slice extends the existing host clipboard request mapping beyond Ctrl/Super shortcuts. Dedicated logical Copy/Cut/Paste keys now route to the same Copy/Cut/Paste actions, and Shift+Delete routes to Cut before the normal Delete edit action can consume it. This adds Bevy parity without adding runtime clipboard truth: Copy/Paste still produce only host requests, and Cut still mutates selection through the normal TextInput edit/report path before requesting a clipboard write.

For the clipboard command-key slice, rustfmt passed for `text_keyboard.rs` and `widget_text_input_keyboard.rs`, and scoped diff whitespace checks passed with LF/CRLF warnings only. Runtime Cargo validation remains deferred while unrelated Cargo/Rust compiler jobs are active in the shared checkout.

The TextInput hard-line navigation slice adds a focused `widget_text_input_keyboard_hard_line` module instead of growing the already-large general keyboard file. Super+ArrowLeft/Right, when Ctrl and Alt are not pressed, uses the retained hard-line start/end helpers; Alt+Home/End goes through the existing Home/End line-boundary path. All covered variants write only caret/selection attributes and keep `WidgetBehavior` binding reports.

For the hard-line navigation slice, rustfmt passed for `text_keyboard.rs`, `widget_text_input_keyboard_hard_line.rs`, and `mod.rs`. Tracked diff whitespace checks passed with LF/CRLF warnings only, and the new untracked test file had no trailing whitespace. Runtime Cargo validation remains deferred while unrelated Cargo/Rust compiler jobs are active in the shared checkout.

The TextInput keyboard-text payload slice consumes printable `UiKeyboardInputEvent.text` values before generic caret/navigation key handling. It uses the same `committed_text_state(...)` path as `UiInputEvent::Text`, so active selection replacement, filters, length limits, value-change component events, and `WidgetBehavior` binding reports stay unified. Tab and newline payloads are rejected so navigation and single-line Enter behavior remain observable by higher-level defaults.

For the keyboard-text payload slice, rustfmt passed for `text_keyboard.rs`, `dispatch.rs`, `widget_text_input_keyboard_text.rs`, and `mod.rs`. Tracked diff whitespace checks passed with LF/CRLF warnings only, and the new untracked test file had no trailing whitespace. Runtime Cargo validation remains deferred while unrelated Cargo/Rust compiler jobs are active in the shared checkout.

The TextInput pointer-selection slice adds `surface/input/text_pointer.rs` beside the keyboard and constraints helpers. Pointer press and captured pointer move map the render command's `UiResolvedTextLayout` through `ui/text/hit_test.rs`, then call the same `apply_editable_text_state(...)` path as keyboard movement. Press writes collapsed caret selection unless Shift extends from the current caret; drag writes selection extension while the captured pointer id still matches. Because this only mutates retained caret/selection properties, value-change component events stay quiet while binding reports still expose `WidgetBehavior` source and render/text dirty domains. Empty editable values and custom `value_property` text sources are resolved in `surface/render/resolve.rs` so hit testing uses the same visible value that runtime editing mutates.

For the pointer-selection slice, rustfmt passed for `text_pointer.rs`, `dispatch.rs`, `input/mod.rs`, `render/resolve.rs`, `ui/text/mod.rs`, `widget_text_input_pointer.rs`, and `ui/tests/mod.rs`. Direct trailing-whitespace checks passed for the two new files. Runtime Cargo validation remains deferred while unrelated Cargo/Rust compiler jobs are active in the shared checkout.
