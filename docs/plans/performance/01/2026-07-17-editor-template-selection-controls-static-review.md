---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_control_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_control_geometry
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls_tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - selection identity/metrics/geometry/style/state/toggle/mark/pixel tests
  - current-source Windows Cargo running
  - 1/100/10000 selection selector/theme/metrics/resource/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template selection controls逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_selection_control_geometry*`、`template_selection_controls*`与tests共 **26/26** 个Rust文件、**1,264** 行已逐文件阅读。覆盖checkbox/radio/toggle identity、metrics/geometry、style selector、label/tick、surface/thumb commands以及完整state/style/mark/toggle/pixel tests。当前源Cargo、规模计数与产品像素trace未完成，因此仍留在`pending.md`。

## P0：同一selection control重复完整selector与metrics projection

Checkbox分别为background、border和label调用3次完整`select_workbench_selection_control_style`；radio为background、border、label以及checked accent调用3至4次；toggle为label、track、border和thumb调用4次。每次selector都重新解析state并读取全局palette，随后计算完整surface/border/thumb/accent/text/label结构，只取其中一个字段。

Geometry同样分散读取全局metrics。Checkbox/radio的leading mark、label rect、default gap与label text各自取metrics，radio默认dot再取一次；toggle的track、outer layout、gap、label text与thumb合计可达约5次。Checked checkbox还每paint解析/复制checkmark asset，fallback展开3个quad，回链PERF-MVP-179/181。

PERF-MVP-195要求changed-node入口构建一次`SelectionControlPaintSpec`：typed checkbox/radio/toggle kind、resolved state/style、单一theme/metrics/text snapshot、label、mark/track/thumb geometry与resource handle。各command helper只借用spec；stable generation由PERF-MVP-178复用compiled segment。不能仅在某个leaf缓存style而留下其他helper重复读theme。

## 动态验收

在1/100/10,000 checkbox/radio/toggle、checked/selected/hover/pressed/focused/disabled/loading与declared-style控件上记录identity、selector/state、palette/metrics reads、label builds、resource resolve/raster/copy、fallback segments与commands。Changed node要求selector/style/label各<=1、theme/metrics acquisition各<=1；stable generation以上计数及command build均为0。保持现有26文件覆盖的state priority、declared colors/metrics、focus border、mark/track/thumb geometry/order与GPU/Softbuffer pixels parity。
