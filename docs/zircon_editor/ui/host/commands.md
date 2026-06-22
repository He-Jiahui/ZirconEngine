---
related_code:
  - zircon_editor/src/ui/host/commands/mod.rs
  - zircon_editor/src/ui/host/commands/context.rs
  - zircon_editor/src/ui/host/commands/descriptor.rs
  - zircon_editor/src/ui/host/commands/key_chord.rs
  - zircon_editor/src/ui/host/commands/keymap.rs
  - zircon_editor/src/ui/host/commands/palette.rs
  - zircon_editor/src/ui/host/commands/registry.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/binding/core/payload.rs
  - zircon_editor/src/ui/binding/core/payload_codec.rs
  - zircon_editor/src/ui/binding/core/payload_constructors.rs
  - zircon_editor/src/ui/binding_dispatch/editor_event_normalization.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/command.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/command_palette.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/ui/host/editor_event_execution/dispatch.rs
  - zircon_editor/src/ui/workbench/reflection/transient_ui_state.rs
  - zircon_editor/src/ui/retained_host/event_bridge.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/src/ui/retained_host/app/native_keyboard_actions.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/command_palette.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/mod.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_items_for_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/reference_overlay_apply_tests.rs
  - zircon_editor/src/ui/retained_host/app/tests/mod.rs
  - zircon_editor/src/tests/ui/component_adapter.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/dispatcher.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/command_palette.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_editor/assets/ui/editor/keymap/default.keymap.toml
  - zircon_editor/assets/ui/editor/windows/workbench_window.v2.ui.toml
  - zircon_editor/src/ui/workbench/model/menu/default_menu_bar.rs
  - zircon_editor/src/ui/workbench/model/menu/mod.rs
  - zircon_editor/src/ui/workbench/event/menu_action_id.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/ui/workbench/model/menu_bar_model.rs
  - zircon_editor/src/ui/workbench/model/menu_model.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/attributes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/entries.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/entry.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/ids.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/options.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/parse.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/tests.rs
  - zircon_runtime/src/ui/component/state_reducer/command_palette.rs
implementation_files:
  - zircon_editor/src/ui/host/commands/mod.rs
  - zircon_editor/src/ui/host/commands/context.rs
  - zircon_editor/src/ui/host/commands/descriptor.rs
  - zircon_editor/src/ui/host/commands/key_chord.rs
  - zircon_editor/src/ui/host/commands/keymap.rs
  - zircon_editor/src/ui/host/commands/palette.rs
  - zircon_editor/src/ui/host/commands/registry.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/binding/core/payload.rs
  - zircon_editor/src/ui/binding/core/payload_codec.rs
  - zircon_editor/src/ui/binding/core/payload_constructors.rs
  - zircon_editor/src/ui/binding_dispatch/editor_event_normalization.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/command.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/command_palette.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/ui/host/editor_event_execution/dispatch.rs
  - zircon_editor/src/ui/workbench/reflection/transient_ui_state.rs
  - zircon_editor/src/ui/retained_host/event_bridge.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/src/ui/retained_host/app/native_keyboard_actions.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/command_palette.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/mod.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_items_for_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/reference_overlay_apply_tests.rs
  - zircon_editor/src/ui/retained_host/app/tests/mod.rs
  - zircon_editor/src/tests/ui/component_adapter.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/dispatcher.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/command_palette.rs
  - zircon_editor/assets/ui/editor/keymap/default.keymap.toml
  - zircon_editor/assets/ui/editor/windows/workbench_window.v2.ui.toml
  - zircon_editor/src/ui/workbench/model/menu/default_menu_bar.rs
  - zircon_editor/src/ui/workbench/model/menu/mod.rs
plan_sources:
  - user: 2026-06-15 implement editor UI architecture from docs/plans/zircon_editor/editor_ui
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
tests:
  - rustfmt --edition 2021 --check zircon_editor/src/ui/host/commands/*.rs zircon_editor/src/ui/host/module.rs zircon_editor/src/ui/host/mod.rs zircon_editor/src/lib.rs
  - Python tomllib parse of zircon_editor/assets/ui/editor/keymap/default.keymap.toml
  - rustfmt --edition 2021 --check zircon_editor/src/ui/host/commands/registry.rs zircon_editor/src/ui/workbench/model/menu/default_menu_bar.rs zircon_editor/src/ui/workbench/model/menu/mod.rs
  - old static workbench menu builder reference scan
  - git diff --check over command-registry/menu files
  - rustfmt --edition 2021 --check over EditorCommand dispatch/binding/projection files
  - EditorCommand payload reference scan over zircon_editor/src
  - rustfmt --edition 2021 --check over command component adapter files
  - rustfmt --edition 2021 --check over unhandled keymap dispatch bridge files
  - rustfmt --edition 2021 --check over visible Workbench CommandPalette route files
  - rustfmt --edition 2021 --check over CommandPalette open/effect route files
  - rustfmt --edition 2021 --check over retained native keymap pump files
  - rustfmt --edition 2021 --check over visible Workbench CommandPalette native activation files
  - cargo test -p zircon_editor --lib keymap_dispatches_unhandled_keyboard_result_through_editor_command_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib keymap_dispatch --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib key_chord_normalizes_runtime_keyboard_input --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib keymap_resolves_unconsumed_chord_to_command --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - Python tomllib parse of zircon_editor/assets/ui/editor/windows/workbench_window.v2.ui.toml
  - cargo test -p zircon_editor --lib workbench_command_palette --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615
  - cargo test -p zircon_editor --lib command_palette_command_requests_open_effect --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615
  - cargo test -p zircon_editor --lib command_component_adapter_dispatches_palette_open_command --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615
  - cargo test -p zircon_editor --lib command_registry_maps_menu_command_ids_to_editor_events --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615
  - cargo test -p zircon_editor --lib componentized_workbench_inspector_property_edit_updates_row_preview --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615
  - cargo test -p zircon_editor --lib native_unhandled_ctrl_shift_p_opens_workbench_command_palette --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615
  - cargo test -p zircon_editor --lib native_command_palette_enter_commits_focused_workbench_command --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615
  - cargo test -p zircon_editor --lib apply_presentation_projects_open_workbench_command_palette_rows_for_native_input --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615
  - cargo test -p zircon_editor --lib command_palette_option_routes_to_commit_activation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615
  - cargo test -p zircon_editor --lib commands --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib menu_bar_projects_registry_commands_with_contextual_enablement --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_editor/src/ui/host/commands/registry.rs zircon_editor/src/ui/host/editor_event_dispatch.rs zircon_editor/src/tests/editor_event/runtime.rs
  - cargo test -p zircon_editor --lib menu_commands_project_operation_backed_bindings_when_operation_paths_exist --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib editor_command_operation_action_invokes_operation_registry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib command_palette --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib menu_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/menu_pointer/menu_items_for_layout.rs zircon_editor/src/tests/host/retained_menu_pointer/dispatcher.rs zircon_editor/src/tests/host/retained_menu_pointer/pointer_bridge.rs
  - cargo test -p zircon_editor --lib shared_menu_pointer_click_dispatches_reset_layout_through_runtime_dispatcher --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib retained_menu_pointer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615 --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# Editor Host Commands

The editor command layer is owned by `zircon_editor::ui::host`. Runtime UI still owns generic component behavior, including the `CommandPalette` reducer and retained/native row projection; the editor host owns the workbench command catalog, default keymap, and command action payloads.

`EditorCommandRegistry` is the single metadata source for workbench commands. Each descriptor carries a stable id, label, category, menu path, optional default chord, enablement predicate, search keywords, and an action payload. The command ids remain stable (`workbench.project.open`, `workbench.history.undo`, `workbench.view.open.editor.hierarchy`, etc.), while built-in commands with matching operation paths now use `EditorCommandAction::Operation`. Commands without an operation path can still use the existing `MenuAction` contract, and `EditorCommandAction::OpenCommandPalette` is the editor-owned global search entry.

`EditorCommandContext` is intentionally small and state-derived. It answers whether a command is enabled for project-open, undo/redo, selection, and play-mode state. The registry does not pull state from `EditorManager`; callers pass a context snapshot, so menus, shortcut dispatch, and CommandPalette projection can all evaluate the same predicates without introducing runtime/editor coupling.

`EditorKeymap` loads `zircon_editor/assets/ui/editor/keymap/default.keymap.toml` as a TOML document table and maps normalized `EditorKeyChord` values to command ids. `EditorKeyChord::from_keyboard_input(...)` converts runtime `UiKeyboardInputEvent` values into command chords for pressed keys, preserving Control/Shift/Alt/Super modifiers, ignoring modifier keys themselves, and falling back from unidentified keys to legacy key codes for common keys such as Delete and function keys. The keymap is meant to run only after the runtime input route and focused component path decline to consume a key. That preserves text-editing behavior such as Delete in an input field, while still allowing global commands such as `Ctrl+Shift+P`, `Ctrl+S`, `Ctrl+Z`, `Delete`, and `F5` from ordinary workbench focus.

`EditorCommandPaletteEntry` bridges the editor registry to the existing runtime `CommandPalette` data contract. It projects each command as a `UiValue::Map` with `id`, `label`, `source`, `shortcut`, `category`, `keywords`, and `disabled`. The runtime reducer can filter those maps by query/source, skip disabled commands for keyboard focus/commit, and record the committed command id. The low-level editor binding path now has `EditorUiBindingPayload::EditorCommand { command_id }`; runtime command execution resolves that stable command id through `EditorCommandRegistry`, then invokes either an editor operation, a legacy menu event, or the command-palette transient event. Static normalization can still ask `EditorCommandRegistry::event_for_command(...)` for the event shape of built-in operation-backed commands. The host-side command component adapter can turn a `command` domain `UiComponentEvent::Commit { property: "committed_command_id", value: UiValue::String(command_id) }` envelope into that same `EditorCommand` binding.

The workbench menu bar now projects directly from `EditorCommandRegistry::menu_bar_model(...)`. `default_menu_bar_with_extensions(...)` derives an `EditorCommandContext` from `EditorChromeSnapshot`, requests the registry menu projection, then appends extension menus through the existing extension hook. Operation-backed descriptors become `EditorOperation` menu bindings with their `operation_path` attached; non-operation descriptors still carry `MenuAction` bindings. The retained menu-pointer fallback used when no projected model is supplied emits the same built-in operation ids for File/Edit/Selection/Runtime/View/Window rows, while layout preset save/load ids stay on their legacy non-operation menu-action contract. Non-menu actions such as `OpenCommandPalette` remain available to the palette/keymap source but are not surfaced as clickable menu rows; their execution path is handled by editor command dispatch and the retained-host palette-open effect route.

`EditorCommandRegistry::event_for_command(...)` is the static event-shape convergence point. Built-in operation-backed command ids resolve through the built-in operation registry to their mapped `EditorEvent`, so an `EditorCommand` binding for `workbench.history.redo` still normalizes to the same event shape as the old menu action id. Runtime execution no longer stops at that static event shape: `EditorEventRuntime::dispatch_binding(...)` intercepts `EditorCommand` and `EditorOperation` payloads and calls `invoke_operation(...)` for operation-backed commands, preserving operation metadata, operation-stack entries, failure journaling, remote/Cli source mapping, and argument recording. `editor.command_palette` dispatches to `EditorEvent::Transient(EditorEventTransient::OpenCommandPalette)` and records `EditorEventEffect::CommandPaletteOpenRequested`. Projection helpers expose `EditorCommand` ids as action ids where host/native metadata needs a stable row identity.

`component_adapter::command` is the committed CommandPalette bridge. It validates `UiComponentEventEnvelope` targets in the `command` domain, accepts only `Commit` events whose value is a non-empty string command id, builds an `EditorCommand` binding, and asks `EditorEventRuntime` to dispatch it as a retained-host event. This keeps palette execution on the same normalization path as keymap and native palette activation. `workbench_window.v2.ui.toml` mounts the Workbench `CommandPalette` as a collapsed overlay sibling and declares `CommandPalette/Commit` as a `Submit` route. `workbench_window_template_bindings.rs` registers that binding with an `EditorCommand("editor.command_palette")` placeholder so the route is discoverable, while `callback_dispatch::workbench::command_palette` takes the native commit value, builds a `command` domain `committed_command_id` envelope, reuses the adapter validation, and dispatches the resulting command id through the retained-host event path.

The palette open route is now also connected. `UiHostEventEffects` carries `open_command_palette_requested` for the new command-palette effect, and `RetainedEditorHost::open_workbench_command_palette(...)` builds a fresh command list from `EditorCommandRegistry::default_workbench()` plus the current `EditorChromeSnapshot` enablement context. The host writes `commands`, `filtered_commands`, `disabled_commands`, `selected_command_id`, `focused_index`, clears the default source filter, and toggles `popup_open`/visibility through `BuiltinWorkbenchWindowTemplateSurfaceBridge::open_command_palette(...)`. Closing is represented by the same bridge through `close_command_palette(...)`, which collapses the mounted overlay and clears the popup state.

`EditorEventRuntime::dispatch_unhandled_input_keymap_command(...)` is the first runtime-keyboard bridge. It accepts a `UiInputDispatchResult`, verifies the runtime reply disposition is `Unhandled`, extracts the keyboard event, resolves it through `EditorKeymap::resolve_keyboard_input(...)`, then dispatches an `EditorCommand` binding through the same retained-host event path used by the CommandPalette commit adapter. Handled keyboard replies, released keys, non-keyboard events, and unmapped chords return `Ok(None)` and leave the editor journal untouched. The retained native host now calls the same bridge after native text/popup/focus handling declines a pressed key: `UiHostWindow` tracks current modifiers and input sequence, translates the `KeyEvent` into `UiKeyboardInputEvent`, invokes `UiHostContext::on_unhandled_keyboard_input`, and `RetainedEditorHost::dispatch_unhandled_native_keyboard_input(...)` applies the returned command record effects. The later full runtime `UiSurface` hard-cutover pump remains a separate migration edge.

`EditorModule` registers `EditorCommandRegistry` and `EditorKeymap` as lazy managers:

- `EditorModule.Manager.EditorCommandRegistry`
- `EditorModule.Manager.EditorKeymap`

They have no runtime manager dependencies. That keeps command metadata available to menus, CommandPalette state construction, and future shortcut dispatch without forcing project asset or editor manager startup.

The default registry currently covers the existing stable workbench surface:

- File: open/save project, save/reset layout.
- Edit: undo/redo.
- Selection: create scene node variants and delete selection.
- Runtime: enter/exit play mode.
- View and Window: open registered workbench panels and functional windows.
- Help/Command: workbench guide placeholder and global command palette.

The retained native M4 edge is now connected. `Ctrl+Shift+P` reaches `editor.command_palette` through the default keymap and opens the mounted Workbench overlay with live command entries. The mounted `WorkbenchCommandPalette` projects through the command-palette row contract even when the template node is a Workbench Mount, so native Enter and primary pointer activation both submit the visible focused row through `CommandPalette/Commit` and reuse the same `EditorCommand` dispatch path. Operation-backed rows now execute through the operation registry and update the operation stack. Remaining acceptance is live/manual real-window verification plus the later full runtime `UiSurface` hard-cutover pump.

Validation on 2026-06-15 passed Rust formatting, keymap TOML parsing, Workbench window TOML parsing, scoped diff checking, conflict-marker scanning, trailing-whitespace scanning, an old static workbench menu builder reference scan, a scoped scan of `EditorUiBindingPayload` match sites after adding `EditorCommand`, and rustfmt/diff/conflict/trailing scans for the command component adapter. The focused `cargo test -p zircon_editor --lib commands --locked ...` path first caught a local command-registry compile issue where a `default_chord` local variable shadowed the `chord(...)` helper; that is fixed. Later focused Cargo attempts for the registry/menu projection timed out without new Rust diagnostics, and the earlier complete-output rerun had already stopped before `zircon_editor` compilation because the shared `zircon_runtime` post-process/SSR layer failed to compile at that time. The focused `command_component_adapter_dispatches_committed_command_id_through_editor_events` Cargo run reached `zircon_editor` compilation and caught a local adapter type mismatch (`EditorUiBinding` expected, `UiEventBinding` passed); that is fixed. The unhandled-keymap bridge focused validation then reached editor tests and caught two local issues: the default keymap TOML was parsed through `Value::from_str` instead of a TOML document table, and a `bindings` table local shadowed the output vector after that parser fix. Both are fixed. The focused keymap runs pass. After the retained native pump wiring, `native_unhandled_ctrl_shift_p_opens_workbench_command_palette` passes 1/0/2014, `cargo test -p zircon_editor --lib command_palette --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-command-registry-0615` passes 10/0/2005, and `command_registry_maps_menu_command_ids_to_editor_events` passes 1/0/2014. After visible palette activation wiring, `native_command_palette_enter_commits_focused_workbench_command` passes 1/0/2017, `apply_presentation_projects_open_workbench_command_palette_rows_for_native_input` passes 1/0/2017, `command_palette_option_routes_to_commit_activation` passes 1/0/2017, and the `command_palette` filter passes 13/0/2005. After operation-backed command convergence, `menu_commands_project_operation_backed_bindings_when_operation_paths_exist` passes 1/0/2019 after a one-time 12m15s incremental compile, `editor_command_operation_action_invokes_operation_registry` passes 1/0/2019, `commands` passes 37/0/1983, `command_palette` passes 13/0/2007, `menu_binding` passes 6/0/2014, the retained menu-pointer reset-layout regression passes 1/0/2019 after fixing fallback ids, and `retained_menu_pointer` passes 22/0/1994 with four screenshot tests ignored; existing warning noise remains unrelated.
