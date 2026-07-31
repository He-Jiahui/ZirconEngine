---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-v2-surface-tree-layout-contract-reparse
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/layout/pass/material.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime/src/ui/tests/material_layout
  - zircon_runtime/src/ui/v2/surface_tree/node.rs
  - zircon_runtime/src/ui/v2/surface_tree/layout.rs
  - zircon_runtime/src/ui/v2/surface_tree/slot.rs
  - zircon_runtime/src/ui/v2/surface_tree/interaction.rs
tests:
  - compiled-generation layout parse-once counter
  - 10k-node surface build TOML-clone byte test
  - container slot responsive and fallback parity suite
---

# Runtime UI v2 SurfaceTree重复解析布局契约

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：v2 surface_tree 6/6逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`
- 联动责任：EditorUI05在canonical compiled arena承载typed contract。
- 交接原因：统一layout DTO、slot contract和Taffy投影由EditorUI02定稿。

## 失败现象与复现证据

PERF-MVP-276：SurfaceTree构建为每node重拼TOML attributes并解析layout/slot/interaction，单/双源layout都clone map。局部止损已把owned slot attributes直接move进metadata，删除两次额外map clone。Material measure原先还对10个layout metric执行presence预扫后再次解析；本轮已融合为单次解析并让非Material component提前退出，但stable generation仍每次从TOML读取metric，未替代typed contract。

## 最低共享层根因

compiled arena仍保存authoring TOML而非validated typed layout/slot/input contract，surface projection只能重复merge、parse、format和clone；编译与runtime布局之间缺少稳定DTO边界。

## 架构修复验收

- compiled generation内每node layout/slot/input contract parse≤1并携带source diagnostic。
- surface build只投影typed compact DTO，TOML parse=0；slot map额外clone=0，stable identity不逐nodeformat String。
- typed contract接入persistent layout tree/slot index，不以另一轮全树DTO重建替代当前parse。
- 1/100/10k nodes记录parse、map/value/String clone bytes、tree build CPU/RSS；全部container/slot/responsive/MUI/fallback parity与Cargo通过。

## 禁止临时方案

- 不得缓存merged TOML table作为长期runtime contract；权威必须是validated typed layout DTO。
- 不得在SurfaceTree内复制第二套container/slot语义解释器。

## 修复结果与回传

Open state: `等待EditorUI02回传typed compiled layout/slot contract、persistent projection和规模证据`。
