---
related_code:
  - zircon_editor/src/core/commands/
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editing/operation/
  - zircon_editor/src/core/editing/history.rs
  - zircon_editor/src/scene/selection/
  - zircon_editor/src/core/gateway/
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_extension/view_descriptor.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_session_state.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/assets/ui/editor/keymap/default.keymap.toml
implementation_files:
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editing/operation/command.rs
  - zircon_editor/src/core/editing/operation/error.rs
  - zircon_editor/src/core/editing/operation/factory.rs
  - zircon_editor/src/core/editing/operation/registration.rs
  - zircon_editor/src/core/editing/history.rs
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/commands/mod.rs
  - zircon_editor/src/core/commands/descriptor.rs
  - zircon_editor/src/core/commands/when.rs
  - zircon_editor/src/core/commands/document_kind.rs
  - zircon_editor/src/core/commands/play_mode_predicate.rs
  - zircon_editor/src/core/commands/eval_snapshot_handle.rs
  - zircon_editor/src/core/commands/contribution.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/commands/registry_handle.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/commands/menu.rs
  - zircon_editor/src/core/commands/menu_model.rs
  - zircon_editor/src/core/commands/palette.rs
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/commands/key_chord.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_extension/view_descriptor.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/ui/host/editor_session_state.rs
  - zircon_editor/src/ui/host/workspace_state.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_editor/editor/07/failure-2026-07-12-command-eval-focused-document-projection.md
tests:
  - zircon_editor/src/core/commands/descriptor.rs::tests::command_enablement_does_not_materialize_an_effective_when_clause
  - zircon_editor/src/core/commands/document_kind.rs::tests::document_kind_validation_streams_segments
  - zircon_editor/src/core/commands/registry.rs::tests::command_descriptor_validation_streams_path_segments
  - zircon_editor/src/tests/editor_event/runtime/registry.rs::editor_operation_path_validation_streams_segments_without_collecting
  - zircon_editor/src/tests/commands/registry.rs
  - zircon_editor/src/tests/commands/when.rs
  - zircon_editor/src/tests/commands/descriptor_when.rs
  - zircon_editor/src/tests/commands/operation_factory.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/dispatcher.rs
  - zircon_editor/src/tests/gateway/handle.rs
  - zircon_editor/src/tests/editor_event/runtime/registry.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration.rs
  - zircon_editor/src/tests/editor_event/runtime/when_evaluation.rs
  - zircon_editor/src/tests/editor_event/focused_document_projection_hard_cut.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/exclusive_page.rs
doc_type: module-detail
---

# Editor commands

`core::commands` is the headless owner of editor command identity, metadata, discovery, invocation lookup, menu projection, palette projection, and keymap metadata. The former `ui/host/commands` owner and the parallel `EditorOperationDescriptor`/`EditorOperationRegistry` graph were deleted; no compatibility module or re-export remains.

## Single descriptor and identifier space

`EditorCommandDescriptor` contains the former command and operation metadata in one record: display text, category, menu path, default chord, structured `when`, keywords, payload schema, remote-call policy, and required capabilities. Its key is `EditorOperationPath`, retained as the validated stable identifier type used by UI bindings and the operation-control DTOs. Undo display metadata belongs to the matching `OperationCommandFactoryRegistration`, next to the factory that creates the undoable command; it is not duplicated on the descriptor.

`EditorCommandMenuProjection` separates menu metadata from its single materialization owner. Ordinary commands use `CommandRegistry`; generated extension-view commands use `ExtensionRegistry`. Both keep a canonical `menu_path` for discovery, but only the selected owner emits a menu row. This prevents the shared registry and the capability-filtered extension projection from rendering the same View command independently, and it keeps an inactive plugin out of Workbench menus without deleting command metadata or weakening invocation permission checks.

`EditorCommandAction` has two explicit states:

- `Emit(EditorEvent)` stores the already-supported typed event inline. There is no command-to-operation string hop.
- `Operation` marks commands that require an Editor 03 operation factory. The command registry stores a matching `OperationCommandFactoryRegistration`; ordinary event lookup returns `OperationRequiresInvocation`, while the operation dispatcher creates a real `EditCommand` and executes it through the transaction engine. Missing or mismatched factories are typed registration/dispatch failures, never fabricated events or fake undo records.

## Performance-sensitive evaluation

`is_enabled` evaluates the stored `when` clause, required capabilities, and `AssetWritable`
requirement directly. It does not call `effective_when`, so menu, palette, and invocation permission
checks do not clone capability strings or materialize/sort a temporary clause tree. The public
`effective_when` projection remains available to serialization and inspection consumers.

Document-kind, menu-path, payload-schema, and `EditorOperationPath` validation stream dotted/path
segments without a temporary `Vec`. Source guards completed RED-to-GREEN and formatting/diff checks
pass; focused current-source Cargo and retained-menu allocation traces remain pending.

The retained pointer integration guard registers an operation-only descriptor plus factory, clicks
the materialized menu item, and requires one `EditCommand::apply` plus an undoable Global history
record. This locks the real retained route to factory and transaction execution rather than merely
testing registry lookup or an event-backed operation.

### Palette catalog generation and query windows

`EditorCommandRegistry` publishes one immutable `Arc<EditorCommandPaletteCatalog>` per registry
generation. Repeated palette opens share that catalog and its normalized search documents; a
successful descriptor/factory/asset-write-target registration advances the generation and clears
the lazy catalog once. Failed or idempotent mutations do neither. The deleted
`command_palette_entries` and `command_palette_value` full-materialization APIs have no compatibility
wrapper.

`command_palette_query_window(context, query, offset, limit)` is the sole palette discovery query.
It evaluates the canonical descriptor `when` predicate, fuzzy-matches the generation-owned search
document, and returns lightweight catalog handles for only the requested window. Non-empty queries
use fixed 256 score buckets: the first streaming pass counts ranks and the second materializes the
requested page in stable descriptor order inside each score. Consequently every match remains
addressable by `offset` without retaining a full result-id vector, while query metrics report
visited entries, text comparisons, total matches, retained handles, and owned buffers.

The retained host projects 8 visible rows plus 4 overscan rows. It no longer clones the complete
enabled catalog on open or query edit; the temporary `commands`/`filtered_commands` UI values are
bounded to that 12-row window. `CommandPalette/QueryChanged` is declared in the `.zui`, registered in
the authoritative template binding table, intercepted before generic menu dispatch, and coherently
updates query/window/generation/match-count while the registry lock is released. Keyboard-driven
window advance beyond the first page and managed input-p95 evidence remain open.

`EditorOperationInvocation`, `EditorOperationControlRequest`, `EditorOperationControlResponse`, source types, and control errors remain in `core::editor_operation` because they are transport DTOs, not a second registry.

`EditorOperationPath` owns its wire invariant as well as its constructor invariant. Its handwritten
`Deserialize` implementation decodes a string and calls the same canonical `parse` path used by
commands and extensions. Persisted workspace, plugin, CLI, and control payloads therefore reject
short paths, uppercase text, empty segments, spaces, and punctuation instead of constructing an id
that the public API cannot create. Consumers such as `AssetToolkitOpenRoute` derive ordinary serde
against the typed id and do not repeat operation-path validation at individual wire boundaries.

## Unique runtime owner

`EditorCommandRegistry` rejects duplicate typed identifiers and validates menu paths and payload schema identifiers. `EditorCommandRegistryHandle` owns the registry behind one shared mutex. `EditorContextBuilder` creates the built-in instance, and `EditorManager`, `EditorHostEventController`, and the named `EditorCommandRegistry` module service all share that exact handle.

`EditorCommandContributionSet` carries descriptors only until extension registration. Registration drains those one-shot descriptors into a transactional clone of the shared registry, atomically replaces the value held by the same handle, and persists only typed command ids with the extension's non-command contributions. `EditorExtensionRegistry` never owns a second `EditorCommandRegistry` or a long-lived duplicate descriptor graph.

## Structured when evaluation

`WhenClause` replaces the deleted `EditorCommandContext` and `EditorCommandEnablement` types. It supports `Always`, project/history availability, validated `DocumentKind`, typed `SceneModeId`, selection count, `PlayModePredicate` over `PlayStateKind`, named capabilities, and recursive `All`/`Any`/`Not` composition. An inapplicable contextual predicate stays false in headless evaluation even under `Not`, so the absence of a document, scene mode, selection, or interactive play state cannot become a fabricated success.

`CommandEvalCtx` has two explicit modes. Interactive snapshots carry project, undo/redo, focus, scene-mode, selection, play-state, and a deterministic capability set. Headless snapshots carry only capabilities; their stored play state is `Edit` for determinism, but `PlayMode(Edit)` remains inapplicable and false. `ViewDescriptor.document_kind` is now the typed domain owner for scene、Prefab、material、UI asset、animation sequence 与 animation graph；`EditorSessionState.focused_view` 是跨主文档区和浮动窗口的唯一焦点 owner。Chrome 构建只从 `focused_view -> ViewInstance -> ViewDescriptor.document_kind` 投影 `focused_document_kind`，不会从 tab title、显示名、路径后缀或 descriptor id 猜类型。默认布局或仅打开项目时，没有显式焦点就持续保留 `None`；只有原先为 `Some` 的焦点实例关闭或失效时，才回到当前主页面的 active document。`Building` is represented by the core DTO and tests but is not synthesized from the current two-state UI session surface. Remaining authority projections stay routed to their functional owners: [Editor 04 Play state](../../plans/zircon_editor/editor/04/failure-2026-07-12-command-eval-play-state-projection.md) and [Editor 05 scene mode and selection](../../plans/zircon_editor/editor/05/failure-2026-07-12-command-eval-scene-mode-selection-projection.md).

`EditorContext` owns one `CommandEvalSnapshotHandle`. The host projects `EditorChromeSnapshot` plus the manager capability snapshot into that handle during reflection and retained-host recomputation. Menu, palette, and UI-binding invocation read cloned interactive snapshots; Remote and CLI operation control create headless snapshots from the same capability source. Snapshot locks are released before registry or shell mutation, avoiding a new shell/registry lock-order cycle.

`required_capabilities` remains sorted, duplicate-free discovery metadata. It is not a second permission gate: `effective_when()` derives a capability conjunction around the descriptor's stored `when`, and `is_enabled()` is the shared predicate entry used by menus, palette rows, list filtering, UI invocation, and remote invocation. Repeated builders and deserialization normalize only the metadata vector and never materialize capability clauses back into the serialized descriptor, so repeated evaluation cannot accumulate duplicate predicates.

Editor-internal remote operation control, editor-command UI bindings, workbench reflection menus, retained-host recomputation, and the command palette all lock the shared handle. Production view-model builders require an explicit registry reference and have no implicit `default_workbench()` fallback, so extension commands are visible on those editor-owned surfaces. The `zircon_app` CLI still uses stale symbols and is tracked as an open Editor 16 handoff in [`failure-2026-07-12-command-registry-hard-cut-cli.md`](../../plans/zircon_editor/editor/16/failure-2026-07-12-command-registry-hard-cut-cli.md); this milestone does not claim that the application CLI gate is established.

## Headless and UI boundary

The registry and its `MenuBarModel`, `MenuModel`, and `MenuItemModel` products live in `core::commands` and do not import `crate::ui`. Menu items contain typed action/operation data; the workbench event adapter creates `EditorUiBinding` values at the UI boundary. This preserves headless command listing and menu/palette projection without a `core -> ui` dependency.

Built-ins are declared in `defaults.rs`; their previous enablement variants map directly to structured clauses. Menu rows and palette rows are projections of registered descriptors, while `assets/ui/editor/keymap/default.keymap.toml` binds the same typed command identifiers. Real edit-command factories are now owned by `EditorCommandRegistry.operation_factories`; retained and remote invocation create commands through that registry and execute them through the shared transaction engine. Contributed-menu three-source merging and user keymap overlays remain later Plan 08 milestones.

## Validation status

The Workbench world-command stack now stores ordered before/after
`HistorySelectionSnapshot` values only on selection-changing entries. Create,
delete, and import operations restore the complete active-domain set and primary;
update, reflection, batch-inspector, and gizmo entries carry no selection
snapshot. Workbench restores a snapshot through `SelectionModel::replace_active`,
so a command-level `Option<NodeId>` can no longer silently collapse an ordered
multi-selection while the remaining world commands migrate to Editor03.

M1.1 adds duplicate-id rejection, source-tree hard-cut guards, and a runtime assertion that a command registered by an extension is returned by list and accepted by invoke through the same registry. M1.2 adds predicate composition, state/capability/headless matrices, required-capability normalization, menu/palette agreement, remote headless list/invoke and interactive UI invocation. Editor07 additionally covers project-open 不伪造 scene focus、typed descriptor 到 Chrome/CommandEvalCtx 的投影、浮动动画文档激活以及关闭焦点浮动文档后的 UI asset focus 回退；静态 hard-cut guard 1/1 已通过。动画 session 的私有 resolver、错误分类与测试文本也已从旧 “active center tab” 词汇硬切到唯一 `focused view` 语义，guard 先红后绿且旧字段/函数/错误词汇扫描为 0。为遵守结构计划，新增的 extension `ViewDescriptor` 责任已从 907 行的 `core/editor_extension.rs` 提取到 folder-backed `core/editor_extension/view_descriptor.rs`，root 收敛为 860 行；模块边界 guard 完成 RED→GREEN 1/1，日志为 `.codex/tmp/editor07-focused-document-projection-module-boundary-red-20260714.log` 与 `.codex/tmp/editor07-focused-document-projection-module-boundary-green-20260714.log`。current-source Cargo exact 第二轮已通过受管 job `9cc782db74224c43887dfe73b46a4680` 实际编译；本模块自有的 manager-name import 已跟随 `ui::host::module` 唯一 owner 修正，不恢复 host-root re-export。测试体仍被 EditorUI03 retained paint-text fixture 缺少 `ShapedGlyph.font_instance_id` 的 E0063 阻断，日志 `.codex/tmp/editor07-focused-document-current-exact-r2-20260714.log`，失败已追加到对应文本计划，因此本模块暂不声明 Cargo 门通过。Static validation also covers formatting, stale-symbol/path scans, the `core -> ui` boundary, production snapshot ownership, and diff whitespace.
