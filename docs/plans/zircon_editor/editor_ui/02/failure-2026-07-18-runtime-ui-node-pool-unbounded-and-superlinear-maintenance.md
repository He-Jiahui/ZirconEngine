---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-node-pool-unbounded-and-superlinear-maintenance
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Blueprint/UserWidgetPool.h
tests:
  - 100k-row pool entry-byte-age hard-cap test
  - bulk detach slot-visit and bulk insert paint-order counter
  - transient state reset and identity-rebind test
---

# Runtime UI node pool无界驻留与超线性维护

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface node pool与virtual-row consumer审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`
- 交接原因：virtualization、row reuse、slot/index和layout lifetime由EditorUI02统一拥有。

## 失败现象与复现证据

PERF-MVP-279：pool保存完整detached nodes但无count/byte/age cap；identity String key限制通用row reuse。detach原O(D×S)，insert原每node全树找max paint order。本轮已把slots过滤改为subtree set+单次retain。

## 最低共享层根因

pool没有family/type reusable identity、bulk mutation API或resource ownership budget，只是按完整authoring identity缓存任意数量UiTreeNode。

## 架构修复验收

- family/type pool复用时显式rebind node/control/path，清理focus/capture/IME/drag/style/layout transient state。
- per-class count、global bytes、idle age和shutdown release均有hard cap/drop stats。
- bulk detach/insert只更新一次slots/index并分配paint-order range；10k scroll稳定窗口alloc近0。
- 1/1k/100k rows记录create/reuse/evict、slot visits/full scans、pool bytes/age和CPU p95，Cargo通过。

## 禁止临时方案

- 不得只限制bucket Vec长度而不计完整metadata/String/TOML bytes及跨bucket总量。
- 不得复用node却保留旧input/drag/accessibility/resource owner状态。

## 修复结果与回传

Open state: `等待EditorUI02回传bounded family pool、bulk mutation和virtual-scroll规模证据`。
