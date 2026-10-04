---
related_code:
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime/src/ui/tests/material_layout
  - zircon_runtime/src/ui/layout/pass/material.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
tests:
  - 23 behavioral tests reviewed; 37 helper/direct surface layout executions represented
  - preflight attribute-rescan source guard observed RED then GREEN
  - rustfmt and scoped source predicate passed
  - current-source Windows Cargo and 1/100/10k-node lookup/allocation counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI material layout测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已逐文件完整阅读`zircon_runtime/src/ui/tests/material_layout.rs`与`material_layout/**`，共6/6个tracked Rust文件、831行。当前24个测试包含23个行为测试和本轮新增的1个源码性能守卫；行为测试通过helper或直接路径代表37次surface layout，覆盖button/icon、field/value/placeholder/options、row/label、child content、authored constraint及render command语义。

## 已直接止损：layout metric重复查表

`measure_material_content`原先先以`has_layout_attribute`最多查10个BTreeMap key，再判断component family，命中后又逐项读取相同10个metric；非Material component也可能先扫属性表。源码守卫先确认RED。本轮把component family分类提前，并让`MaterialLayoutMetrics::resolve`在每项唯一读取时同时记录是否authored，保留“key存在但值类型无效仍启用Material默认metric”的旧语义。守卫随后GREEN并通过`rustfmt`；动态Cargo因共享CPU FIFO未取得lane而仍pending。

## PERF-MVP-276/289：局部止损不是typed contract

新实现把layout metric的重复查表从最多20次收敛为10次，非Material component为0次，但每次layout仍从通用TOML读取10个metric及最多4个icon key；render又独立解析visible value/label/options并物化String。EditorUI02必须让compiled generation发布typed layout metrics，EditorUI04/05发布typed visual/text descriptor；stable generation的layout/render TOML lookup都应为0。不得在material pass和special renderer各建私有缓存。

## 现有测试门禁与缺口

测试对MVP控件像素尺寸和placeholder/editable value分离有较强语义约束，可作为typed cutover parity；但所有树都是1到3个节点，没有lookup、text resolve、String clone、layout cache hit、CPU/RSS计数。验收需以1/100/10k Material nodes记录metric/value/option probes、text/layout cache、allocation和p95，并跑当前源码Cargo、workbench Inspector/form产品trace与像素对拍。

## 验收结论

本切片不新增重复性能ID，局部修复计入PERF-MVP-276/289。Cargo、规模counter、产品trace和像素完成前，6/6继续留在`pending.md`，不进入`review.md`。
