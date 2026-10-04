---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_row_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_row_geometry
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_row_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_row_glyphs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_row_kind.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_row_kind
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_rows
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_rows_tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - inspector identity/geometry/resource/shadow/disclosure/pixel tests
  - current-source Windows Cargo running
  - bool mixed-case allocation regression pending
  - 1/100/10000 inspector-row text/resource/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template inspector rows逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_inspector_row_{geometry,glyphs,kind}*`、`template_inspector_rows*`与`template_inspector_rows_tests/**`共 **38/38** 个Rust文件、**1,446** 行已逐文件阅读。覆盖resource/disclosure/shadow row分类、geometry/style、text、mesh/material/check/chevron glyph及现有identity/resource/shadow/pixel tests。当前源Cargo、局部修复与规模trace未完成，因此仍留在`pending.md`。

## 已有共享根因

Mesh/material resource row在stable paint仍复制label/count/value 2至3个String并分别构建text command；leading icon与chevron各执行一次asset resolve，asset miss各展开3个quad。Disclosure与shadow row也重复资源解析或文本复制。这些成本回链PERF-MVP-174/178/179/181：presentation generation提交typed row和shared text，compiled segment持resource handle并在stable generation复用；不得建立inspector-local资源或字符串缓存。

## P1：布尔属性解析的无条件String分配

`bool_value`对每个Receive/Cast Shadows值先`trim().to_ascii_lowercase()`生成owned String，再匹配6个真值；`bool_display_value`也委托同一路径。它位于属性检查器的基础布尔控件paint路径，且可用`eq_ignore_ascii_case`等价完成。PERF-MVP-193要求先补mixed-case/whitespace真值与false值回归，再改为在静态候选上无分配比较；这是一项局部直接修复，不等待EditorUI08架构迁移。

## 动态验收

局部验收要求truth parsing String allocation/bytes=0，并保持`true/1/on/yes/check/checked`大小写不敏感及其他值为false。规模验收在1/100/10,000 resource/disclosure/shadow rows记录classification、text owned bytes、asset resolve/raster/copy、fallback segments与commands；stable generation以上计数均为0。保持现有38文件覆盖的row promotion、declared style、geometry、glyph order/resource与GPU/Softbuffer pixels parity。
