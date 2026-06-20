---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/discovery.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/options.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/search.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/test_support.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/discovery.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/options.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/search.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - native keyboard subtree ownership scan
  - native keyboard target discovery/model/row subtree ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Native Keyboard

`native_keyboard.rs` is the retained-host popup keyboard entry. It should remain a small dispatch-facing module that declares the folder-backed subtree and re-exports the stable functions used by `window/text_input.rs` and test support.

## Child Modules

`native_keyboard/commands.rs` owns raw winit key translation into `WorkbenchPopupKeyboardCommand`. It maps only popup-navigation boundary keys: arrows, Home/End, Enter, and Escape.

`native_keyboard/dispatch.rs` owns keyboard command execution against the active popup. It advances hover rows, accepts command-palette/options/menu rows through `PaneSurfaceHostContext`, cancels the popup, clears hover state on cancel, and returns the damaged frame region.

`native_keyboard/target.rs` owns only the folder-backed popup target entry and stable re-exports. `target/discovery.rs` scans Workbench template nodes and selects the active popup. `target/options.rs` builds option rows and projected/dropdown option frames. `target/menu.rs` builds actionable menu rows. `target/selection.rs` resolves the current row from interaction state and creates `PopupKeyboardTarget`. `target/model.rs` owns `PopupKeyboardTarget`/`PopupKeyboardRow` plus next-row and text-search behavior. `target/search.rs` owns popup text query normalization and prefix matching.

`native_keyboard/tests.rs` owns module-local key translation regressions that previously lived inline in the root file.

## Boundary Rules

- Keep `native_keyboard.rs` limited to subtree declarations and stable retained-host imports.
- Keep raw key mapping in `commands.rs`; do not mix it with popup row geometry or callback dispatch.
- Keep callback invocation and damage calculation in `dispatch.rs`.
- Keep popup node scanning, option/menu row construction, current-row resolution, target model behavior, and text search in the `target/` child owners; do not re-grow `target.rs` beyond module wiring.
- Keep focused text editing in `window/text_input.rs`; popup keyboard is only the fallback/navigation layer around active Workbench popups.

## Validation Notes

The 2026-06-18 native keyboard subtree split reduced `native_keyboard.rs` to an 11-line entry. The production owners are `target.rs` at 267 lines, `dispatch.rs` at 101 lines, and `commands.rs` at 23 lines; `tests.rs` carries the moved 13-line regression body.

Evidence for this slice is formatting, a native keyboard subtree ownership scan, scoped diff whitespace checks, and a scoped `zircon_editor` library type check. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-20 native keyboard target split reduced `native_keyboard/target.rs` from 267 lines to an 8-line module wiring entry. `target/model.rs` is 70 lines and owns target/row DTOs plus next-row/text-search behavior; `discovery.rs` is 55 lines and owns active popup selection; `options.rs` is 72 lines and owns option row construction and row frame selection; `menu.rs` is 35 lines and owns actionable menu row construction; `selection.rs` is 54 lines and owns target assembly/current row resolution; `search.rs` is 12 lines and owns normalized query and prefix matching. The 2026-06-20 owner visibility sweep kept the target model/discovery API within `host_contract` and corrected popup bounds lookup through `template_geometry`. Validation now includes `cargo fmt -p zircon_editor --check` and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passes with existing warning noise only; full Cargo tests remain deferred per the user's instruction.
