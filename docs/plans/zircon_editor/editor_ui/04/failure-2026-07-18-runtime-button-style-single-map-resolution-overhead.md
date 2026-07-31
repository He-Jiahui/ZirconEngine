---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-button-style-single-map-resolution-overhead
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/style.rs
  - zircon_runtime/src/ui/tests/material_button_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector
---

# Runtime button style单表解析重复物化

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui` root/binding/event_ui/theme/tree生产文件28/28
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md`
- 联动责任：EditorUI08的presentation/style generation消费同一immutable theme snapshot；回链PERF-MVP-161/182/183。
- 交接原因：单次解析止损属于runtime helper，跨节点/帧的style与theme generation缓存属于EditorUI04。

## 失败现象与复现证据

PERF-MVP-251：button单表解析原先clone完整values map、构造Arc/Weak，并让17个属性逐项upgrade；字符串枚举再复制和lowercase。本轮已直接改为借用一次map，保留旧trait必实现入口并让内置属性转发borrowed values，消除Arc/Weak和lowercase分配。

## 最低共享层根因

runtime helper把已解析的单一values map重新包装为完整scope图，再通过通用多scope property链读取；上层又没有按style/theme/state generation保存resolved结果，导致结构复制和重复解析同时存在。

## 架构修复验收

- 按`style document + theme + component semantic state generation`缓存`ResolvedButtonStyle`，stable generation不重复解析。
- changed node compile从同一immutable theme/style snapshot借用值；不得在每个property或paint helper读取全局锁。
- 1/100/10k nodes记录map clone bytes、Arc upgrade、String alloc、property lookup、resolve count、cache bytes/hit/miss与CPU p50/p95。
- 单次resolve除custom style/role必要所有权外map clone、Arc upgrade、lowercase alloc均为0；stable generation resolve=0。
- variant/color/size/icon/state/loading/disabled、custom hex/style name、serde与editor software/WGPU视觉语义不变；current-source Cargo通过。

## 禁止临时方案

- 不得用无界全局String intern或无失效style cache换取0 allocation。
- 不得恢复每property Weak upgrade，或在paint阶段重新从String猜semantic role。
- 不得只优化button而让theme snapshot继续按helper读取全局锁并宣称完成。

## 修复结果与回传

Open state: `等待EditorUI04回传generation-owned resolved style/theme snapshot、规模counter、current-source Cargo与视觉等价证据`。
