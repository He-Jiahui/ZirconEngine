---
handoff_kind: failure
status: open
created_at: 2026-08-23
summary_slug: ui12-editor-event-binding-operation-error-conversion
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/optimize/zircon_runtime/74
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs
tests:
  - .\tools\build\build-editor.ps1
---

# Runtime74: Editor event binding operation error conversion

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source `target-editor-host` product build and visual acceptance gate
- 修复责任计划：`docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md`
- 交接原因：最低共享原因位于 Runtime74 正在租赁和迁移的 typed editor-event binding dispatch contract；UI12 不应越权修改该 owner 文件。

## 失败现象与复现证据

2026-08-23 的受管产品构建到达 `zircon_editor` 后，
`zircon_editor/src/ui/host/editor_event_dispatch.rs:567` 与 `:591` 报告
`E0308`。两个调用点返回 `EditorOperationDispatchError`，而函数公开返回
`EditorEventBindingDispatchError`。该枚举已声明
`Operation(#[from] EditorOperationDispatchError)`，但调用点没有触发转换。

Current-source follow-up at 2026-08-23 14:56 +08:00: Runtime74 expanded the
dispatcher-associated typed error surface and renewed its source lease, but the
two product blockers remain. `dispatch_operation_binding` still ends the
operation branch with `.map(Some)`, and `dispatch_editor_command_binding` still
returns `self.invoke_operation_with_binding_path(...)` directly. The shared
Cargo lane is idle; UI12 is waiting only for these existing `From` conversions
before rebuilding the product bundle and starting WGPU visual acceptance.

Current-source follow-up at 2026-08-23 17:33 +08:00: managed product build Job
`42ac6d93102c4b399ad9286c3d8018ed` compiled the current main worktree through
`zircon_editor` and produced seven Runtime74-owned errors:

- `zircon_editor/src/ui/host/mod.rs:78` (`E0603`): public re-export traverses the
  private `editor_event_runtime_access::asset_access` module instead of the
  module's public surface.
- `zircon_editor/src/ui/host/editor_event_dispatch.rs:546`, `:586`, and `:610`
  (`E0308`): three nested typed error results still return directly without
  applying their existing `From` conversions.
- `zircon_editor/src/ui/host/editor_event_execution/menu_action.rs:366` and
  `:369` (`E0308`): `EditorStateOperationError` results are returned directly
  from arms whose authority is `MenuActionExecutionError`.
- `zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs:169`
  (`E0599`): `enabled_asset_types_for_shell` now returns a typed `Result`, but
  the caller invokes `.get(...)` before extracting the registry.

The Job exited 101 and was released with no live compiler processes. Atomic
publication correctly left
`E:\ZirconBuilds\ui12-current-b9277856c5f2-srgb-r1-20260823` absent, so no old
or partial executable will be used for visual acceptance. Evidence log:
`.codex/state/ui12-build-editor-current-b9277856-retry2-20260823.log`.

复现命令：

```powershell
$env:CODEX_THREAD_ID = 'editor-ui12-zui-aa-visual-acceptance-r5-21242973-20260823'
$env:CODEX_SESSION_ID = $env:CODEX_THREAD_ID
.\tools\build\build-editor.ps1 -OutputDirectory E:\ZirconBuilds\ui12-current-1354e50da53d-srgb-r1-20260823
```

证据日志：`.codex/state/ui12-build-editor-r5-retry5-20260823.log`。构建在产品发布前失败，因此目标 bundle 不存在，UI12 不声称编译或视觉通过。

## 最低共享层根因

Runtime74 将 binding dispatch 的错误面从 `String` 收敛为
`EditorEventBindingDispatchError`，但两个 operation-dispatch 分支仍直接返回内部
`Result<_, EditorOperationDispatchError>`。类型边界已有唯一 `From` 映射，调用点尚未用
`?` 完成该映射。

## 架构修复验收

- `dispatch_operation_binding` 与 `dispatch_editor_command_binding` 通过现有 typed `From` 边界返回，不新增字符串映射、别名或兼容分支。
- `dispatch_binding_typed`、menu-action state operations 与 asset registry access
  同样在各自现有 typed `From` 边界使用 `?`，不绕过 owner error surface。
- `EditorAssetOperationInvokeError` 从 `editor_event_runtime_access` 的公开模块
  surface 导出；host root 不穿透私有子模块。
- Runtime74 的 focused typed binding dispatch tests 通过。
- 上述 UI12 `target-editor-host` product build 通过并原子发布 bundle，随后恢复 GPU 截图验收。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- Do not convert either operation error back to `String` or duplicate the existing typed `From` mapping.

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

## 2026-09-18 current-source review and validation admission

The current main snapshot already contains the typed-boundary source repair that
the 2026-08-23 product failure required. A read-only source contract probe was
run from the Windows checkout and passed all seven assertions:

```text
EDITOR_EVENT_TYPED_CONVERSION_SOURCE_PASS
operation binding uses typed ?
editor command binding maps typed result
command invocation uses typed ?
asset registry extracts Result before get
asset error public module export
host reexports public surface
menu state operations typed conversion
```

The probe recorded these SHA-256 values before any new edit:

```text
zircon_editor/src/ui/host/mod.rs                                      5C3F07B76A90A8D22E85FFC4C492995DF95DCF73B9B130C74C4636A7A43BF438
zircon_editor/src/ui/host/editor_event_dispatch.rs                    577463D24B5DA6E9BF3251267B739005DCAC7B3874A5F8E070626704FA8CC049
zircon_editor/src/ui/host/editor_event_execution/menu_action.rs       B05DEC43CF3B806B073A7458EFF4C00B0CBB69E27BDAE8C807B13E417596D622
zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs A0C569AB75ADBE98ED0E7F6F5C8B7588BA2FAE29BAFE61556732636C6D1CDF47
zircon_editor/src/ui/host/editor_event_runtime_access/mod.rs          98E38A62CC44B256EB6DD348700A006374E98B0C04DFF794BEB056F3C09EE832
```

This is source evidence only. The `menu_action.rs` snapshot carries an
unattributed foreign one-line working-tree change, and the repository's managed
Cargo gate remains unavailable while the external `E:\Git\zr_vm` worktree is
dirty. No product build, focused Cargo test, bundle publication, or GPU visual
acceptance was claimed or replayed. The failure therefore remains open and
awaits a clean external revision plus the original Runtime74 focused tests and
the UI12 `target-editor-host` build.

The lower-layer source probe was submitted through the coordinator as ticket
`1b96353470cb42b2bf0c7809ece8427c` (request
`failure-roll-01a084c8-runtime74-ui12-event-binding-source-20260918-r2`). It is
queued with no blockers and is intentionally scoped as source-contract evidence
(`upwardAcceptance: false`); its result must not be reused as the missing Cargo
or product-build acceptance.

### 2026-09-19 corrected static receipt

The original ticket `1b96353470cb42b2bf0c7809ece8427c` failed in the checker
itself (`re.PatternError: unbalanced parenthesis`) before any source assertion.
Corrected ticket `3a047a4fea10498da70c829edb084eb4` (request
`failure-roll-01a084c8-runtime74-ui12-event-binding-20260919-r3`) replaced the
malformed regular expressions with balanced literal source anchors against the
current failure-doc snapshot. Coordinator job `682aad8df39f48a7a6bfa44ef447a2a5`
/ run `3a047a4fea10498da70c829edb084eb4` exited 0 and emitted
`EDITOR_EVENT_TYPED_CONVERSION_SOURCE_PASS` plus all seven typed-boundary
assertions; cleanup completed. This remains static source evidence only and
does not satisfy the Runtime74/UI12 Cargo, product, GPU, review, return, or
closeout gates.

### 2026-09-29 approved-root acceptance command correction

The original 2026-08-23 reproduction command is retained below. Its output
root is outside the current physical `D/E/F:\cargo-targets` policy:

```powershell
tools/build/build-editor.ps1 -OutputDirectory E:\ZirconBuilds\ui12-current-b9277856c5f2-srgb-r1-20260823
```

The tests frontmatter now invokes the builder without `OutputDirectory`, so it
generates a unique approved-root bundle path. This metadata correction does not
claim a build, product publication, or UI12 visual acceptance.
