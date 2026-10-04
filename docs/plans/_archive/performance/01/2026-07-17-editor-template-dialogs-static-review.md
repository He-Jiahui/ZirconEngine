---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - dialog identity/open-close/theme/metrics/severity/action layout/text/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - stable-open and 1/100 dialog metrics/palette/variant/text/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template dialogs逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_dialogs*`共 **22/22** 个Rust文件、**1,196** 行已逐文件阅读。覆盖dialog/confirm/alert identity与open gate、theme/metrics/severity/variant state、surface/content/action text measurement与responsive action layout，以及单元/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，stable-open产品trace尚未完成，因此仍留在`pending.md`。

## P1：打开modal重复metrics/palette/variant与action复制测量

Closed dialog在theme/text前退出，行为正确。打开confirm/alert的surface、severity、action frames、content geometry和text leaf各自获取metrics，典型confirm约14次。Surface/border/severity/title/body/confirm/cancel分别获取palette，按severity/destructive分支约8–10次，并重复扫描五个variant字段。两个action各执行owned `row_data`、label String复制和runtime text measurement；title/body再各复制String。

PERF-MVP-209要求changed dialog一次构建`DialogPaintSpec`，包含kind/open/unavailable/severity/confirm state、单一theme/metrics/palette snapshot、shared title/body/actions、一次action measurement/layout和ordered commands。Stable-open generation由PERF-MVP-178复用；closed dialog仍保持零工作。Modal数量通常为1，因此列为P1而非抢占row/list P0。

## 动态验收

在closed、stable-open 300 frames、dialog/confirm/alert、normal/disabled/loading/destructive/info/warning/error及wide/narrow/short布局上记录metrics/palette acquisition、variant/severity scans、action row_data/String/measurement、title/body bytes与build/commands。Closed全部=0；changed open各projection/classification<=1且每action measurement=1；stable-open全部及build=0。保持action enable/order/stacking、severity chrome、content fit、clip和GPU/Softbuffer pixels parity。
