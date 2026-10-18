# UI 后续产品化与验证归档计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. In `zirconEngine`, follow milestone-first cadence: implementation slices first, testing stage at milestone boundary.

**Goal:** 把 M10/M12/M13 foundation gate 之后剩余的 workspace validation、Editor contract productization、M18 binding semantics、M14/M21 runtime foundation 和 UI roadmap productization 缺口归档成可执行后续路线。

**Architecture:** 继续保持 Runtime UI asset/template/component behavior authority 在 `zircon_runtime::ui`，neutral UI DTO contract authority 通过 `zircon_runtime_interface::ui` 收敛，Editor 只消费 runtime/interface-owned diagnostics、descriptor、contract、package report 与 authoring DTO。后续工作按“DTO identity cutover -> editor-facing runtime report projection -> editor recovery/tools/resource UX -> dual-host parity -> broad validation”推进，避免用上层 workaround 掩盖底层 shared support 问题。

**Tech Stack:** Rust, Cargo, Serde/TOML, `zircon_runtime::ui::template::asset`, `zircon_runtime::ui::component`, `zircon_editor::ui::asset_editor`, existing `.ui.toml` assets, `.codex/plans`, `docs/ui-and-layout`。

---

## 执行边界

| Area | Decision |
| --- | --- |
| Branch | 继续直接在 `main`；不创建 worktree/feature branch。 |
| Validation | 每个 milestone 有单独 testing stage；implementation slice 不强制每步跑 full workspace。 |
| Cargo target | 运行 Cargo 前检查 target drive；`<= 50GB` 先 clean 对应 target dir。 |
| Active overlap | 避开 graphics/plugin renderer、Runtime UI showcase/schema panel、active incremental layout/surface lane 和未分配的 broader resource browser / loader / RHI UX；M15 runtime/package collector-cache-invalidation hook、M15 minimal editor dependency view、M15 asset-browser direct-reference graph slice、M16 back-half、M18 runtime binding semantics、M18 editor binding diagnostic/schema projection、M21 action policy runtime foundation、M14 localization runtime foundation、M21/M14 editor runtime-report projection、M22 focused runtime/editor dual-host parity 已完成。后续 watcher/runtime-loader/render parity/broad validation 只能在 runtime-interface DTO cutover 与 active renderer/editor-operation/layout lanes 协调后推进。 |
| No-Slint fence | 不恢复 `zircon_editor/ui/**/*.slint` 或 `temp/slint-migration`。 |
| Docs | 每个落地 milestone 更新 `docs/ui-and-layout`、`.codex/plans` 和 session/archive evidence。 |

## 当前已知活跃/已存在计划

| Item | Status | Archive Handling |
| --- | --- | --- |
| M15 Resource Refs | Runtime/package gate accepted on 2026-05-01; classifier regression fixed on 2026-05-02; minimal editor dependency view and resolver/file-existence diagnostics accepted on 2026-05-07; asset-browser direct reference graph slice landed on 2026-05-08 with Cargo validation blocked by active incremental layout field drift; planning/spec: `docs/superpowers/plans/2026-05-01-ui-asset-resource-refs-m15.md`, `docs/superpowers/specs/2026-05-01-ui-asset-resource-refs-m15-design.md` | Runtime resource model, serialized `imports.resources`, fallback validation, resource-kind inference, diagnostics, dependency row DTO, collector, compiled dependency accessors, M12 cache/invalidation fingerprint, M16 manifest hookup, editor constructor compatibility, strict resource-table classifier, UI Asset Editor read-only rows, session-level typed accessors, runtime-owned file-existence resolver diagnostics, and UI asset direct references for collected resource dependencies are landed. Broader resource browser UX, watcher/hot reload, runtime loader backend, and graphics/RHI consumption remain future milestones. |
| M16 Package Validation / Back-Half | V1 session archived at `.codex/sessions/archive/20260501-2013-m16-ui-compiled-artifact-package-validation.md`; formal plan: `docs/superpowers/plans/2026-05-01-ui-compiled-artifact-package-validation.md`; back-half reconciled in active UI productization session. | Header/manifest/package report, binary artifact envelope, cache record, package manifest writer/importer, and compiled resource dependency rows are landed. Future work is cross-process cache store, runtime-loader/editor resource browser UX, and broad workspace green. |
| Runtime UI Showcase | Schema-panel session archived; root notes still include `.codex/sessions/20260428-0108-runtime-ui-showcase-deep-state.md` and `.codex/sessions/20260430-2203-ui-cutover-runtime-visual-contract.md` | 不碰 showcase retained state/projection/data-source 或 visual-contract owned files，除非对应 session 关闭。 |
| Graphics/Plugin | Handoff/archive: `.codex/sessions/archive/20260501-1850-plugin-renderer-hard-cutover.md` | Broad validation blocker 仍归因到 graphics/plugin；UI plan 不修改 graphics/plugin。 |

## Milestone 0: Broad Workspace Validation Triage

**Current status:** Scoped validation executed and partially unblocked. Runtime package validation remains blocked by graphics/plugin `ViewportRenderFrame` boundary drift. The earlier editor `UiAssetImports.resources` constructor drift was fixed in the M15 gate, and the later UI-owned editor failures in UI Asset Editor detail dispatch, Component Showcase authored props, and Asset Browser button props have also been fixed; `cargo test -p zircon_editor --lib` passed in the closeout target. Workspace validation is not run and not green.

**Goal:** 把“workspace 未声明通过”的原因从模糊风险变成可追踪验证矩阵和 blocker 清单。

| Scope | Details |
| --- | --- |
| In scope | 只做验证/归因/归档，不顺手修 graphics/plugin/editor watcher drift。 |
| Out of scope | 不修 RHI、VG/HGI plugin renderer、Runtime UI showcase、editor watcher。 |
| Owners | `.github/workflows/ci.yml`, Cargo workspace root, `zircon-dev` validator, `.codex/sessions` blockers。 |

### Implementation Slices

- [x] 刷新 active session context，记录当前 blockers 属于哪个 active owner。
- [x] 建立 validation matrix：`zircon_runtime`, `zircon_editor`, workspace-level build/test, focused UI suites。
- [x] 对每条失败输出归类：UI-owned、graphics/plugin-owned、editor watcher-owned、format-only、Cargo environment。
- [x] 把“不声明 broad green”的原因写入归档，不把 unrelated failures 改成 UI debt。
- [ ] 如果 graphics/plugin blockers 已关闭，再运行 full workspace validation stage；M15 constructor drift 已在 M15 gate 修复并通过 editor check，editor closeout target 也已通过 package lib tests。

### Testing Stage

| Command | Expected |
| --- | --- |
| `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never` | Runtime package validation passes or produces categorized non-UI blocker. |
| `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never` | Editor package validation passes or produces categorized non-UI blocker. |
| `cargo test --workspace --locked --jobs 1 --target-dir E:\cargo-targets\zircon-workspace-validation-closeout --message-format short --color never` | Only run when active owner blockers are resolved or user explicitly asks for broad failure evidence. |

### Exit Evidence

- Archive has exact command outputs, failure ownership, and whether workspace is actually green.
- No UI milestone is marked complete solely because scoped UI tests pass.

### Current Triage Snapshot: 2026-05-01 20:27 +08:00

| Signal | Evidence | Ownership |
| --- | --- | --- |
| CI baseline | `.github/workflows/ci.yml` runs `cargo build --workspace --locked --verbose` then `cargo test --workspace --locked --verbose` on Ubuntu after installing winit/wgpu/slint system deps. | Workspace CI, not UI-specific. |
| Validator script availability | The documented `.codex\skills\zircon-dev\scripts\validate-matrix.ps1` / `.opencode\skills\zircon-dev\scripts\validate-matrix.ps1` was not found in this checkout; Milestone 0 uses direct Cargo commands until the validator script exists. | Validation tooling gap. |
| Cargo disk gate | `Get-PSDrive -Name E` reported `Free = 11469635584` bytes, below the `<= 50GB` cleanup threshold. | Environment/blocker; do not start new heavy target without cleanup/coordination. |
| Active Cargo writers | Fresh process scan found existing Cargo/rustc work in `E:\cargo-targets\zircon-m16-ui-package-validation`, `E:\cargo-targets\zircon-ui-foundation-m10-contract-fresh`, and `zircon_plugins\target`. | Owned by active M16 / plugin renderer / other validation lanes; this Milestone 0 triage does not compete. |
| Runtime package validation | `cargo test -p zircon_runtime --lib ...` was not started by this triage because active Cargo writers and low disk made a new validation target unsafe. | Deferred; no runtime package pass claim. |
| Editor package validation | `cargo test -p zircon_editor --lib ...` was not started by this triage for the same active-writer/low-disk reason. | Deferred; no editor package pass claim. |
| Workspace validation | Full `cargo test --workspace --locked ...` remains gated until graphics/plugin blocker ownership is resolved and target-disk constraints clear. | Deferred; no workspace green claim. |

### Validation Run: 2026-05-01 21:18 +08:00

| Command | Result | Ownership |
| --- | --- | --- |
| Preflight process scan | No active `cargo.exe` / `rustc.exe` processes were found before starting validation. | Clear to run scoped validation serially. |
| `Get-PSDrive -Name E` | `Free = 188547960832` bytes; target cleanup was not required. | Environment OK for `E:\cargo-targets\zircon-ui-validation-closeout`. |
| `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never` | Failed after running tests: `609 passed; 1 failed; 0 ignored`. Failing test: `tests::graphics_surface::internal_visibility::graphics_surface_keeps_viewport_frame_and_icon_source_internal`. | Non-UI graphics/plugin boundary blocker. The assertion expects `ViewportRenderFrame` to remain crate-private, while active plugin renderer hard-cutover notes intentionally changed `zircon_runtime/src/graphics/mod.rs` and `zircon_runtime/src/graphics/types/mod.rs` around that seam. |
| `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never` | Failed during compile with `error[E0063]: missing field resources in initializer of UiAssetImports` at `zircon_editor/src/ui/asset_editor/style/theme_authoring.rs:224:18`. | UI/M15 resource-ref constructor drift. `UiAssetImports` now includes `resources: Vec<UiResourceRef>` in `zircon_runtime/src/ui/template/asset/document.rs`; editor theme promotion still constructs only `widgets` and `styles`. |
| Workspace validation | Not run after scoped failures. | No workspace green claim. |

### M15 Gate Follow-Up: 2026-05-01 22:48 +08:00

| Signal | Evidence | Ownership |
| --- | --- | --- |
| M15 constructor drift | `zircon_editor/src/ui/asset_editor/style/theme_authoring.rs` initializes `UiAssetImports.resources`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never` passed. | UI/M15 fixed. |
| Runtime package blocker | Previous broad runtime package failure remains attributed to graphics/plugin `ViewportRenderFrame` boundary drift; M15 and the editor closeout did not edit that lane. | Non-UI graphics/plugin blocker. |
| Workspace validation | Still not run after focused M15 gate; no workspace green claim. | Deferred until graphics/plugin blocker clears or user asks for broad failure evidence. |

### Editor Validation Recovery: 2026-05-02 00:14 +08:00

| Command / Signal | Result | Ownership |
| --- | --- | --- |
| `Get-PSDrive -Name E` | `Free (GB) 150.84`; no target cleanup required for `E:\cargo-targets\zircon-ui-validation-closeout`. | Environment OK for scoped editor recovery. |
| Process scan for `cargo` / `rustc` | No active Cargo writers before the accepted serial reruns. | Clear to run scoped validation serially. |
| Initial parallel focused validation attempt | Hit Cargo target lock / dep-info contention. | Environment contention only; superseded by serial reruns. |
| `cargo test -p zircon_editor --lib ui_asset_editor_host_genericizes_detail_event_dispatch --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never` | Passed. | UI Asset Editor host guard now follows the component-adapter detail commit route. |
| `cargo test -p zircon_editor --lib component_showcase_authored_props_are_declared_by_runtime_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never` | Passed. | Authored showcase `VirtualList` / `PagedList` props now use runtime catalog `items`; generated `collection_items` remains projection state only. |
| `cargo test -p zircon_editor --lib asset_browser_projection_maps_bootstrap_asset_into_mount_nodes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never` | Passed. | Asset Browser authored `Button` nodes now use runtime `text` props instead of non-catalog `label`. |
| `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never` | Passed: `876 passed; 0 failed; 1 ignored`. Output: `C:\Users\HeJiahui\.local\share\opencode\tool-output\tool_de453af3e0010aErgOQWsOjnn3`. | Scoped editor package validation is green. Runtime/workspace green is still not claimed. |

## Milestone 1: Editor Contract Productization

**Goal:** 把 M10 runtime contract errors 产品化到 UI Asset Editor：结构化 diagnostics、source-outline deep link、root-class authoring UX。

| Scope | Details |
| --- | --- |
| Runtime owner | Existing `zircon_runtime/src/ui/template/asset/component_contract/*` remains source of truth. |
| Editor owner | `zircon_editor/src/ui/asset_editor`, diagnostics/session/presentation surfaces. |
| Docs | `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md`, `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md`。 |

### Implementation Slices

- [x] Define structured contract diagnostic DTO sourced from runtime component-contract validation without parsing display strings where possible.
- [x] Add diagnostic codes for invalid public part, private selector, API mismatch, closed root class, binding/focus private target.
- [x] Add source outline target mapping from diagnostic location to component/reference node when a runtime diagnostic carries `target_node_id`.
- [x] Add root-class policy authoring state: `AppendOnly` vs `Closed`, plus editor-side validation preview.
- [x] Surface diagnostics in UI Asset Editor pane presentation without changing compiler authority.
- [x] Add editor tests for diagnostic code, target node/style rule, and root-class policy editing.

### Testing Stage

| Command | Expected |
| --- | --- |
| `cargo test -p zircon_runtime --lib asset_component_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-contract-productization --message-format short --color never` | Runtime contract gate still passes. |
| `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-contract-productization --message-format short --color never` | Editor contract diagnostics pass or precise unrelated blocker recorded. |
| `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-contract-productization --message-format short --color never` | Editor type-check passes. |

### Exit Evidence

- Runtime still owns contract semantics.
- Editor displays stable structured diagnostics and source/deep-link targets.
- Root-class authoring UX exists without compatibility shims.

### Diagnostic Slice Evidence: 2026-05-01 22:26 +08:00

| Command | Result | Notes |
| --- | --- | --- |
| `rustfmt --edition 2021 --check --config skip_children=true ...` over touched runtime contract/editor diagnostic/session/test files | Passed with no output. | Used `skip_children=true` to avoid formatting active M15 child modules. |
| `git diff --check -- ...` over touched runtime contract/editor diagnostic/session/test files | Passed with LF-to-CRLF notices only. | No whitespace errors in the owned slice. |
| `cargo test -p zircon_runtime --lib asset_component_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never` | Passed, `15 passed; 0 failed; 611 filtered out`. | Output still has unrelated runtime graphics warnings. |
| `cargo test -p zircon_editor --lib contract_diagnostics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never` | Passed, `3 passed; 0 failed; 877 filtered out`. | First cold compile attempt timed out; warmed rerun passed. Output still has unrelated runtime graphics warnings and one unrelated editor test unused-import warning. |
| `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never` | Passed. | Type-checks the editor diagnostic projection surface; output still has unrelated runtime graphics warnings. |

This closes the structured contract diagnostic projection slice. It did not close the whole milestone until the root-class authoring UX follow-up below, and full workspace validation is still blocked by unrelated graphics/plugin work.

### Root-Class Authoring UX Evidence: 2026-05-02 02:24 +08:00

| Command | Result | Notes |
| --- | --- | --- |
| `rustfmt --edition 2021 --check ...` over the root-class session/presentation/host/adapter/test slice | Passed with no output. | Formatting check was scoped to the owned root-class files. |
| `cargo test -p zircon_editor --lib ui_asset_editor_session_projects_and_updates_root_class_policy --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never` | Passed. | First cold compile attempt timed out; warmed rerun passed. Covers presentation field, editability flag, inspector item, document mutation, undo, and redo. |
| `cargo test -p zircon_editor --lib asset_editor_component_adapter_updates_selected_component_root_class_policy --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never` | Passed. | Covers Runtime UI component-adapter commit into selected component contract. |
| `cargo test -p zircon_editor --lib editor_component_adapter_registry_advertises_reflection_and_asset_editor_sources --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never` | Passed. | Covers field descriptor exposure for `component.root_class_policy`. |
| `cargo test -p zircon_editor --lib ui_asset_editor_host_genericizes_detail_event_dispatch --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never` | Passed. | Covers Slint host detail route and adapter-owned target path guard. |
| `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never` | Passed, `204 passed; 0 failed; 675 filtered out`. | Covers the UI Asset Editor package test subset after the focused root-class gates. |
| `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never` | Blocked before compile. | Cargo refused to update `Cargo.lock` under `--locked` because of unrelated runtime-interface manifest/lock drift in the shared checkout. |
| `cargo test -p zircon_runtime --lib asset_component_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never` | Blocked before compile. | Same unrelated `--locked` lockfile drift; not treated as M10 failure evidence. |

This closes Milestone 1 at focused editor-contract productization scope: runtime owns the component-contract semantics, editor projects structured diagnostics and source-outline targets, and root-class policy authoring now edits the runtime-owned contract through replay-aware session state and the Runtime UI component-adapter envelope. Workspace/runtime broad green is still not claimed because the known graphics/plugin blocker and current unrelated runtime-interface manifest/lock drift remain outside this milestone.

## Milestone 2: M18 Runtime Binding Expression Semantics

**Goal:** 把当前 binding/action/payload editing 从“可编辑/可预览”提升为 runtime-owned binding schema 和 restricted expression model。

| Scope | Details |
| --- | --- |
| In scope | Prop/class/visibility/enabled binding schema, expression AST, static validation, preview fixtures。 |
| Out of scope | General scripting VM, arbitrary eval, host side-effect policy。 |
| Runtime owner | `zircon_runtime/src/ui/template/asset/binding` owns runtime validation; `zircon_runtime_interface/src/ui/template/asset/binding` owns neutral DTOs。 |

### Implementation Slices

- [x] Inventory existing `UiBindingRef`, `UiActionRef`, component data-binding descriptors, and editor preview mock schema.
- [x] Define restricted expression AST: literals, param refs, prop refs, comparison, boolean combinators, enum/string matching.
- [x] Add binding target schema for `props`, `classes`, `visibility`, `enabled`, and action payload fields.
- [x] Validate expressions during asset compile before expansion/projection.
- [x] Add preview fixture evaluation with deterministic inputs and no side effects.
- [x] Add editor authoring diagnostics for invalid target, invalid value kind, unresolved ref, unsupported operator.
- [x] Keep action side-effect classification out of M18 and reserve for M21.

### Testing Stage

| Command | Expected |
| --- | --- |
| `cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` | New binding schema/expression tests pass. |
| `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never -- --nocapture` | Existing asset compiler behavior remains stable. |
| `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` | Editor binding diagnostics/preview tests pass. |

### Exit Evidence

- Binding expressions are parsed/validated by runtime.
- Editor only authors and previews against runtime schema.
- No arbitrary script execution or host side-effect policy is introduced in M18.

### Runtime Semantics Evidence: 2026-05-02 08:10 +08:00

| Command | Result | Notes |
| --- | --- | --- |
| `rustfmt --edition 2021 --check <M18 binding runtime files>` | Passed with no output. | Covers the runtime binding schema, expression parser, diagnostics, compiler hook, and public integration test file. |
| `cargo test -p zircon_runtime --test ui_asset_binding_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` | Passed, `16 passed; 0 failed`. | Covers valid prop/class/visibility/enabled/action-payload targets, descriptor-owned prop authority, missing action-payload target rejection, invalid target/value kind/unresolved ref/unsupported operator diagnostics, boolean operators/parentheses, preview-expression filtering, and compiler precondition integration. |
| `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` | Passed with unrelated graphics/plugin warnings. | Production runtime library type-checks with M18 binding schema and compiler wiring. |
| `cargo test -p zircon_runtime --lib asset_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never` | Blocked before filtered tests by unrelated lib-test UI DTO identity errors. | The errors are in asset font and graphics UI renderer test modules, already tracked by runtime-interface/UI coordination notes. |
| `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never -- --nocapture` | Blocked by the same unrelated lib-test compile blocker. | Not treated as M18 source failure. |

This closes M18 runtime binding expression semantics at runtime-owned scope. The later editor binding diagnostic/schema projection evidence below also closes the editor-facing M18 projection at focused scope; broad workspace green is still not claimed.

### Editor Projection Evidence: 2026-05-02 11:08 +08:00

| Command | Result | Notes |
| --- | --- | --- |
| `rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/binding/diagnostic.rs zircon_editor/src/ui/asset_editor/diagnostics/mod.rs zircon_editor/src/ui/asset_editor/diagnostics/binding.rs zircon_editor/src/ui/asset_editor/session/lifecycle.rs zircon_editor/src/ui/asset_editor/binding/schema_projection.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/binding_semantics.rs` | Passed with no output. | Covers the M18 editor binding diagnostic projection files and the runtime diagnostic string code helper. |
| `git diff --check -- zircon_runtime/src/ui/template/asset/binding/diagnostic.rs zircon_editor/src/ui/asset_editor/diagnostics/mod.rs zircon_editor/src/ui/asset_editor/diagnostics/binding.rs zircon_editor/src/ui/asset_editor/session/lifecycle.rs zircon_editor/src/ui/asset_editor/binding/schema_projection.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/binding_semantics.rs` | Passed with only LF-to-CRLF notices. | No whitespace errors in the owned slice. |
| `cargo test -p zircon_editor --lib ui_asset_editor_projects_runtime_binding_diagnostic_and_schema_items --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never` | Passed, `1 passed; 0 failed; 888 filtered out`. | Proves runtime `invalid_value_kind` binding diagnostics project into `UiAssetEditorDiagnostic`, keep `target_binding_id`, and appear in inspector binding schema rows. |
| `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never` | Blocked before compilation. | Unrelated workspace lock drift: Cargo selected `web-sys 0.3.97` requiring `js-sys 0.3.97`, while `Cargo.lock` selected `js-sys 0.3.95`; active Cargo/Rustc jobs were present, so this M18 slice did not mutate `Cargo.lock`. |

M18 now has runtime-owned binding semantics and editor structured diagnostic/schema projection at focused scope. Broad editor/workspace green is still not claimed because the scoped editor check did not reach Rust compilation under the unrelated lockfile conflict.

### Plan Reconciliation: 2026-05-07

The stale unchecked M18 implementation slices above are now marked complete based on the accepted evidence already recorded in this plan and in `.codex/sessions/20260501-2047-ui-productization-full-milestones.md`: `zircon_runtime_interface/src/ui/template/asset/binding/expression.rs` / `target.rs`, `zircon_runtime/src/ui/template/asset/binding/validation.rs`, `zircon_runtime/src/ui/tests/asset_binding.rs`, editor binding diagnostic projection, and the M21 action-policy separation all exist. This reconciliation did not claim broad workspace green and did not touch M6 designer-tool source while the `zircon-ui-designer-m6` Cargo lane was active.

## Milestone 3: M5 Workspace Hot Reload And Conflict Flows

**Current status:** Focused accepted on 2026-05-07. Existing watcher/conflict infrastructure already covered clean reload, dirty external conflict preservation, reload/keep-local, diff snapshot, and stale import recovery. This pass added save-local-copy affordance/state/action routing and focused regression evidence. Broad workspace green remains unclaimed.

**Goal:** 补齐 workspace save/hot reload/conflict productization：external conflict、reload/keep-local/diff/save-copy flows、stale import diagnostics。

| Scope | Details |
| --- | --- |
| In scope | File change detection, document reload policy, conflict diagnostics, last-good preservation。 |
| Out of scope | Full workspace watcher rewrite if active watcher session owns it。 |
| Owner | `zircon_editor/src/ui/host/asset_editor_sessions`, `zircon_editor/src/ui/asset_editor/session`。 |

### Implementation Slices

- [x] Read active editor watcher/session notes before touching watcher files.
- [x] Define `ExternalConflict` state with source hash, local dirty flag, external/local source snapshots, diff snapshot, and last-good session preservation.
- [x] Add reload choices: reload external, keep local, open diff, save as copy.
- [x] Add stale import diagnostics when registered widget/style/resource imports change.
- [x] Add session tests for conflict transitions and last-good preview retention.

### Testing Stage

| Command | Expected |
| --- | --- |
| `cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-hot-reload-m5 --message-format short --color never` | Watcher/conflict tests pass if available. |
| `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-hot-reload-m5 --message-format short --color never` | Editor session tests pass. |
| `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-hot-reload-m5 --message-format short --color never` | Editor type-check passes. |

### Exit Evidence: 2026-05-07

| Command | Result | Notes |
| --- | --- | --- |
| `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-hot-reload-m5 --message-format short --color never` | Passed with existing runtime warnings. | Rerun after the initial editor test hit an incremental/intermediate runtime-interface DTO compile miss; current source and warmed target type-check. |
| `cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-hot-reload-m5 --message-format short --color never -- --nocapture --test-threads=1` | Passed, `5 passed; 0 failed; 1120 filtered out`. | Covers clean external reload, dirty conflict preservation, save-local-copy without resolving conflict, diff snapshot, reload/keep-local, and stale import recovery. |
| `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-hot-reload-m5 --message-format short --color never -- --nocapture --test-threads=1` | Passed, `205 passed; 0 failed; 920 filtered out`. | Confirms broader UI Asset Editor authoring/session paths still pass after conflict action projection changes. |
| `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-hot-reload-m5 --message-format short --color never` | Passed with existing runtime/editor warnings. | Confirms editor lib type-checks with save-copy façade, pane presentation state, and native action dispatch. |

M5 is closed at focused productization scope. Remaining resource-browser/runtime-loader/RHI media consumption work stays under M15/M16 follow-up, not M5 hot reload/conflict.

## Milestone 4: M6 Designer Canvas Tools

**Goal:** 把已 authored 的 `DesignerToolModeRow` 和 `DesignerDiagnosticOverlayPanel` 推进到行为闭环：slot resize 和 preview interact dispatch。

| Scope | Details |
| --- | --- |
| In scope | Tool mode state, overlay hit regions, slot resize transaction, preview interaction dispatch。 |
| Out of scope | Rendering/RHI, unrelated host chrome, full layout debugger。 |
| Owner | `zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml`, `zircon_editor/src/ui/asset_editor`。 |

### Implementation Slices

- [x] Revalidate authored shell nodes remain present and `.slint` fence remains zero.
- [x] Add designer tool state model: select, resize slot, preview interact.
- [x] Route overlay hit candidates into stable editor commands.
- [x] Add slot resize transaction with undo/redo and dirty/save behavior.
- [x] Add preview interact dispatch through existing component adapter/action paths.
- [x] Add bootstrap/projection tests plus command replay tests.

### Testing Stage

| Command | Expected |
| --- | --- |
| `cargo test -p zircon_editor --lib ui_asset_editor_bootstrap --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never -- --nocapture` | Authored shell/projected nodes still pass. |
| `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never` | Tool/resize/interact tests pass. |

### 2026-05-07 M6 Closure Evidence

M6 is focused-closed. `UiDesignerToolMode` now drives Select / Resize Slot / Preview Interact presentation state; slot resize applies width and height as a single undoable document transaction; Preview Interact maps a preview canvas node back through the shared preview projection to the authored document node and resolves the node binding route/action/payload/side-effect classification for UI presentation.

Accepted validation:

- `rustfmt` over the M6 designer tool/editor host/DTO/test slice passed after applying formatting only to touched M6 files.
- `git diff --check -- <M6 touched files>` passed with Git LF-to-CRLF notices only.
- `cargo test -p zircon_editor --lib designer_tools --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never -- --nocapture --test-threads=1` passed, `3 passed; 0 failed; 1125 filtered out`.
- `cargo test -p zircon_editor --lib ui_asset_editor_bootstrap --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never -- --nocapture --test-threads=1` passed, `9 passed; 0 failed; 1119 filtered out`.
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never -- --nocapture --test-threads=1` passed, `208 passed; 0 failed; 920 filtered out`.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never` passed with existing runtime/editor warnings.

## Milestone 5: M14 Localization

**Goal:** 记录已完成的 runtime localization foundation，并把 UI Asset Editor 内的 locale preview、dependency/extraction rows、diagnostics projection、external locale resolver / missing-key IO diagnostics 收束为 editor-facing runtime report。

| Scope | Details |
| --- | --- |
| Runtime | 已有 localized text ref model、dependency/candidate report、text direction metadata、compiler precondition validation、locale table key catalog resolver 和 M16 package manifest/report hookup。 |
| Editor | UI Asset Editor pane presentation now exposes locale preview options, selected preview locale/index, dependency rows, extraction rows, missing-table/missing-key rows, and localization diagnostic rows. |
| Avoid | Do not entangle with M15 media/resource refs except via shared dependency manifest extension points。 |

### Implementation Slices

- [x] Define `UiLocalizedTextRef` in runtime asset document model.
- [x] Add locale dependency collection and deterministic manifest entries.
- [x] Add structured localization diagnostics for invalid authored refs during compiler/package validation.
- [x] Add text direction metadata to the runtime report surface.
- [x] Add extraction report rows for authored literal text candidates.
- [x] Add docs and focused runtime tests for localized refs, dependencies, extraction candidates, and package rows.
- [x] Add editor locale preview selector/report rows and extraction UI projection through `UiAssetEditorPanePresentation`.
- [x] Add external locale table resolver/missing-key IO diagnostics in a dedicated resource/localization resolver milestone.

### Testing Stage

| Command | Expected |
| --- | --- |
| `cargo test -p zircon_runtime --lib asset_localization --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-localization-m14 --message-format short --color never` | Runtime localization tests pass. |
| `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-localization-m14 --message-format short --color never` | Editor locale preview/extraction tests pass. |

### Runtime Foundation Evidence: 2026-05-02

- `cargo test -p zircon_runtime --lib asset_localization --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never` passed, `5 passed; 0 failed`.
- `cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never` passed, `9 passed; 0 failed`, covering package rows beside resource/action metadata.
- `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never -- --nocapture` later passed, `112 passed; 0 failed; 546 filtered out`, after the M15 resource-table classifier support fix. Broad workspace green is still not claimed.

### Editor Productization Evidence: 2026-05-07

- `cargo test -p zircon_editor --lib action_localization_reports --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m21-m14-editor-reports --message-format short --color never -- --nocapture --test-threads=1` passed, `2 passed; 0 failed; 1138 filtered out`.
- `cargo test -p zircon_editor --lib runtime_reports --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m21-m14-editor-reports --message-format short --color never -- --nocapture --test-threads=1` passed, `1 passed; 0 failed; 1139 filtered out`.
- `cargo test -p zircon_editor --lib runtime_report_productization --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m21-m14-editor-reports --message-format short --color never -- --nocapture --test-threads=1` passed, `1 passed; 0 failed; 1139 filtered out`.
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m21-m14-editor-reports --message-format short --color never -- --nocapture --test-threads=1` passed, `219 passed; 0 failed; 921 filtered out`.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m21-m14-editor-reports --message-format short --color never` passed with existing runtime/editor warnings.
- 2026-05-07 resolver closure: `cargo test -p zircon_runtime --lib asset_localization --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m14-m15-resolver --message-format short --color never -- --nocapture --test-threads=1` passed, `7 passed; 0 failed; 942 filtered out`, covering locale table TOML key flattening plus missing table/key diagnostics.
- 2026-05-07 resolver closure: `cargo test -p zircon_editor --lib action_localization_reports --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m14-m15-resolver --message-format short --color never -- --nocapture --test-threads=1` passed, `3 passed; 0 failed; 1145 filtered out`, covering editor locale preview missing-table/missing-key pane rows and structured source/node diagnostics.
- Accepted scope: UI Asset Editor report projection for action-policy rows, capability explanation rows, locale preview/dependency/extraction/diagnostic rows, localization report diagnostics mapped to `UiAssetEditorDiagnostic` source path/node target, external locale table missing-table/missing-key diagnostics, and compiled resource dependency/diagnostic rows. Broad workspace green, resource browser, runtime loader backend, and graphics/RHI consumption remain unclaimed.

## Milestone 6: M15 Media/Font/Resource References

**Goal:** Record the accepted M15 media/font/resource ref typed-model gate and keep remaining productization gaps explicit.

| Scope | Details |
| --- | --- |
| Current owner | Foundation M1 gate completed by `.codex/sessions/archive/20260501-2033-ui-asset-resource-refs-m15-implementation.md`; runtime/package reconciliation completed in `.codex/sessions/20260501-2047-ui-productization-full-milestones.md`. |
| In this archive | Records accepted runtime/package scope, minimal editor dependency projection, and future resource-loading gaps; no need to block other sessions on active M15 source ownership after archival. |
| Dependency | Provides the typed model and collector that M16 back-half consumes through one compiled dependency manifest surface. Future resource UX/loading work must extend that surface instead of replacing it. |

### Acceptance Criteria

- [x] Runtime asset model can declare font/media/resource refs.
- [x] Compiler collects explicit, token, node, style, component, imported-widget, and imported-style resource refs into `UiCompiledDocument`.
- [x] M16 package manifest consumes compiled resource dependencies as deterministic dependency rows.
- [x] M12 fingerprints/cache keys include resource dependency changes.
- [x] Editor shows minimal read-only resource dependency/diagnostic rows through the UI Asset Editor runtime-report presentation.
- [x] Runtime resolver reports resource file-existence diagnostics and editor projects them when resolver roots are configured.
- [x] No full resource browser UX in M15 V1.

### Accepted Evidence

- Targeted `rustfmt --edition 2021 --check` over runtime resource refs, asset document/template exports, runtime resource-ref tests, test module registration, and editor theme constructor passed with no output.
- `asset_resource_refs`: `7 passed; 0 failed`, covering typed imports roundtrip, validation failures, fallback mismatch, and legacy resource-kind inference.
- `cargo check -p zircon_editor --lib` passed with existing unrelated runtime graphics warnings, covering the `UiAssetImports.resources` constructor impact.
- Reconciliation rerun: `cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed, `9 passed; 0 failed`, covering collector and resource fingerprint stability.
- `cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed, `9 passed; 0 failed`, covering resource dependency cache misses and the shape-precondition-before-resource-fingerprint regression.
- `cargo test -p zircon_runtime --lib asset_invalidation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed, `6 passed; 0 failed`, covering `ResourceDependency` change/stage/impact.
- `cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed, `9 passed; 0 failed`, covering compiled resource dependency manifest rows and the contract-precondition-before-resource-fingerprint regression.
- 2026-05-07 editor accessor pass: `rustfmt --edition 2021 --check` passed over the M15 editor session/presentation/DTO/test slice; `cargo test -p zircon_editor --lib resource_dependency_view --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m15-resource-ux --message-format short --color never -- --nocapture --test-threads=1` passed, `4 passed; 0 failed; 1136 filtered out`; `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m15-resource-ux --message-format short --color never` passed with existing runtime/editor warnings.
- 2026-05-07 resolver closure: `cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m14-m15-resolver --message-format short --color never -- --nocapture --test-threads=1` passed, `12 passed; 0 failed; 937 filtered out`, covering `UiResourcePathResolver` primary and placeholder fallback missing-file diagnostics.
- 2026-05-07 resolver closure: `cargo test -p zircon_editor --lib resource_dependency_view --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m14-m15-resolver --message-format short --color never -- --nocapture --test-threads=1` passed, `5 passed; 0 failed; 1143 filtered out`, covering editor resource pane missing-file rows and refresh after files appear.
- 2026-05-07 resolver closure: `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m14-m15-resolver --message-format short --color never -- --nocapture --test-threads=1` passed, `221 passed; 0 failed; 927 filtered out`; `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m14-m15-resolver --message-format short --color never` passed with existing runtime/editor warnings.
- Accepted editor scope: `UiAssetEditorSession::resource_dependencies()` and `UiAssetEditorSession::resource_diagnostics()` expose compiler-owned dependency/diagnostic DTOs plus resolver-owned file-existence diagnostics, refresh after valid preview compiles/source edits and resolver/file changes, and clear on resource compile failure. Pane presentation rows still come from the runtime-report projection.
- 2026-05-08 asset-browser reference graph slice: `zircon_runtime::asset::assets::ui_asset_references(...)` now consumes `collect_document_resource_dependencies(...)` for document-local resource dependencies and fallback URIs, dedupes by asset locator, and strips component/style import labels before handing references to the editor asset graph. Focused tests were added in `zircon_runtime/src/asset/tests/assets/ui.rs`. `rustfmt --edition 2021 --check` passed over the touched runtime asset files, and `git diff --check -- zircon_runtime/src/asset/assets/ui.rs zircon_runtime/src/asset/tests/assets/ui.rs` passed with only LF-to-CRLF normalization notices.
- 2026-05-08 Cargo validation note: `cargo test -p zircon_runtime --lib ui_asset_direct_references --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m15-resource-browser --message-format short --color never -- --nocapture --test-threads=1` is blocked before executing the new tests by active incremental layout field drift in `zircon_runtime/src/ui/surface/surface.rs` and `zircon_runtime/src/ui/tests/surface_dirty_domains.rs` (`UiSurfaceRebuildDebugStats`/`UiSurfaceRebuildReport` field mismatch). This blocker is outside the M15 asset reference graph slice and is not counted as M15 completion evidence.
- Not accepted yet: broader resource browser UX, watcher/hot reload, runtime loader backend, and graphics/RHI resource consumption.

### Remaining Productization Boundary

- Broader resource browser UX, watcher/hot reload, runtime loader backend, graphics/RHI resource consumption, and richer editor authoring UX remain outside the runtime/package gate, resolver-backed read-only editor view, and 2026-05-08 asset-browser direct-reference graph slice.
- Future resource UX/loading work must consume the existing compiled dependency manifest surface; do not invent a second resource scan.

## Milestone 7: M16 Back-Half Compiled Artifact Pipeline

**Goal:** 记录已完成的 M16 V1 与 back-half package pipeline，并固定后续只剩跨进程 cache store、resolver/runtime-loader/editor UX 与 broad validation。

| Scope | Details |
| --- | --- |
| Completed V1 | `zircon_runtime::ui::template::asset::compiler::package` owns `UiCompiledAssetHeader`, dependency manifest, runtime/editor profile, and validation report. |
| Current plan | `docs/superpowers/plans/2026-05-01-ui-compiled-artifact-package-validation.md` plus `.codex/plans/M16 UI Compiled Artifact And Package Validation Implementation Plan.md` history. |
| Back-half boundary | Binary artifact envelope, cache record, package manifest writer/importer pipeline are implemented; do not redesign V1 report rules in parallel. |
| Dependency | M15 typed resource model and collector have landed; M16 back-half consumes compiled dependencies through `UiCompiledAssetDependencyManifest.resource_dependencies`. |

### Acceptance Criteria

- [x] Define a deterministic binary compiled artifact envelope and schema version, separate from the V1 metadata report.
- [x] Persist compiled cache records only after reusing M12 cache key and invalidation fingerprint inputs.
- [x] Add package manifest writer/importer that consumes widget/style/resource dependencies from one manifest surface.
- [x] Keep existing M10/M12/M13/M16 V1 focused tests passing while adding back-half coverage.
- [x] Archive records the back-half as separate evidence; full completion still excludes resolver/runtime-loader/editor resource UX and broad workspace green.

### Accepted Evidence

- `cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed, `9 passed; 0 failed`, covering artifact envelope roundtrip, cache record reuse of `UiCompileCacheKey`/`UiInvalidationSnapshot`, package manifest writer/importer, resource dependency rows, profile stripping, compiler error propagation, cache-input stability, and contract-precondition ordering.
- `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed, `86 passed; 0 failed`.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with unrelated graphics/plugin warnings.
- Artifact format accepted here is binary magic `ZRUIA016`, little-endian `u32` artifact schema version, little-endian `u64` payload length, and UTF-8 TOML payload for `UiCompiledAssetArtifact { report, compiled }`.

## Milestone 8: M21 Action Safety And Host Policy

**Goal:** 记录已完成的 runtime action side-effect classification/host policy foundation，并把 UI Asset Editor 内的 action-policy report、capability explanation、runtime/editor host enforcement rows 与 unsafe action authoring guidance 收束为 editor-facing runtime report。

| Scope | Details |
| --- | --- |
| Runtime | 已有 action side-effect classification、runtime/editor host policy defaults、package validation report hookup。 |
| Editor | UI Asset Editor pane presentation now exposes action-policy report rows, capability explanation rows, runtime-default/editor-authoring enforcement rows, and unsafe action guidance rows from runtime-owned host policy validation. |
| Dependency | Should consume M18 binding/action schema, not invent parallel action paths。 |

### Implementation Slices

- [x] Define side-effect classes: local UI, editor mutation, asset IO, scene mutation, external process, network.
- [x] Add host policy schema and runtime/editor profile validation.
- [x] Route action policy report generation through runtime-owned package validation.
- [x] Add focused runtime tests for rejected side effects under runtime/editor profiles.
- [x] Add editor action-policy report rows for unsafe/unavailable actions through `UiAssetEditorPanePresentation`.
- [x] Add capability explanation rows alongside the editor runtime report projection.
- [x] Add broader host enforcement presentation and richer unsafe action authoring guidance through runtime-report pane rows.

### Runtime Foundation Evidence: 2026-05-02

- `cargo test -p zircon_runtime --lib asset_action_policy --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never` passed, `3 passed; 0 failed`.
- `cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m21-m14 --message-format short --color never` passed, `9 passed; 0 failed`.
- `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-binding-m18 --message-format short --color never -- --nocapture` later passed, `112 passed; 0 failed; 546 filtered out`, after the M15 resource-table classifier support fix. Broad workspace green is still not claimed.

### Editor Productization Evidence: 2026-05-07

- Same `D:\cargo-targets\zircon-ui-m21-m14-editor-reports` editor validation as Milestone 5 passed: `action_localization_reports` 2 passed, `runtime_reports` 1 passed, `runtime_report_productization` 1 passed, broader `ui_asset_editor` 219 passed, and `cargo check -p zircon_editor --lib` passed with existing warnings.
- Accepted scope: runtime-owned `UiActionHostPolicy::editor_authoring()` diagnostics are projected into editor action-policy rows; allowed/blocked side-effect classes are projected into capability explanation rows. The follow-up host enforcement slice adds runtime-default/editor-authoring profile rows plus per-binding unsafe action guidance, without replacing the package validation policy owner.

### Host Enforcement Presentation Evidence: 2026-05-07

- `rustfmt --edition 2021 --check` passed for the M21 editor runtime-report projection files, native/Slint host DTO conversion files, and focused runtime-report tests.
- `cargo test -p zircon_editor --lib runtime_reports --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m14-m15-resolver --message-format short --color never -- --nocapture --test-threads=1` passed, `1 passed; 0 failed; 1150 filtered out`.
- `cargo test -p zircon_editor --lib runtime_report_productization --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m14-m15-resolver --message-format short --color never -- --nocapture --test-threads=1` passed, `1 passed; 0 failed; 1150 filtered out`.
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m14-m15-resolver --message-format short --color never -- --nocapture --test-threads=1` passed, `221 passed; 0 failed; 930 filtered out`.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m14-m15-resolver --message-format short --color never` passed with existing runtime/editor warnings.
- `git diff --check -- <M21 host enforcement touched files/docs>` passed with only existing LF-to-CRLF normalization notices.

## Milestone 9: M22 Runtime/Editor Dual-Host Parity

**Current status:** Focused accepted on 2026-05-07. `dual_host_parity.rs` now covers representative `.ui.toml` documents through `UiSurface` layout, editor host model projection, Slint/native host projection, and Component Showcase material-event dispatch. Broad workspace green and Graphics/RHI rendering consumption remain unclaimed.

**Goal:** 验证同一 `.ui.toml` 在 runtime extract 和 editor projection 下 layout/event semantics 对齐。

| Scope | Details |
| --- | --- |
| In scope | Parity fixtures, projection vs runtime extract comparison, event dispatch equivalence。 |
| Out of scope | Graphics/RHI rendering consumption and showcase-specific demo behavior。 |

### Implementation Slices

- [x] Pick representative assets: generic host window, UI Asset Editor shell, Component Showcase, and Workbench window templates.
- [x] Build parity fixture runner that constructs one runtime `UiSurface`, then projects the same frame through editor host model and Slint/native host projection.
- [x] Compare layout-critical frames, stable template attributes, style tokens, bindings, component ids, registered route ids, collection slots, virtualization metadata, and world-space metadata.
- [x] Add event dispatch parity tests for value changed, commit, action route, adapter mutation result, checkbox state, and button commit.
- [x] Record intentional Component Showcase divergences explicitly: runtime demo state overlays `selected`, `selection_state`, `value_text`, validation rows, and generated `collection_items` on top of the raw runtime surface; stable authored attributes still compare exactly after filtering those state overlay fields.

### Focused Parity Evidence: 2026-05-07

| Command / Signal | Result | Notes |
| --- | --- | --- |
| `rustfmt --edition 2021 --check zircon_editor\src\tests\host\template_runtime\dual_host_parity.rs zircon_editor\src\tests\host\template_runtime\mod.rs` | Passed. | Covers the new parity test module and template-runtime test registration. |
| `git diff --check -- zircon_editor/src/tests/host/template_runtime/dual_host_parity.rs zircon_editor/src/tests/host/template_runtime/mod.rs` | Passed with LF-to-CRLF notices only. | No whitespace errors in the owned slice. |
| `cargo test -p zircon_editor --lib dual_host_parity --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m22-parity --message-format short --color never -- --nocapture --test-threads=1` | Passed, `2 passed; 0 failed; 1140 filtered out`. | Covers runtime surface vs host/native projection layout/attribute/style/binding parity and Material Component Showcase event dispatch, including the authored UI Asset Editor shell in the representative document loop. |
| `cargo test -p zircon_editor --lib template_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m22-parity --message-format short --color never -- --nocapture --test-threads=1` | Passed, `44 passed; 0 failed; 1098 filtered out`. | Broader template runtime regression gate after adding the parity fixture. |
| `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m22-parity --message-format short --color never` | Passed. | Existing runtime/editor warnings remain; no workspace green claim. |

## Milestone 10: M24 Emergency Shell Fallback

**Goal:** 把 authored `EmergencyShellPanel` 推进到 last-valid/emergency shell fallback state machine 和 Reload/Revert/Open Asset Browser 行为闭环。

**Current status:** Focused accepted on 2026-05-07. The authored `EmergencyShellPanel` now has session state, last-valid fallback preservation, pane/host DTO projection, and recovery action routing. Broad workspace green remains unclaimed.

| Scope | Details |
| --- | --- |
| In scope | Last-valid compile cache, emergency state, recovery actions, user-visible diagnostics。 |
| Out of scope | New chrome visual redesign, resource browser feature expansion。 |

### Implementation Slices

- [x] Define UI Asset Editor shell state: valid, stale, invalid, emergency.
- [x] Preserve last-valid projection when current source fails.
- [x] Display emergency shell with exact diagnostic summary and recovery actions.
- [x] Implement Reload/Revert/Open Asset Browser actions through existing editor command paths.
- [x] Add tests for invalid source -> emergency -> recover/revert flows.

### Exit Evidence: 2026-05-07

| Command / Signal | Result | Notes |
| --- | --- | --- |
| `rustfmt --edition 2021 --check <M24 editor/session/host/test files>` | Passed. | Covered shell state, emergency session state, lifecycle/session owner files, host refresh/action routing, Slint host DTO conversion, and focused tests. |
| `git diff --check -- <M24 editor/session/host/test/session slice>` | Passed with LF-to-CRLF notices only. | No whitespace errors in the owned slice. |
| Source guard | Passed. | Emergency reload/revert/open-asset-browser action IDs route through Slint host action dispatch; reload/keep-local/diff/revert flow through existing `EditorUiHost` session manager methods. |
| `cargo test -p zircon_editor --lib emergency_shell --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never -- --nocapture --test-threads=1` | Passed, `4 passed; 0 failed; 1128 filtered out`. | Covers invalid source -> Emergency, last-valid preview retention, manager-driven revert, and Slint/native emergency action routing. |
| `cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never -- --nocapture --test-threads=1` | Passed, `6 passed; 0 failed; 1126 filtered out`. | Covers clean reload, dirty conflict preservation, diff snapshot, reload/keep-local resolution, save-local-copy, stale import recovery, and emergency revert through the manager facade. |
| `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never -- --nocapture --test-threads=1` | Passed, `211 passed; 0 failed; 921 filtered out`. | Broader UI Asset Editor authoring/session regression gate after emergency recovery routing. |
| `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never` | Passed. | Existing runtime/editor warning noise remains. |

## Milestone 11: Remaining Roadmap Buckets

| Milestone | Archive Status | Proposed Next Plan |
| --- | --- | --- |
| M2 pseudo-state style application | Partial | Fold into style/invalidation plan after M18 or M22. |
| M3 descriptor registration behavior | Mostly complete but not closed | Audit after M16 package report consumes descriptor registry. |
| M4 structured diagnostics | Partial | Shared diagnostics plan spanning M10/M16/M18/M21. |
| M8 accessibility/input authoring | Partial | Separate accessibility/input authoring plan after M18 binding schema. |
| M9 journal adapter/replay artifact | Partial to mostly complete | 2026-05-08 bug-report replay artifact export and versioned command journal adapter are code-landed; remaining work is durable journal persistence, host event capture/normalization, bug-report attachment routing, and Cargo validation after active runtime blockers clear. |
| M11 theme governance | Partial | Separate theme/token governance plan after localization/resource refs. |
| M17 collaboration diagnostics | Partial | Defer until stable package/report and source outline diagnostics exist. |
| M19 transitions/animation | Missing | Separate animation plan; not needed for immediate productization. |
| M20 layout debug packet/debugger | Missing | Pair with M22 parity and M6 canvas overlay diagnostics. |

## Recommended Execution Order

| Order | Work | Reason |
| --- | --- | --- |
| 1 | Runtime-interface UI DTO cutover continuation | Lower shared DTO identity must stabilize before action inspector, editor operation validation, and broad parity tests. Continue `docs/superpowers/plans/2026-05-02-ui-runtime-interface-big-cutover.md` Milestones 2-5, coordinated with active renderer/editor-operation notes. M18 editor binding diagnostic/schema projection is already focused-accepted. |
| 2 | M18 editor binding diagnostic projection | Accepted at focused scope on 2026-05-02: editor structured diagnostics and inspector schema rows consume runtime/interface `UiBindingDiagnostic` values, preserve `target_binding_id`, and passed `ui_asset_editor_projects_runtime_binding_diagnostic_and_schema_items`. No remaining source slice is queued here; only broad editor/workspace green remains unclaimed. |
| 3 | M21/M14 editor productization | Accepted on 2026-05-07 for UI Asset Editor runtime-report projection: action policy, capability explanations, runtime/editor host enforcement rows, unsafe action authoring guidance, locale preview/dependency/extraction/diagnostics, external missing-key diagnostics, and resource dependency/diagnostic rows. |
| 4 | M5/M24 editor recovery and emergency shell | Hot reload/conflict and last-valid/emergency recovery are user-facing editor completion and should reuse stable editor/runtime DTO/report surfaces. |
| 5 | M6 designer canvas tools | Slot resize and preview-interact behavior should build on stable recovery/session command paths. |
| 6 | M15 resource UX/loading | Resolver-backed editor dependency view is accepted through session typed accessors, runtime-report rows, and file-existence diagnostics. The 2026-05-08 asset-browser direct-reference graph slice now feeds collected UI resource dependencies into `ui_asset_references(...)`, with Cargo execution blocked only by active layout field drift. Remaining watcher integration, runtime loader backend, and broader resource browser must consume the existing compiled dependency manifest; graphics/RHI consumption waits for renderer/plugin lanes. |
| 7 | M22 runtime/editor dual-host parity | Focused representative parity is accepted: generic host window, UI Asset Editor shell, Component Showcase, and Workbench now compare shared layout/projection/event semantics. Future work is only broader golden coverage or render/RHI parity. |
| 8 | Broad validation triage expansion | Run workspace/package validation only after active graphics/plugin/runtime-interface blockers are resolved or when the user explicitly asks for broad failure evidence. |
| 9 | M9 persistence/host capture follow-up | Bug-report artifact export and command journal adapter are now landed; next M9 work should add durable journal persistence, host event capture/normalization, and bug-report attachment routing without touching active layout/surface or scene/ECS ownership. |
| 10 | Remaining roadmap buckets | M2/M8/M11/M17/M19/M20 as independent follow-up plans. |

## M9 Replay Artifact Slice: 2026-05-08

**Scope:** close the bug-report replay artifact export and internal command journal adapter gaps without entering active layout/surface or scene/ECS lanes. This slice keeps executable command authority in the existing UI Asset Editor undo/replay structures, adds a redacted diagnostic export for applied undo history, and adds a versioned command journal replay adapter.

### Implementation Slices

- [x] Expose applied undo-stack replay records in oldest-to-newest order without mutating undo/redo state.
- [x] Add versioned `UiAssetEditorBugReportReplayArtifact` DTOs for route, selection, diagnostics, source summaries, replay records, command summaries, and external effects.
- [x] Redact raw source bodies by storing only byte length, line count, and deterministic stable hash.
- [x] Summarize document commands with structural metadata only: ids, selectors, key names, counts, and references rather than raw style/prop values.
- [x] Add regression coverage proving serialized artifact output does not leak raw TOML or generated external asset source.
- [x] Add a versioned command journal DTO for source/tree edit records and undo/redo external effects.
- [x] Replay command journal entries through existing `UiAssetEditorCommand` and undo external-effect paths after schema/sequence preflight.
- [ ] Add durable journal storage and bug-report attachment routing.
- [ ] Add host event capture/normalization that records user operations as command journal entries.

### Exit Evidence

| Command / Signal | Result | Notes |
| --- | --- | --- |
| `rustfmt --edition 2021 --check zircon_editor/src/ui/asset_editor/undo_stack.rs zircon_editor/src/ui/asset_editor/session/mod.rs zircon_editor/src/ui/asset_editor/session/replay_artifact.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/tests/editing/ui_asset_replay.rs` | Passed. | Covers the new replay artifact module, undo-stack record accessor, exports, and regression test. |
| `git diff --check -- zircon_editor/src/ui/asset_editor/undo_stack.rs zircon_editor/src/ui/asset_editor/session/mod.rs zircon_editor/src/ui/asset_editor/session/replay_artifact.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/tests/editing/ui_asset_replay.rs` | Passed with LF-to-CRLF notices only. | No whitespace errors in the owned slice. |
| `cargo test -p zircon_editor --lib ui_asset_editor_session_exports_sanitized_bug_report_replay_artifact --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m9-replay-artifact --message-format short --color never -- --nocapture --test-threads=1` | Blocked before editor test execution. | First run timed out during cold dependency compilation; warmed rerun reached the active layout/surface blocker: unresolved `compute_incremental_layout_tree` import in `zircon_runtime/src/ui/surface/surface.rs`. |
| `rustfmt --edition 2021 --check zircon_editor/src/ui/asset_editor/command.rs zircon_editor/src/ui/asset_editor/undo_stack.rs zircon_editor/src/ui/asset_editor/session/mod.rs zircon_editor/src/ui/asset_editor/session/journal.rs zircon_editor/src/ui/asset_editor/session/replay_artifact.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/tests/editing/ui_asset_replay.rs` | Passed. | Covers the command journal module, serde-enabled command/replay DTOs, exports, artifact module, and regression tests. |
| `git diff --check -- zircon_editor/src/ui/asset_editor/command.rs zircon_editor/src/ui/asset_editor/undo_stack.rs zircon_editor/src/ui/asset_editor/session/mod.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/tests/editing/ui_asset_replay.rs` | Passed with LF-to-CRLF notices only. | No whitespace errors in the tracked owned slice. |
| `cargo test -p zircon_editor --lib ui_asset_editor_command_journal --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m9-replay-artifact --message-format short --color never -- --nocapture --test-threads=1` | Blocked before editor test execution. | `zircon_runtime` fails first in active scene/ECS code: missing `crate::scene::ecs`, missing `SystemStage` and `Schedule` exports from `scene::components`, plus follow-on type inference errors. |

This slice closes the M9 bug-report replay artifact export and internal command journal adapter gaps. The full milestone stays open until durable journal persistence, host event capture/normalization, bug-report attachment routing, and unblocked focused Cargo validation exist.

## Archive Update Rules

- Do not mark a milestone `complete` without exact code owner, focused tests, docs, and validation command evidence.
- Keep `Partial to mostly complete` when runtime foundation exists but editor UX/package/productization remains.
- If broad workspace validation fails outside UI, record blocker owner and active session note instead of treating it as UI failure.
- Every completed slice must update `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md`.
- Every code-bearing slice must update the relevant `docs/ui-and-layout/*.md` module document with related code, implementation files, plan sources, and tests.
- Active `.codex/sessions/*.md` notes must be retired or archived when their slice closes.
