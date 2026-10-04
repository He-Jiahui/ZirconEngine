---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: workbench-menu-control-generation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/06
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/asset/type_registry/registry.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/asset_creation_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/popup_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/control_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
  - zircon_editor/src/ui/workbench/reference/template_surface.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/toolbar_layout/priority.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_window_menus/asset_creation.rs
tests:
  - tools.tests.test_editor06_workbench_toolbar_priority_contract
  - template/control/resize/action scale matrix
---

# Editor06：Workbench menu/control generation

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-560 workbench menu/control generation
- 修复责任计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 交接原因：workbench menu action table 与 control slot 的 generation owner 由 Editor06 联动 Editor09 收敛。

## 失败现象与复现证据

PERF-MVP-560确认每次layout重建asset creation labels/count/map/set与全部String，单action点击又重建整map再remove。toolbar priority原为约39个control各自全tree scan，本轮已用单次借用HashMap index止损，合同1/1；但稳定generation仍缺compiled action/control slot owner。

## 最低共享层根因

稳定 generation 缺少统一的 compiled menu action table 与 control slot owner，layout 和 click 仍会重建可复用的映射与字符串数据。

## 架构修复验收

Editor06联动Editor09按template+asset-type generation发布稳定menu action table、UiValue数组与control slots；layout只应用breakpoint/visibility delta，click按compiled action O(1)取得typed request。1/100/10k templates/controls/nodes、1k resize、1M click记录String/map/tree scan/layout dirty/p95；stable menu rebuild=0、click rebuild=0，collision/safe label/folder/menu height与F4像素/行为等价。

## 禁止临时方案

不得以全局可漂移 String cache 或 compat lookup 保留第二份 truth，也不得用弱化规模矩阵替代 generation-owned 证据。

## 修复结果与回传

Open state: `asset generation、shortcut signature 与主菜单尺寸复用已有源码修复；真实点击仍解析/重写整份菜单行，宿主投影仍复制菜单行。原始规模、性能、F4 与向上验收未完成`。下层 immutable typed projection 依赖已有的 [EditorUI05 failure](../../editor_ui/05/failure-2026-07-17-template-projection-deep-copy-and-cache-generation.md)；不得用 typed request 循环代替真实菜单点击验收。

## 2026-09-19 受管静态合同回执

- 受管 ticket：`6c08565bfa614205bffd9f925395b72b`；copy/run：`e645e49a7ed1418baa93d941c5361a12` / `6c08565bfa614205bffd9f925395b72b`；结果：`passed`，exit code `0`。
- 精确命令实际执行 `rustfmt +1.94.1 --check`（11 个 Rust 路径）及 `python -m unittest tools.tests.test_editor06_workbench_toolbar_priority_contract -v`；5/5 测试通过，末尾标记为 `EDITOR06_WORKBENCH_MENU_GENERATION_CONTRACT_PASS`，`CHECKED_PATHS=13`。
- 当前源码 manifest：`3d658cca72be71e4a26bce1a89e9bfe2aa74843b2060cff53ed9c587ae219834`；受管回执已保留原始 stdout/stderr 与 cleanup 记录。
- 本回执仅证明当前源码静态合同（immutable generation/indexed action、stable control-slot projection、persistable workbench slot boundary）；未证明 Windows `zircon_editor` Cargo/template-control/resize/action 矩阵、1/100/10000 与 1k/1M 性能门槛、上行 F4 等价、独立 Critical/Important/Moderate 复审、`failure return` 或 closeout。外部 `E:/Git/zr_vm` dirty blocker 仍保留。
- Failure lifecycle 继续保持 `open`；不得将此 partial/static receipt 复用为最终修复或关闭证据。

## 2026-09-27 stable-layout shortcut repair

Session `failure-roll-01a0df1a-editor06-menu-shortcut-r2` adopted this record, the menu bridge,
and its mounted Rust behavior test from archived/cancelled owners through coordinator transfer
`01b929dd61cda7e2daf5cb6213858178d452d5a7067991abb5f31db8bc78d9b4`. The current
source chain is linked above; the Editor09 registry and workbench template-surface owners are
related consumers, not paths claimed by this Session.

The stable menu path previously formatted up to three shortcut `String`s before comparing the
unchanged menu generation and shortcuts. The bridge now compares borrowed `EditorKeyChord` values
against the cached typed signature first. Only a changed menu clones those chords; it formats each
shortcut directly into its menu item without an intermediate display string. The existing mounted
shortcut-override test covers stable reuse, removed shortcuts, changed chords, and displayed menu
parity, but has not executed in a fresh managed Rust run. The exact edited source hash is
`50b587d0c95256f284d5ecf8cb34846cd31ce15e4e6a4eb6e33909099defebe7`.

Local Rust 1.94.1 scoped `rustfmt --check`, scoped `git diff --check`, and the five-test Python
workbench toolbar source-contract module passed. These are static/format results only. Managed
`zircon_editor` Cargo behavior, the original 1/100/10k template/control/node scale matrix,
1k resize and 1M typed action dispatch, allocation/scan/layout-dirty/p95 counters, F4 product
parity, independent C/I/M review, canonical return, and closeout remain pending. No dynamic pass
or menu-wide zero-allocation claim is made by this partial repair.

### Managed static receipt for the shortcut repair

- Session: `failure-roll-01a0df1a-editor06-menu-shortcut-r2`; frozen snapshot: `4104`.
- Ticket: `8c1bba80060b4e2b83aa9f174b3d7d90`; submitted source-manifest hash:
  `7aaff324e59dff2f8e490e972073c2845c8c470889d0c4d62f3dc6cf590c28ee`.
- The managed copy ran `python -B -m unittest tools.tests.test_editor06_workbench_toolbar_priority_contract -v`;
  terminal status `passed`, exit code `0`, and all five source-contract tests executed. The
  retained job/run ID is `be3adfe8b1b14031adc454aaf19880cd` /
  `8c1bba80060b4e2b83aa9f174b3d7d90`; the coordinator recorded cleanup.
- This ticket binds the menu source hash above, but tests source structure only. It does not
  execute the mounted Rust shortcut test, original scale/performance gates, F4 product parity,
  or independent review. The failure remains `open` with Session `waiting_validation`.

### 2026-09-27 typed action scale regression preparation

The mounted 10,000-template / 1,000-resize test previously performed one million action-ID
lookups but constructed only one typed menu request. The regression now covers 1, 100, and 10,000
templates with 1, 100, and 1,000 layout recomputes respectively. At the largest size it traverses
the action check and typed request path one million times, checking asset type, template ID,
target folder, and unchanged menu publish count. This is a prepared Rust regression, not an
executed managed Cargo result. Control/node scale, allocation/scan/layout-dirty/p95 measurements,
F4 parity, and source-matched dynamic acceptance remain open.

### 2026-09-27 compiled action fixture and real resize correction

The operation-dispatch regression still sent the obsolete literal action ID
`menu.item.create_u_i_layout`; the current generation publishes
`menu.item.asset_create.{generation}.{ordinal}`. The test now finds the registered
`ui_asset.layout` entry in the runtime's creation-menu generation, verifies that the bridge
recognizes its compiled action, and dispatches that action before checking the operation journal.
The scale regression now cycles through 1,024x768, 640x480, and 900x620 shell sizes, checking
that the retained menu frame remains positive and bounded by each shell while the publish count
stays at one. Its largest case performs 1,000 actual size changes and one million typed requests.
These changes repair the acceptance fixture; they have not yet executed in a managed Rust run.
Snapshot `4150` predates these test changes and must not be used for their dynamic acceptance.

### 2026-09-27 compiled menu extent and remaining click cost

Independent source review found that opening the main menu used the generic menu extent path,
cloning and measuring all rows and replacing the compiled height cap with the entire shell height.
The main-menu open path now reuses the current generation's row count, cached desired width, and
the existing toolbar-available-height calculation. Other menus retain their current generic path.
The resize regression checks the original fixed width/height constraints and expected content
height; a clipped visible frame alone cannot prove that the authored extent fits the available area.
Coordinator transfer `64eb5042e43be20db0491351cdfdf1169dcbbc45f4e28cfb51d6822ab3850144`
extended the same Session to `window_menu_state.rs` and `popup_state.rs`; the latter is unchanged.

The actual selected-item dispatch still normalizes and rewrites the complete menu row array in
`popup_state.rs`. Its state refresh also rebuilds host models whose `Vec<String>` menu rows are
copied in the lower template adapter. One million typed requests in the prepared scale regression
do not exercise one million real menu clicks. That shared production path, the original performance
measurements and F4 acceptance remain unresolved; this scoped extent repair cannot close the failure.
Snapshot `4166` predates the extent repair. No new source-matched managed Rust pass is recorded.
