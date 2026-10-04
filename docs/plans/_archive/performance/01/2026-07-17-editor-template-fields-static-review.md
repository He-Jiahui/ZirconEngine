---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_field_stepper.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_field_stepper
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - field identity/geometry/style/search/stepper/pixel tests
  - current-source Windows Cargo running
  - 1/100/10000 field classification/theme/metrics/allocation trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template fields逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_fields*`、`template_field_stepper*`与`template_fields_tests/**`共 **21/21** 个Rust文件、**1,126** 行已逐文件阅读。覆盖field identity、search/stepper分类、geometry、surface/state layer、style selector、text/icon command与现有identity/geometry/style/search/stepper/pixel tests。当前源Cargo正在运行，规模trace与像素验收未完成，因此仍留在`pending.md`。

## P0：高频输入字段重复字符串分类与全局快照读取

`is_search_field`依次检查control id、interaction action、role、component role与variant；每个候选都先`to_ascii_lowercase()`再`contains("search")`，一次调用最多产生5个owned lowercase字符串。该函数在identity、paint rect、search glyph、text inset和placeholder判断等路径重复执行，所以一个search field的单次paint会重复扫描相同字段并分配字符串。

Field入口虽然只计算一次`field_style`，但placeholder判断会先构建一次`template_node_label`，随后text command通过`field_label`再次生成同一标签。`workbench_field_metrics()`也分散在pixel alignment、search rect、search icon/fallback、surface、text、text inset和stepper helper中，常见field一次paint约7次以上读取全局metrics；text-field selector本身又有PERF-MVP-182记录的约5次palette读取。

PERF-MVP-191要求changed-node入口只构建一次`FieldPaintSpec`：projection提供typed normal/search/stepper role，spec持有一次生成的label、placeholder/state/style、单一theme/metrics/text snapshot以及resolved geometry。Surface、state layer、glyph、text和stepper command只借用spec；最终由PERF-MVP-178 compiled segment在stable generation直接复用。不得另加一个与presentation generation无关的field-local字符串缓存。

## 动态验收

在1/100/10,000 normal/search/stepper、empty/value、hover/focus/press/disabled field上记录lowercase String allocations/bytes、identity/classification、label build、style selector、palette/metrics/preferences reads、text measurements与command数。Changed field要求lowercase allocation=0，classification/style/label/measurement/theme snapshot各<=1；stable generation以上计数及field command build均为0。保持现有21文件覆盖的half-pixel geometry、search compact height/icon inset、placeholder tone、declared style、stepper offset/order与GPU/Softbuffer pixels parity。
