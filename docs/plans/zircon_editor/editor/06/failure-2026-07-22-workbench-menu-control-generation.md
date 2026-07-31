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
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/asset_creation_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/toolbar_layout/priority.rs
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

Open state: `待 Editor06/Editor09 建立 generation owner 并回传规模、像素与行为等价证据`。
