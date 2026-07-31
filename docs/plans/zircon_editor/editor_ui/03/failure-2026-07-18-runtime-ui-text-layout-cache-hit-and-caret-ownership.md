---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-text-layout-cache-hit-and-caret-ownership
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/ui/text/grapheme.rs
tests:
  - stable-frame zero source/layout deep-copy test
  - indexed line/cluster hit and caret zero-shape test
  - 100k-character interaction allocation and visit counter test
---

# Runtime UI text layout cache-hit与caret interaction所有权

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/text` tracked source 56/56
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md`
- 联动责任：Text03拥有cluster/line layout，Text09拥有cache/budget，EditorUI08消费generation artifact。
- 交接原因：UI layout product、editable interaction geometry与surface render text identity由EditorUI03统一整合。

## 失败现象与复现证据

PERF-MVP-298/299：frame/persistent cache hit仍深clone完整layout resolution；style key/width metric重复构造。hit-test原先复制valid advances，caret/selection/IME geometry逐boundary re-shape line prefix，line/navigation查询从头扫描。

本轮已让plain request在frame lookup前borrow source、frame miss后以单一Arc共享两层cache，并让valid hit advances borrow。完整证据见`docs/plans/performance/01/2026-07-18-runtime-ui-text-static-review.md`。

## 最低共享层根因

缓存返回owned layout DTO而不是generation handle；resolved line缺少可供hit/caret/range/navigation共享的interval与cluster prefix index，因此interaction只能复制或重新shape。

## 架构修复验收

- source/style/layout为generation-owned Arc artifact；frame/persistent hit只clone handle，正文和lines/runs/advances authoritative owner各1。
- compiled style identity缓存font/language normalization与width bucket metric；stable generation key metric work=0。
- layout包含line interval、cluster source↔visual map和prefix advances；hit/caret/range/navigation lookup近O(logL)+O(logG)或更优且shape calls=0。
- 1/10k/100k chars、1/100/10k lines、stable 300 frames和连续100k interaction记录clone/alloc bytes、owners、line/prefix visits、shape calls与CPU p95。
- preedit/BiDi/affinity/ligature/tab/justify/rich table/vertical、cache eviction/generation、Cargo、IME与像素通过。

## 禁止临时方案

- 不得新增不受source/style/font generation约束的第三层layout cache。
- 不得只把deep clone包入另一个DTO；consumer必须共享同一immutable artifact。
- 不得用fixed advance替代真实cluster prefix，或在hit handler旁私存stale geometry。

## 修复结果与回传

Open state: `等待EditorUI03联动Text03/09与EditorUI08回传generation-owned layout artifact、indexed zero-shape interaction、规模counter与current-source Cargo/产品trace`。
